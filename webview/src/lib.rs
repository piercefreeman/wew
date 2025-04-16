mod page;

use std::{
    env::args,
    ffi::{c_char, c_int},
    sync::Arc,
    thread,
};

pub use self::page::{Page, PageObserver, PageOptions};

pub use webview_sys::{Modifiers, MouseButtons, PageState, TouchEventType, TouchPointerType};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActionState {
    Down,
    Up,
}

impl ActionState {
    pub fn is_pressed(self) -> bool {
        self == Self::Down
    }
}

#[derive(Debug, Clone)]
pub enum MouseAction {
    Click(MouseButtons, ActionState, Option<Position>),
    Move(Position),
    Wheel(Position),
}

#[derive(Debug)]
pub enum ImeAction<'a> {
    Composition(&'a str),
    Pre(&'a str, i32, i32),
}

pub(crate) struct Args(Vec<*const c_char>);

impl Default for Args {
    fn default() -> Self {
        Self(args().map(|it| ffi::into(&it)).collect::<Vec<_>>())
    }
}

impl Drop for Args {
    fn drop(&mut self) {
        for it in &self.0 {
            ffi::free(*it);
        }

        self.0.clear();
    }
}

impl Args {
    pub fn len(&self) -> c_int {
        self.0.len() as c_int
    }

    pub fn as_ptr(&self) -> *mut *const c_char {
        self.0.as_ptr() as _
    }
}

/// webview sub process does not work in tokio runtime!
pub fn execute_subprocess() -> Result<(), std::io::Error> {
    let args = Args::default();
    let code = unsafe { webview_sys::execute_subprocess(args.len(), args.as_ptr()) };
    if code == 0 {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("code = {}", code),
        ))
    }
}

pub fn is_subprocess() -> bool {
    args().find(|v| v.contains("--type")).is_some()
}

#[derive(Debug, Default)]
pub struct AppOptions<'a> {
    pub windowless_rendering_enabled: bool,
    pub cache_dir_path: Option<&'a str>,
    pub browser_subprocess_path: Option<&'a str>,
    pub scheme_dir_path: Option<&'a str>,
    #[cfg(target_os = "macos")]
    pub framework_dir_path: Option<&'a str>,
    #[cfg(target_os = "macos")]
    pub main_bundle_path: Option<&'a str>,
}

#[allow(unused_variables)]
pub trait AppObserver {
    fn on_context_initialized(&self) {}
    fn on_schedule_message_pump_work(&self, delay: u64) {}
}

pub struct App(Arc<wrapper::App>);

impl App {
    pub fn new<T>(options: &AppOptions<'_>, observer: T) -> Option<Self>
    where
        T: AppObserver + Send + Sync + 'static,
    {
        let inner = if let Some(it) = wrapper::App::new(&options, observer) {
            it
        } else {
            return None;
        };

        let inner = Arc::new(inner);
        if cfg!(target_os = "windows") {
            let inner_ = inner.clone();
            thread::spawn(move || {
                inner_.execute();
            });
        } else {
            inner.execute();
        }

        Some(Self(inner))
    }

    pub fn create_page<T>(&self, url: &str, options: &PageOptions, observer: T) -> Option<Page>
    where
        T: PageObserver + 'static,
    {
        self.0
            .create_page(url, options, observer)
            .map(|it| Page(it))
    }

    #[cfg(target_os = "macos")]
    pub fn run() {
        wrapper::MessageLoop::run();
    }

    #[cfg(target_os = "macos")]
    pub fn poll() {
        wrapper::MessageLoop::poll();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        wrapper::MessageLoop::quit();
    }
}

pub(crate) mod wrapper {
    use std::ffi::c_void;

    #[allow(unused_imports)]
    use webview_sys::{
        close_app, create_app, execute_app, poll_message_loop, quit_message_loop, run_message_loop,
    };

    use crate::{
        ffi, page::wrapper::Page, AppObserver, AppOptions, Args, PageObserver, PageOptions,
    };

    pub struct MessageLoop;

    impl MessageLoop {
        #[cfg(target_os = "macos")]
        pub fn run() {
            unsafe { run_message_loop() }
        }

        pub fn quit() {
            unsafe { quit_message_loop() }
        }

        #[cfg(target_os = "macos")]
        pub fn poll() {
            unsafe { poll_message_loop() }
        }
    }

    pub(crate) struct App {
        observer: *mut Box<dyn AppObserver>,
        pub ptr: *mut c_void,
    }

    unsafe impl Send for App {}
    unsafe impl Sync for App {}

    impl App {
        pub(crate) fn new<T>(options: &AppOptions, observer: T) -> Option<Self>
        where
            T: AppObserver + Send + Sync + 'static,
        {
            let mut options = webview_sys::AppOptions {
                cache_dir_path: ffi::into_opt(options.cache_dir_path),
                scheme_dir_path: ffi::into_opt(options.scheme_dir_path),
                browser_subprocess_path: ffi::into_opt(options.browser_subprocess_path),
                windowless_rendering_enabled: options.windowless_rendering_enabled,
                external_message_pump: cfg!(target_os = "macos"),
                multi_threaded_message_loop: !cfg!(target_os = "macos"),
                #[cfg(target_os = "macos")]
                main_bundle_path: ffi::into_opt(options.main_bundle_path),
                #[cfg(target_os = "macos")]
                framework_dir_path: ffi::into_opt(options.framework_dir_path),
                #[cfg(not(target_os = "macos"))]
                main_bundle_path: std::ptr::null(),
                #[cfg(not(target_os = "macos"))]
                framework_dir_path: std::ptr::null(),
            };

            let observer: *mut Box<dyn AppObserver> = Box::into_raw(Box::new(Box::new(observer)));
            let ptr = unsafe {
                create_app(
                    &mut options,
                    webview_sys::AppObserver {
                        on_context_initialized: Some(on_context_initialized),
                        on_schedule_message_pump_work: Some(on_schedule_message_pump_work),
                    },
                    observer as _,
                )
            };

            {
                ffi::free(options.cache_dir_path);
                ffi::free(options.scheme_dir_path);
                ffi::free(options.browser_subprocess_path);
            }

            if ptr.is_null() {
                return None;
            }

            Some(Self { observer, ptr })
        }

        pub(crate) fn create_page<T>(
            &self,
            url: &str,
            options: &PageOptions,
            observer: T,
        ) -> Option<Page>
        where
            T: PageObserver + 'static,
        {
            Page::new(&self, url, options, observer)
        }

        pub(crate) fn execute(&self) {
            let args = Args::default();
            unsafe {
                execute_app(self.ptr, args.len(), args.as_ptr());
            }
        }
    }

    impl Drop for App {
        fn drop(&mut self) {
            unsafe {
                close_app(self.ptr);
            }

            drop(unsafe { Box::from_raw(self.observer) });
        }
    }

    extern "C" fn on_context_initialized(ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn AppObserver>) }.on_context_initialized();
    }

    extern "C" fn on_schedule_message_pump_work(delay: i64, ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn AppObserver>) }.on_schedule_message_pump_work(delay as u64);
    }
}

pub mod ffi {
    use std::{
        ffi::{c_char, CStr, CString},
        ptr::null,
    };

    pub fn into(value: &str) -> *const c_char {
        CString::new(value).unwrap().into_raw()
    }

    pub fn into_opt(value: Option<&str>) -> *const c_char {
        value
            .map(|it| CString::new(it).unwrap().into_raw() as _)
            .unwrap_or_else(|| null())
    }

    pub fn from(value: *const c_char) -> Option<String> {
        if !value.is_null() {
            unsafe { CStr::from_ptr(value) }
                .to_str()
                .map(|s| s.to_string())
                .ok()
        } else {
            None
        }
    }

    pub fn free(value: *const c_char) {
        if !value.is_null() {
            drop(unsafe { CString::from_raw(value as _) })
        }
    }
}
