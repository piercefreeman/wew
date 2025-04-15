use std::{env, fs, path::Path, process::Command};

use anyhow::{anyhow, Result};

fn is_exsit(dir: &str) -> bool {
    fs::metadata(dir).is_ok()
}

fn join(root: &str, next: &str) -> String {
    Path::new(root).join(next).to_str().unwrap().to_string()
}

fn exec(command: &str, work_dir: &str) -> Result<String> {
    let output = Command::new(if cfg!(windows) { "powershell" } else { "bash" })
        .arg(if cfg!(windows) { "-command" } else { "-c" })
        .arg(if cfg!(windows) {
            format!("$ProgressPreference = 'SilentlyContinue';{}", command)
        } else {
            command.to_string()
        })
        .current_dir(work_dir)
        .output()?;
    if !output.status.success() {
        Err(anyhow!("{}", unsafe {
            String::from_utf8_unchecked(output.stderr)
        }))
    } else {
        Ok(unsafe { String::from_utf8_unchecked(output.stdout) })
    }
}

static URL: &'static str = "https://github.com/mycrl/webview-rs/releases/download/distributions";

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=./cxx");
    println!("cargo:rerun-if-changed=./src");
    println!("cargo:rerun-if-changed=./build.rs");

    let out_dir = env::var("OUT_DIR")?;
    let cef_path: &str = &join(&out_dir, "cef");

    #[cfg(target_os = "windows")]
    if !is_exsit(cef_path) {
        exec(
            &format!(
                "Invoke-WebRequest -Uri {URL}/cef-windows-{}.zip -OutFile ./cef.zip",
                env::var("CARGO_CFG_TARGET_ARCH")?
            ),
            &out_dir,
        )?;

        exec("Expand-Archive -Path cef.zip -DestinationPath ./", &out_dir)?;
        exec("Remove-Item ./cef.zip", &out_dir)?;
    }

    #[cfg(target_os = "macos")]
    if !is_exsit(cef_path) {
        exec(
            &format!(
                "wget {URL}/cef-macos-{}.zip -O ./cef.zip",
                env::var("CARGO_CFG_TARGET_ARCH")?
            ),
            &out_dir,
        )?;

        exec("tar -xf ./cef.zip -C ./", &out_dir)?;
        exec("rm -f ./cef.zip", &out_dir)?;
        exec(
            "mv ./cef/Release/cef_sandbox.a ./cef/Release/libcef_sandbox.a",
            &out_dir,
        )?;
    }

    if !is_exsit(&join(cef_path, "./libcef_dll_wrapper")) {
        #[cfg(not(target_os = "windows"))]
        exec(
            "cmake \
            -DCMAKE_CXX_FLAGS=\"-Wno-deprecated-builtins\" \
            -DCMAKE_BUILD_TYPE=Release .",
            cef_path,
        )?;

        #[cfg(target_os = "windows")]
        exec("cmake -DCMAKE_BUILD_TYPE=Release .", cef_path)?;

        exec("cmake --build . --config Release", cef_path)?;
    }

    bindgen::Builder::default()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .prepend_enum_name(false)
        .derive_eq(true)
        .size_t_is_usize(true)
        .header("./cxx/webview.h")
        .generate()?
        .write_to_file(&join(&out_dir, "bindings.rs"))?;

    {
        let mut cfgs = cc::Build::new();
        let is_debug = env::var("DEBUG")
            .map(|label| label == "true")
            .unwrap_or(true);

        cfgs.cpp(true)
            .debug(is_debug)
            .static_crt(true)
            .target(&env::var("TARGET")?)
            .warnings(false)
            .out_dir(&out_dir);

        if cfg!(target_os = "windows") {
            cfgs.flag("/std:c++20");
        } else {
            cfgs.flag("-std=c++20");
        }

        cfgs.file("./cxx/app.cpp")
            .file("./cxx/page.cpp")
            .file("./cxx/control.cpp")
            .file("./cxx/render.cpp")
            .file("./cxx/display.cpp")
            .file("./cxx/webview.cpp")
            .file("./cxx/scheme_handler.cpp");

        cfgs.include(cef_path);

        #[cfg(target_os = "windows")]
        cfgs.define("WIN32", Some("1"))
            .define("_WINDOWS", None)
            .define("__STDC_CONSTANT_MACROS", None)
            .define("__STDC_FORMAT_MACROS", None)
            .define("_WIN32", None)
            .define("UNICODE", None)
            .define("_UNICODE", None)
            .define("WINVER", Some("0x0A00"))
            .define("_WIN32_WINNT", Some("0x0A00"))
            .define("NTDDI_VERSION", Some("NTDDI_WIN10_FE"))
            .define("NOMINMAX", None)
            .define("WIN32_LEAN_AND_MEAN", None)
            .define("_HAS_EXCEPTIONS", Some("0"))
            .define("PSAPI_VERSION", Some("1"))
            .define("CEF_USE_SANDBOX", None)
            .define("CEF_USE_ATL", None)
            .define("_HAS_ITERATOR_DEBUGGING", Some("0"));

        #[cfg(target_os = "linux")]
        cfgs.define("LINUX", Some("1")).define("CEF_X11", Some("1"));

        #[cfg(target_os = "macos")]
        cfgs.define("MACOS", Some("1"));

        cfgs.compile("sys");
    }

    println!("cargo:rustc-link-lib=static=sys");
    println!("cargo:rustc-link-search=all={}", &out_dir);

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=libcef");
        println!("cargo:rustc-link-lib=libcef_dll_wrapper");
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./libcef_dll_wrapper/Release")
        );

        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./Release")
        );
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=cef");
        println!("cargo:rustc-link-lib=cef_dll_wrapper");
        println!("cargo:rustc-link-lib=X11");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework");
        println!(
            "cargo:rustc-link-search=framework={}",
            join(cef_path, "./Release")
        );

        println!("cargo:rustc-link-lib=cef_dll_wrapper");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./libcef_dll_wrapper")
        );

        println!(
            "cargo:rustc-link-search=native={}",
            join(cef_path, "Release")
        );
    }

    Ok(())
}
