use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use tempfile::TempDir;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "wrap_wew")]
#[command(about = "A CLI tool to build and package wew applications")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    entrypoint: PathBuf,
    
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

#[derive(Deserialize)]
struct CargoToml {
    package: Package,
    #[serde(default)]
    bin: Vec<BinaryTarget>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
struct BinaryTarget {
    name: String,
    path: String,
}

fn exec_command(cmd: &str, work_dir: &Path, env_vars: Option<HashMap<String, String>>) -> Result<String> {
    let shell = if cfg!(windows) { "powershell" } else { "bash" };
    let flag = if cfg!(windows) { "-command" } else { "-c" };
    let command = if cfg!(windows) {
        format!("$ProgressPreference = 'SilentlyContinue';{}", cmd)
    } else {
        cmd.to_string()
    };

    let mut process = Command::new(shell);
    process
        .arg(flag)
        .arg(&command)
        .current_dir(work_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    if let Some(env_vars) = env_vars {
        for (key, value) in env_vars {
            process.env(key, value);
        }
    }

    let output = process.output()
        .with_context(|| format!("Failed to execute command: {}", cmd))?;

    if !output.status.success() {
        return Err(anyhow!(
            "Command failed: {}\nStderr: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn find_crate_outdir(target_dir: &Path, crate_name: &str, subdir: &str) -> Result<PathBuf> {
    let build_dir = target_dir.join("build");
    
    if !build_dir.exists() {
        return Err(anyhow!("Build directory not found: {}", build_dir.display()));
    }

    // Find all matching directories and get the most recent one
    let mut candidates = Vec::new();
    
    for entry in fs::read_dir(&build_dir)? {
        let entry = entry?;
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        
        if dir_name_str.starts_with(crate_name) {
            let out_path = entry.path().join("out").join(subdir);
            if out_path.exists() {
                // Get the modification time to find the most recent
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        candidates.push((out_path, modified));
                    }
                }
            }
        }
    }
    
    if candidates.is_empty() {
        return Err(anyhow!("Crate outdir not found for {}", crate_name));
    }
    
    // Sort by modification time (most recent first) and return the most recent
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(candidates[0].0.clone())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {} to {}", src_path.display(), dst_path.display()))?;
        }
    }
    Ok(())
}

fn build_project(
    entrypoint: &Path,
    cargo_args: &[String],
    _temp_dir: &Path,
) -> Result<(PathBuf, String, CargoToml)> {
    // Read the Cargo.toml to get package name
    let cargo_toml_path = entrypoint.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow!("Cargo.toml not found at {}", cargo_toml_path.display()));
    }

    let cargo_content = fs::read_to_string(&cargo_toml_path)?;
    let cargo_toml: CargoToml = toml::from_str(&cargo_content)
        .with_context(|| "Failed to parse Cargo.toml")?;
    
    let package_name = cargo_toml.package.name.clone();

    // Build the project
    let mut cmd_args = vec!["build".to_string()];
    cmd_args.extend(cargo_args.iter().cloned());

    let mut env_vars = HashMap::new();
    if cfg!(target_os = "macos") {
        env_vars.insert("MACOSX_DEPLOYMENT_TARGET".to_string(), "15.4".to_string());
    }
    
    // Get the current working directory to find wew project root
    let current_dir = env::current_dir()?;
    env_vars.insert("CACHE_PATH".to_string(), current_dir.join("target/cache").to_string_lossy().to_string());

    let cargo_cmd = format!("cargo {}", cmd_args.join(" "));
    exec_command(&cargo_cmd, entrypoint, Some(env_vars))?;

    // Determine if this is a release build
    let is_release = cargo_args.contains(&"--release".to_string());
    let target_subdir = if is_release { "release" } else { "debug" };

    // Find target directory - look for workspace target directory first
    let target_dir = if let Some(workspace_root) = find_workspace_root(entrypoint)? {
        workspace_root.join("target").join(target_subdir)
    } else {
        entrypoint.join("target").join(target_subdir)
    };
    
    Ok((target_dir, package_name, cargo_toml))
}

fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>> {
    let mut current = start_path;
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(Some(current.to_path_buf()));
            }
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    
    Ok(None)
}

fn create_windows_package(
    target_dir: &Path,
    package_name: &str,
    temp_dir: &Path,
    cargo_toml: &CargoToml,
) -> Result<PathBuf> {
    let package_dir = temp_dir.join(&package_name);
    fs::create_dir_all(&package_dir)?;

    // Find CEF output directory
    let cef_out_dir = find_crate_outdir(&target_dir, "wew", "cef/Release")?
        .parent()
        .ok_or_else(|| anyhow!("Invalid CEF path"))?
        .to_path_buf();

    // Find main binary (not helper)
    let main_binary = cargo_toml.bin.iter()
        .find(|b| !b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No main binary found"))?;
    
    let main_binary_path = target_dir.join(format!("{}.exe", main_binary.name));
    if main_binary_path.exists() {
        fs::copy(&main_binary_path, package_dir.join(format!("{}.exe", package_name)))?;
    }

    // Find helper binary
    let helper_binary = cargo_toml.bin.iter()
        .find(|b| b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No helper binary found"))?;
    
    let helper_binary_path = target_dir.join(format!("{}.exe", helper_binary.name));
    if helper_binary_path.exists() {
        fs::copy(&helper_binary_path, package_dir.join(format!("{}-helper.exe", package_name)))?;
    }

    // Copy CEF Release and Resources directories
    let cef_release = cef_out_dir.join("Release");
    let cef_resources = cef_out_dir.join("Resources");
    
    if cef_release.exists() {
        copy_dir_all(&cef_release, &package_dir)?;
    }
    if cef_resources.exists() {
        copy_dir_all(&cef_resources, &package_dir)?;
    }

    // Remove unnecessary library files
    let files_to_remove = ["cef_sandbox.lib", "libcef.lib"];
    for file in &files_to_remove {
        let file_path = package_dir.join(file);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
    }

    Ok(package_dir)
}

fn create_macos_package(
    target_dir: &Path,
    package_name: &str,
    temp_dir: &Path,
    cargo_toml: &CargoToml,
) -> Result<PathBuf> {
    let app_dir = temp_dir.join(format!("{}.app", package_name));
    let contents_dir = app_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let frameworks_dir = contents_dir.join("Frameworks");

    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&frameworks_dir)?;

    // Find CEF Release directory
    let cef_release_path = find_crate_outdir(&target_dir, "wew", "cef/Release")?;

    // Find main binary (not helper)
    let main_binary = cargo_toml.bin.iter()
        .find(|b| !b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No main binary found"))?;
    
    let main_binary_path = target_dir.join(&main_binary.name);
    if main_binary_path.exists() {
        fs::copy(&main_binary_path, macos_dir.join(&package_name))?;
    }

    // Copy CEF framework
    let cef_framework_src = cef_release_path.join("Chromium Embedded Framework.framework");
    let cef_framework_dst = frameworks_dir.join("Chromium Embedded Framework.framework");
    if cef_framework_src.exists() {
        copy_dir_all(&cef_framework_src, &cef_framework_dst)?;
    }

    // Create Info.plist
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.github.mycrl.wew.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
</dict>
</plist>"#,
        package_name, package_name, package_name
    );
    fs::write(contents_dir.join("Info.plist"), info_plist)?;

    // Create helper applications
    let helper_configs = [
        (format!("{} Helper", package_name), format!("com.github.mycrl.wew.{}.helper", package_name)),
        (format!("{} Helper (GPU)", package_name), format!("com.github.mycrl.wew.{}.helper.gpu", package_name)),
        (format!("{} Helper (Plugin)", package_name), format!("com.github.mycrl.wew.{}.helper.plugin", package_name)),
        (format!("{} Helper (Renderer)", package_name), format!("com.github.mycrl.wew.{}.helper.renderer", package_name)),
    ];

    // Find helper binary
    let helper_binary = cargo_toml.bin.iter()
        .find(|b| b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No helper binary found"))?;
    
    let helper_binary_path = target_dir.join(&helper_binary.name);

    for (helper_name, helper_identifier) in helper_configs {
        let helper_app_dir = frameworks_dir.join(format!("{}.app", helper_name));
        let helper_contents_dir = helper_app_dir.join("Contents");
        let helper_macos_dir = helper_contents_dir.join("MacOS");
        let helper_resources_dir = helper_contents_dir.join("Resources");

        fs::create_dir_all(&helper_macos_dir)?;
        fs::create_dir_all(&helper_resources_dir)?;

        // Copy helper binary
        if helper_binary_path.exists() {
            fs::copy(&helper_binary_path, helper_macos_dir.join(&helper_name))?;
        }

        // Create helper Info.plist
        let helper_info_plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
</dict>
</plist>"#,
            helper_name, helper_identifier, helper_name
        );
        fs::write(helper_contents_dir.join("Info.plist"), helper_info_plist)?;

        // Fix library paths with install_name_tool
        let helper_binary_in_app = helper_macos_dir.join(&helper_name);
        if helper_binary_in_app.exists() {
            exec_command(
                &format!(
                    "install_name_tool -change \
                    \"@executable_path/../Frameworks/Chromium Embedded Framework.framework/Chromium Embedded Framework\" \
                    \"@rpath/Chromium Embedded Framework.framework/Chromium Embedded Framework\" \
                    \"{}\"",
                    helper_binary_in_app.display()
                ),
                &temp_dir,
                None,
            )?;

            exec_command(
                &format!(
                    "install_name_tool \
                    -add_rpath \"@executable_path/../../../../Frameworks\" \
                    \"{}\"",
                    helper_binary_in_app.display()
                ),
                &temp_dir,
                None,
            )?;
        }
    }

    Ok(app_dir)
}

fn create_linux_package(
    target_dir: &Path,
    package_name: &str,
    temp_dir: &Path,
    cargo_toml: &CargoToml,
) -> Result<PathBuf> {
    let package_dir = temp_dir.join(&package_name);
    fs::create_dir_all(&package_dir)?;

    // Find CEF output directory
    let cef_out_dir = find_crate_outdir(&target_dir, "wew", "cef/Release")?
        .parent()
        .ok_or_else(|| anyhow!("Invalid CEF path"))?
        .to_path_buf();

    // Find main binary (not helper)
    let main_binary = cargo_toml.bin.iter()
        .find(|b| !b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No main binary found"))?;
    
    let main_binary_path = target_dir.join(&main_binary.name);
    if main_binary_path.exists() {
        fs::copy(&main_binary_path, package_dir.join(&package_name))?;
    }

    // Find helper binary
    let helper_binary = cargo_toml.bin.iter()
        .find(|b| b.name.contains("helper"))
        .ok_or_else(|| anyhow!("No helper binary found"))?;
    
    let helper_binary_path = target_dir.join(&helper_binary.name);
    if helper_binary_path.exists() {
        fs::copy(&helper_binary_path, package_dir.join(format!("{}-helper", package_name)))?;
    }

    // Copy CEF Release and Resources directories
    let cef_release = cef_out_dir.join("Release");
    let cef_resources = cef_out_dir.join("Resources");
    
    if cef_release.exists() {
        copy_dir_all(&cef_release, &package_dir)?;
    }
    if cef_resources.exists() {
        copy_dir_all(&cef_resources, &package_dir)?;
    }

    println!("Note: add {} to LD_LIBRARY_PATH env", package_dir.display());

    Ok(package_dir)
}

fn create_tar_archive(source_dir: &Path, output_path: &Path) -> Result<()> {
    let tar_file = File::create(output_path)?;
    let mut tar = tar::Builder::new(tar_file);

    // Get the directory name to use as the root in the tar
    let dir_name = source_dir.file_name()
        .ok_or_else(|| anyhow!("Invalid source directory"))?;

    // Add the root directory first
    tar.append_dir(dir_name, source_dir)?;

    // Add all files and directories from source_dir to the tar
    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(source_dir)?;

        // Skip the root directory since we already added it
        if relative_path == Path::new("") {
            continue;
        }

        // Create the tar path with the directory name as root
        let tar_path = Path::new(dir_name).join(relative_path);

        if path.is_file() {
            tar.append_path_with_name(path, tar_path)?;
        } else if path.is_dir() {
            tar.append_dir(tar_path, path)?;
        }
    }

    tar.finish()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let entrypoint = cli.entrypoint.canonicalize()
        .with_context(|| format!("Invalid entrypoint path: {}", cli.entrypoint.display()))?;

    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    println!("Building project at {}", entrypoint.display());
    
    // Build the project
    let (target_dir, package_name, cargo_toml) = build_project(&entrypoint, &cli.cargo_args, temp_path)?;
    
    println!("Creating package for {}", package_name);

    // Create platform-specific package
    let package_dir = if cfg!(target_os = "windows") {
        create_windows_package(&target_dir, &package_name, temp_path, &cargo_toml)?
    } else if cfg!(target_os = "macos") {
        // For macOS, we need to wrap the .app bundle in a directory named after the package
        let app_bundle = create_macos_package(&target_dir, &package_name, temp_path, &cargo_toml)?;
        let package_wrapper = temp_path.join(&package_name);
        fs::create_dir_all(&package_wrapper)?;
        
        // Move the .app bundle into the wrapper directory
        let app_name = format!("{}.app", package_name);
        let final_app_path = package_wrapper.join(&app_name);
        fs::rename(&app_bundle, &final_app_path)?;
        
        package_wrapper
    } else {
        create_linux_package(&target_dir, &package_name, temp_path, &cargo_toml)?
    };

    // Create tar archive in current working directory
    let current_dir = env::current_dir()?;
    let tar_name = format!("{}.tar", package_name);
    let tar_path = current_dir.join(&tar_name);

    println!("Creating archive: {}", tar_path.display());
    create_tar_archive(&package_dir, &tar_path)?;

    println!("Successfully created {}", tar_name);
    
    Ok(())
} 