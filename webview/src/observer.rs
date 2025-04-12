use webview_sys::{PageState, Rect};

#[allow(unused)]
pub trait Observer: Send + Sync {
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

pub(crate) mod wrapper {
    use std::{
        ffi::{c_char, c_int, c_void},
        slice::from_raw_parts,
    };

    use webview_sys::{PageObserver, PageState, Rect};

    use crate::ffi;

    pub fn create_page_observer() -> PageObserver {
        PageObserver {
            on_state_change: Some(on_state_change_callback),
            on_ime_rect: Some(on_ime_rect_callback),
            on_frame: Some(on_frame_callback),
            on_title_change: Some(on_title_change_callback),
            on_fullscreen_change: Some(on_fullscreen_change_callback),
            on_message: Some(on_message_callback),
        }
    }

    /// Implement this interface to handle events related to browser load
    /// status.
    ///
    /// The methods of this class will be called on the browser process UI
    /// thread or render process main thread (TID_RENDERER).
    extern "C" fn on_state_change_callback(state: PageState, ctx: *mut c_void) {
        unsafe { &*(ctx as *mut Box<dyn super::Observer>) }.on_state_change(state);
    }

    /// Called when the IME composition range has changed.
    ///
    /// selected_range is the range of characters that have been selected.
    /// |character_bounds| is the bounds of each character in view coordinates.
    extern "C" fn on_ime_rect_callback(rect: Rect, ctx: *mut c_void) {
        (unsafe { &*(ctx as *mut Box<dyn super::Observer>) }).on_ime_rect(rect);
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
        (unsafe { &*(ctx as *mut Box<dyn super::Observer>) }).on_frame(
            unsafe { from_raw_parts(texture as _, width as usize * height as usize * 4) },
            width as u32,
            height as u32,
        );
    }

    /// Called when the page title changes.
    extern "C" fn on_title_change_callback(title: *const c_char, ctx: *mut c_void) {
        if let Some(title) = ffi::from(title) {
            (unsafe { &*(ctx as *mut Box<dyn super::Observer>) })
                .on_title_change(title);
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
        (unsafe { &*(ctx as *mut Box<dyn super::Observer>) })
            .on_fullscreen_change(fullscreen);
    }

    extern "C" fn on_message_callback(message: *const c_char, ctx: *mut c_void) {
        if let Some(message) = ffi::from(message) {
            (unsafe { &*(ctx as *mut Box<dyn super::Observer>) }).on_message(message);
        }
    }
}
