use raw_window_handle::RawWindowHandle;
use webview_sys::{Modifiers, PageState, Rect, TouchEventType, TouchPointerType};

use crate::{ActionState, ImeAction, MouseAction};

#[derive(Debug)]
pub struct PageOptions {
    /// External native window handle.
    pub window_handle: Option<RawWindowHandle>,
    /// The maximum rate in frames per second (fps) that CefRenderHandler::OnPaint
    /// will be called for a windowless browser.
    pub windowless_frame_rate: u32,
    /// window size width.
    pub width: u32,
    /// window size height.
    pub height: u32,
    /// window device scale factor.
    pub device_scale_factor: f32,
    /// page defalt fixed font size.
    pub default_font_size: u32,
    /// page defalt fixed font size.
    pub default_fixed_font_size: u32,
    /// Controls whether JavaScript can be executed.
    pub javascript: bool,
    /// Controls whether JavaScript can access the clipboard.
    pub javascript_access_clipboard: bool,
    /// Controls whether local storage can be used.
    pub local_storage: bool,
}

unsafe impl Send for PageOptions {}
unsafe impl Sync for PageOptions {}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            window_handle: None,
            device_scale_factor: 1.0,
            windowless_frame_rate: 30,
            default_font_size: 12,
            default_fixed_font_size: 12,
            javascript: true,
            javascript_access_clipboard: false,
            local_storage: true,
        }
    }
}

#[allow(unused)]
pub trait PageObserver: Send + Sync {
    /// Implement this interface to handle events related to browser load
    /// status.
    ///
    /// The methods of this class will be called on the browser process UI
    /// thread or render process main thread (TID_RENDERER).
    fn on_state_change(&self, state: PageState) {}
    /// Called when the IME composition range has changed.
    ///
    /// selected_range is the range of characters that have been selected.
    /// |character_bounds| is the bounds of each character in view coordinates.
    fn on_ime_rect(&self, rect: Rect) {}
    /// Called when an element should be painted.
    ///
    /// Pixel values passed to this method are scaled relative to view
    /// coordinates based on the value of CefScreenInfo.device_scale_factor
    /// returned from GetScreenInfo. |type| indicates whether the element is the
    /// view or the popup widget. |buffer| contains the pixel data for the whole
    /// image. |dirtyRects| contains the set of rectangles in pixel coordinates
    /// that need to be repainted. |buffer| will be |width|*|height|*4 bytes in
    /// size and represents a BGRA image with an upper-left origin. This method
    /// is only called when CefWindowInfo::shared_texture_enabled is set to
    /// false.
    fn on_frame(&self, texture: &[u8], width: u32, height: u32) {}
    /// Called when the page title changes.
    fn on_title_change(&self, title: String) {}
    /// Called when web content in the page has toggled fullscreen mode.
    ///
    /// If |fullscreen| is true the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling CefWindow::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The CefWindowDelegate::OnWindowFullscreenTransition method will be
    /// called during the fullscreen transition for notification purposes.
    fn on_fullscreen_change(&self, fullscreen: bool) {}
    fn on_message(&self, message: String) {}
}

/// CefClient
///
/// The CefClient interface provides access to browser-instance-specific
/// callbacks. A single CefClient instance can be shared among any number of
/// browsers. Important callbacks include:
///
/// Handlers for things like browser life span, context menus, dialogs, display
/// notifications, drag events, focus events, keyboard events and more. The
/// majority of handlers are optional. See the class documentation for the side
/// effects, if any, of not implementing a specific handler.
///
/// OnProcessMessageReceived which is called when an IPC message is received
/// from the render process. See the “Inter-Process Communication” section for
/// more information.
///
/// An example CefClient implementation can be seen in
/// cefsimple/simple_handler.h and cefsimple/simple_handler.cc.
pub struct Page(pub(crate) wrapper::Page);

impl Page {
    /// Send a mouse click event to the browser.
    ///
    /// Send a mouse move event to the browser.
    ///
    /// Send a mouse wheel event to the browser.
    pub fn mouse(&self, action: MouseAction) {
        self.0.mouse(action);
    }

    /// Send a key event to the browser.
    pub fn keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
        self.0.keyboard(scan_code, state, modifiers);
    }

    /// Send a touch event to the browser for a windowless browser.
    pub fn touch(
        &self,
        id: i32,
        x: i32,
        y: i32,
        ty: TouchEventType,
        pointer_type: TouchPointerType,
    ) {
        self.0.touch(id, x, y, ty, pointer_type);
    }

    /// Completes the existing composition by optionally inserting the specified
    /// |text| into the composition node.
    ///
    /// Begins a new composition or updates the existing composition.
    ///
    /// Blink has a special node (a composition node) that allows the input
    /// method to change text without affecting other DOM nodes. |text| is the
    /// optional text that will be inserted into the composition node.
    /// |underlines| is an optional set of ranges that will be underlined in the
    /// resulting text. |replacement_range| is an optional range of the existing
    /// text that will be replaced. |selection_range| is an optional range of
    /// the resulting text that will be selected after insertion or replacement.
    /// The |replacement_range| value is only used on OS X.
    ///
    /// This method may be called multiple times as the composition changes.
    /// When the client is done making changes the composition should either be
    /// canceled or completed. To cancel the composition call
    /// ImeCancelComposition. To complete the composition call either
    /// ImeCommitText or ImeFinishComposingText. Completion is usually signaled
    /// when:
    ///
    /// 1, The client receives a WM_IME_COMPOSITION message with a GCS_RESULTSTR
    /// flag (on Windows), or; 2, The client receives a "commit" signal of
    /// GtkIMContext (on Linux), or; 3, insertText of NSTextInput is called
    /// (on Mac).
    ///
    /// This method is only used when window rendering is disabled.
    pub fn ime(&self, action: ImeAction) {
        self.0.ime(action);
    }

    /// Notify the browser that the widget has been resized.
    ///
    /// The browser will first call CefRenderHandler::GetViewRect to get the new
    /// size and then call CefRenderHandler::OnPaint asynchronously with the
    /// updated regions. This method is only used when window rendering is
    /// disabled.
    pub fn resize(&self, width: u32, height: u32) {
        self.0.resize(width, height);
    }

    /// Retrieve the window handle (if any) for this browser.
    ///
    /// If this browser is wrapped in a CefBrowserView this method should be
    /// called on the browser process UI thread and it will return the handle
    /// for the top-level native window.
    pub fn window_handle(&self) -> RawWindowHandle {
        self.0.window_handle()
    }

    /// Open developer tools (DevTools) in its own browser.
    ///
    /// The DevTools browser will remain associated with this browser.
    pub fn set_devtools_state(&self, is_open: bool) {
        self.0.set_devtools_state(is_open);
    }

    pub fn send_message(&self, message: &str) {
        self.0.send_message(message);
    }
}

pub(crate) mod wrapper {
    use std::{
        ffi::{c_char, c_int, c_void},
        num::NonZeroIsize,
        ptr::null,
        slice::from_raw_parts,
    };

    use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
    use webview_sys::{
        close_page, create_page, page_get_hwnd, page_resize, page_send_ime_composition,
        page_send_ime_set_composition, page_send_keyboard, page_send_message,
        page_send_mouse_click, page_send_mouse_click_with_pos, page_send_mouse_move,
        page_send_mouse_wheel, page_send_touch, page_set_devtools_state, Modifiers, PageState,
        Rect, TouchEventType, TouchPointerType,
    };

    use crate::{ffi, wrapper::App, ActionState, ImeAction, MouseAction};

    use super::{PageObserver, PageOptions};

    /// CefClient
    ///
    /// The CefClient interface provides access to browser-instance-specific
    /// callbacks. A single CefClient instance can be shared among any number of
    /// browsers. Important callbacks include:
    ///
    /// Handlers for things like browser life span, context menus, dialogs, display
    /// notifications, drag events, focus events, keyboard events and more. The
    /// majority of handlers are optional. See the class documentation for the side
    /// effects, if any, of not implementing a specific handler.
    ///
    /// OnProcessMessageReceived which is called when an IPC message is received
    /// from the render process. See the “Inter-Process Communication” section for
    /// more information.
    ///
    /// An example CefClient implementation can be seen in
    /// cefsimple/simple_handler.h and cefsimple/simple_handler.cc.
    pub(crate) struct Page {
        pub observer: *mut Box<dyn PageObserver>,
        pub raw: *mut c_void,
    }

    unsafe impl Send for Page {}
    unsafe impl Sync for Page {}

    impl Page {
        pub(crate) fn new<T>(
            app: &App,
            url: &str,
            options: &PageOptions,
            observer: T,
        ) -> Option<Self>
        where
            T: PageObserver + 'static,
        {
            let options = webview_sys::PageOptions {
                width: options.width,
                height: options.height,
                device_scale_factor: options.device_scale_factor,
                windowless_frame_rate: options.windowless_frame_rate,
                default_fixed_font_size: options.default_fixed_font_size as c_int,
                default_font_size: options.default_font_size as c_int,
                javascript: options.javascript,
                javascript_access_clipboard: options.javascript_access_clipboard,
                local_storage: options.local_storage,
                window_handle: if let Some(it) = options.window_handle {
                    match it {
                        RawWindowHandle::Win32(it) => it.hwnd.get() as _,
                        RawWindowHandle::AppKit(it) => it.ns_view.as_ptr() as _,
                        _ => unimplemented!("{:?}", it),
                    }
                } else {
                    null()
                },
            };

            let url = ffi::into(url);
            let observer: *mut Box<dyn PageObserver> = Box::into_raw(Box::new(Box::new(observer)));
            let raw = unsafe {
                create_page(
                    app.ptr,
                    url,
                    &options,
                    webview_sys::PageObserver {
                        on_state_change: Some(on_state_change_callback),
                        on_ime_rect: Some(on_ime_rect_callback),
                        on_frame: Some(on_frame_callback),
                        on_title_change: Some(on_title_change_callback),
                        on_fullscreen_change: Some(on_fullscreen_change_callback),
                        on_message: Some(on_message_callback),
                    },
                    observer as _,
                )
            };

            {
                ffi::free(url);
            }

            if raw.is_null() {
                return None;
            }

            Some(Self { observer, raw })
        }

        pub(crate) fn send_message(&self, message: &str) {
            let message = ffi::into(message);

            unsafe {
                page_send_message(self.raw, message);
            }

            ffi::free(message);
        }

        /// Send a mouse click event to the browser.
        ///
        /// Send a mouse move event to the browser.
        ///
        /// Send a mouse wheel event to the browser.
        pub fn mouse(&self, action: MouseAction) {
            match action {
                MouseAction::Move(pos) => unsafe { page_send_mouse_move(self.raw, pos.x, pos.y) },
                MouseAction::Wheel(pos) => unsafe { page_send_mouse_wheel(self.raw, pos.x, pos.y) },
                MouseAction::Click(button, state, pos) => {
                    if let Some(pos) = pos {
                        unsafe {
                            page_send_mouse_click_with_pos(
                                self.raw,
                                button,
                                state.is_pressed(),
                                pos.x,
                                pos.y,
                            )
                        }
                    } else {
                        unsafe { page_send_mouse_click(self.raw, button, state.is_pressed()) }
                    }
                }
            }
        }

        /// Send a key event to the browser.
        pub fn keyboard(&self, scan_code: u32, state: ActionState, modifiers: Modifiers) {
            unsafe {
                page_send_keyboard(self.raw, scan_code as c_int, state.is_pressed(), modifiers)
            }
        }

        /// Send a touch event to the browser for a windowless browser.
        pub fn touch(
            &self,
            id: i32,
            x: i32,
            y: i32,
            ty: TouchEventType,
            pointer_type: TouchPointerType,
        ) {
            unsafe { page_send_touch(self.raw, id, x, y, ty, pointer_type) }
        }

        /// Completes the existing composition by optionally inserting the specified
        /// |text| into the composition node.
        ///
        /// Begins a new composition or updates the existing composition.
        ///
        /// Blink has a special node (a composition node) that allows the input
        /// method to change text without affecting other DOM nodes. |text| is the
        /// optional text that will be inserted into the composition node.
        /// |underlines| is an optional set of ranges that will be underlined in the
        /// resulting text. |replacement_range| is an optional range of the existing
        /// text that will be replaced. |selection_range| is an optional range of
        /// the resulting text that will be selected after insertion or replacement.
        /// The |replacement_range| value is only used on OS X.
        ///
        /// This method may be called multiple times as the composition changes.
        /// When the client is done making changes the composition should either be
        /// canceled or completed. To cancel the composition call
        /// ImeCancelComposition. To complete the composition call either
        /// ImeCommitText or ImeFinishComposingText. Completion is usually signaled
        /// when:
        ///
        /// 1, The client receives a WM_IME_COMPOSITION message with a GCS_RESULTSTR
        /// flag (on Windows), or; 2, The client receives a "commit" signal of
        /// GtkIMContext (on Linux), or; 3, insertText of NSTextInput is called
        /// (on Mac).
        ///
        /// This method is only used when window rendering is disabled.
        pub fn ime(&self, action: ImeAction) {
            let input = match action {
                ImeAction::Composition(it) | ImeAction::Pre(it, _, _) => ffi::into(it),
            };

            match action {
                ImeAction::Composition(_) => unsafe { page_send_ime_composition(self.raw, input) },
                ImeAction::Pre(_, x, y) => unsafe {
                    page_send_ime_set_composition(self.raw, input, x, y)
                },
            }

            ffi::free(input);
        }

        /// Notify the browser that the widget has been resized.
        ///
        /// The browser will first call CefRenderHandler::GetViewRect to get the new
        /// size and then call CefRenderHandler::OnPaint asynchronously with the
        /// updated regions. This method is only used when window rendering is
        /// disabled.
        pub fn resize(&self, width: u32, height: u32) {
            unsafe { page_resize(self.raw, width as c_int, height as c_int) }
        }

        /// Retrieve the window handle (if any) for this browser.
        ///
        /// If this browser is wrapped in a CefBrowserView this method should be
        /// called on the browser process UI thread and it will return the handle
        /// for the top-level native window.
        pub fn window_handle(&self) -> RawWindowHandle {
            RawWindowHandle::Win32(Win32WindowHandle::new(
                NonZeroIsize::new(unsafe { page_get_hwnd(self.raw) as _ }).unwrap(),
            ))
        }

        /// Open developer tools (DevTools) in its own browser.
        ///
        /// The DevTools browser will remain associated with this browser.
        pub fn set_devtools_state(&self, is_open: bool) {
            unsafe { page_set_devtools_state(self.raw, is_open) }
        }
    }

    impl Drop for Page {
        fn drop(&mut self) {
            unsafe {
                close_page(self.raw);
            }

            drop(unsafe { Box::from_raw(self.observer) });
        }
    }

    /// Implement this interface to handle events related to browser load
    /// status.
    ///
    /// The methods of this class will be called on the browser process UI
    /// thread or render process main thread (TID_RENDERER).
    extern "C" fn on_state_change_callback(state: PageState, ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn PageObserver>) }.on_state_change(state);
    }

    /// Called when the IME composition range has changed.
    ///
    /// selected_range is the range of characters that have been selected.
    /// |character_bounds| is the bounds of each character in view coordinates.
    extern "C" fn on_ime_rect_callback(rect: Rect, ctx: *mut c_void) {
        (unsafe { &*(ctx as *mut Box<dyn PageObserver>) }).on_ime_rect(rect);
    }

    /// Called when an element should be painted.
    ///
    /// Pixel values passed to this method are scaled relative to view
    /// coordinates based on the value of CefScreenInfo.device_scale_factor
    /// returned from GetScreenInfo. |type| indicates whether the element is the
    /// view or the popup widget. |buffer| contains the pixel data for the whole
    /// image. |dirtyRects| contains the set of rectangles in pixel coordinates
    /// that need to be repainted. |buffer| will be |width|*|height|*4 bytes in
    /// size and represents a BGRA image with an upper-left origin. This method
    /// is only called when CefWindowInfo::shared_texture_enabled is set to
    /// false.
    extern "C" fn on_frame_callback(
        texture: *const c_void,
        width: c_int,
        height: c_int,
        ctx: *mut c_void,
    ) {
        (unsafe { &*(ctx as *mut Box<dyn PageObserver>) }).on_frame(
            unsafe { from_raw_parts(texture as _, width as usize * height as usize * 4) },
            width as u32,
            height as u32,
        );
    }

    /// Called when the page title changes.
    extern "C" fn on_title_change_callback(title: *const c_char, ctx: *mut c_void) {
        if let Some(title) = ffi::from(title) {
            (unsafe { &*(ctx as *mut Box<dyn PageObserver>) }).on_title_change(title);
        }
    }

    /// Called when web content in the page has toggled fullscreen mode.
    ///
    /// If |fullscreen| is true the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling CefWindow::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The CefWindowDelegate::OnWindowFullscreenTransition method will be
    /// called during the fullscreen transition for notification purposes.
    extern "C" fn on_fullscreen_change_callback(fullscreen: bool, ctx: *mut c_void) {
        (unsafe { &*(ctx as *mut Box<dyn PageObserver>) }).on_fullscreen_change(fullscreen);
    }

    extern "C" fn on_message_callback(message: *const c_char, ctx: *mut c_void) {
        if let Some(message) = ffi::from(message) {
            (unsafe { &*(ctx as *mut Box<dyn PageObserver>) }).on_message(message);
        }
    }
}
