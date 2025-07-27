//
//  wew.h
//  webview
//
//  Created by mycrl on 2025/6/19.
//

#ifndef wew_h
#define wew_h
#pragma once

#ifdef WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct
{
    int x;
    int y;
    int width;
    int height;
} Rect;

typedef struct
{
    const char *url;
    const char *method;
    const char *referrer;
} Request;

typedef struct
{
    int status_code;
    uint64_t content_length;
    char *mime_type;
} Response;

typedef struct
{
    bool (*open)(void *context);
    bool (*skip)(size_t size, int *cursor, void *context);
    bool (*read)(uint8_t *buffer, size_t size, int *cursor, void *context);
    void (*get_response)(Response *response, void *context);
    void (*cancel)(void *context);
    void (*destroy)(void *context);
    void *context;
} RequestHandler;

typedef struct
{
    RequestHandler *(*request)(Request *request, void *context);
    void (*destroy_request_handler)(RequestHandler *handler);
    void *context;
} RequestHandlerFactory;

typedef struct
{
    const char *name;
    const char *domain;
    const RequestHandlerFactory *factory;
} CustomSchemeAttributes;

typedef struct
{
    const char *name;
    const char *value;
    const char *domain;
    const char *path;
    bool secure;
    bool httponly;
    int64_t expires;
    bool has_expires;
    int64_t creation;
    int64_t last_access;
    int same_site;  // 0 = unspecified, 1 = no_restriction, 2 = lax_mode, 3 = strict_mode
    int priority;   // 0 = low, 1 = medium, 2 = high
} Cookie;

typedef struct
{
    bool (*visit)(const Cookie *cookie, int count, int total, bool *delete_cookie, void *context);
    void (*destroy)(void *context);
    void *context;
} CookieVisitor;

///
/// Cursor type values.
///
typedef enum
{
    WEW_CT_POINTER,
    WEW_CT_CROSS,
    WEW_CT_HAND,
    WEW_CT_IBEAM,
    WEW_CT_WAIT,
    WEW_CT_HELP,
    WEW_CT_EASTRESIZE,
    WEW_CT_NORTHRESIZE,
    WEW_CT_NORTHEASTRESIZE,
    WEW_CT_NORTHWESTRESIZE,
    WEW_CT_SOUTHRESIZE,
    WEW_CT_SOUTHEASTRESIZE,
    WEW_CT_SOUTHWESTRESIZE,
    WEW_CT_WESTRESIZE,
    WEW_CT_NORTHSOUTHRESIZE,
    WEW_CT_EASTWESTRESIZE,
    WEW_CT_NORTHEASTSOUTHWESTRESIZE,
    WEW_CT_NORTHWESTSOUTHEASTRESIZE,
    WEW_CT_COLUMNRESIZE,
    WEW_CT_ROWRESIZE,
    WEW_CT_MIDDLEPANNING,
    WEW_CT_EASTPANNING,
    WEW_CT_NORTHPANNING,
    WEW_CT_NORTHEASTPANNING,
    WEW_CT_NORTHWESTPANNING,
    WEW_CT_SOUTHPANNING,
    WEW_CT_SOUTHEASTPANNING,
    WEW_CT_SOUTHWESTPANNING,
    WEW_CT_WESTPANNING,
    WEW_CT_MOVE,
    WEW_CT_VERTICALTEXT,
    WEW_CT_CELL,
    WEW_CT_CONTEXTMENU,
    WEW_CT_ALIAS,
    WEW_CT_PROGRESS,
    WEW_CT_NODROP,
    WEW_CT_COPY,
    WEW_CT_NONE,
    WEW_CT_NOTALLOWED,
    WEW_CT_ZOOMIN,
    WEW_CT_ZOOMOUT,
    WEW_CT_GRAB,
    WEW_CT_GRABBING,
    WEW_CT_MIDDLE_PANNING_VERTICAL,
    WEW_CT_MIDDLE_PANNING_HORIZONTAL,
    WEW_CT_CUSTOM,
    WEW_CT_DND_NONE,
    WEW_CT_DND_MOVE,
    WEW_CT_DND_COPY,
    WEW_CT_DND_LINK,
    WEW_CT_NUM_VALUES,
} CursorType;

typedef enum
{
    ///
    /// Default logging (currently INFO logging).
    ///
    WEW_LOG_DEFAULT,

    ///
    /// Verbose logging.
    ///
    WEW_LOG_VERBOSE,

    ///
    /// DEBUG logging.
    ///
    WEW_LOG_DEBUG = WEW_LOG_VERBOSE,

    ///
    /// INFO logging.
    ///
    WEW_LOG_INFO,

    ///
    /// WARNING logging.
    ///
    WEW_LOG_WARNING,

    ///
    /// ERROR logging.
    ///
    WEW_LOG_ERROR,

    ///
    /// FATAL logging.
    ///
    WEW_LOG_FATAL,

    ///
    /// Disable logging to file for all messages, and to stderr for messages with
    /// severity less than FATAL.
    ///
    WEW_LOG_DISABLE = 99
} LogLevel;

typedef struct
{
    const CustomSchemeAttributes *custom_scheme;

    /// The directory where data for the global browser cache will be stored on disk.
    const char *cache_path;

    /// The root directory for installation-specific data and the parent directory for profile-specific data.
    const char *root_cache_path;

    /// The path to a separate executable that will be launched for sub-processes.
    const char *browser_subprocess_path;

    /// Set to true (1) to enable windowless (off-screen) rendering support.
    ///
    /// Do not enable this value if the application does not use windowless rendering as it may reduce
    /// rendering performance on some systems.
    bool windowless_rendering_enabled;

    /// Set to true (1) to control browser process main (UI) thread message pump scheduling via the
    /// CefBrowserProcessHandler::OnScheduleMessagePumpWork() callback.
    bool external_message_pump;

    /// The path to the CEF framework directory on macOS.
    ///
    /// If this value is empty then the framework must exist at
    /// "Contents/Frameworks/Chromium Embedded Framework.framework" in the top-level app bundle.
    /// If this value is non-empty then it must be an absolute path. Also configurable using the
    /// "framework-dir-path" command-line switch.
    const char *framework_dir_path;

    /// The path to the main bundle on macOS.
    const char *main_bundle_path;

    /// Set to true (1) to have the browser process message loop run in a separate thread.
    bool multi_threaded_message_loop;

    /// Set to true (1) to disable the use of standard CEF and Chromium command-line parameters to configure the browser
    /// process.
    bool command_line_args_disabled;

    /// To persist session cookies (cookies without an expiry date or validity interval) by default when using the
    /// global cookie manager set this value to true (1).
    bool persist_session_cookies;

    /// Value that will be returned as the User-Agent HTTP header.
    const char *user_agent;

    /// Value that will be inserted as the product portion of the default User-Agent string.
    const char *user_agent_product;

    /// The locale string that will be passed to WebKit.
    const char *locale;

    /// The directory and file name to use for the debug log.
    const char *log_file;

    /// The log severity.
    LogLevel log_severity;

    /// Custom flags that will be used when initializing the V8 JavaScript engine.
    const char *javascript_flags;

    /// The fully qualified path for the resources directory.
    const char *resources_dir_path;

    /// The fully qualified path for the locales directory.
    const char *locales_dir_path;

    /// Background color used for the browser before a document is loaded and when no document color is specified.
    uint32_t background_color;

    /// Specify whether signal handlers must be disabled on POSIX systems.
    bool disable_signal_handlers;
} RuntimeSettings;

typedef struct
{
    void (*on_context_initialized)(void *context);
    void (*on_schedule_message_pump_work)(int64_t delay_ms, void *context);
    void *context;
} RuntimeHandler;

#ifdef LINUX
typedef unsigned long RawWindowHandle;
#else
typedef const void *RawWindowHandle;
#endif

typedef struct
{
    /// window size width.
    uint32_t width;

    /// window size height.
    uint32_t height;

    /// window device scale factor.
    float device_scale_factor;

    /// webview defalt font size.
    int default_font_size;

    /// webview defalt fixed font size.
    int default_fixed_font_size;

    /// The minimum font size.
    int minimum_font_size;

    /// The minimum logical font size.
    int minimum_logical_font_size;

    /// Controls whether WebGL is enabled.
    bool webgl;

    /// Controls whether databases are enabled.
    bool databases;

    /// Controls whether JavaScript can be executed.
    bool javascript;

    /// Controls whether JavaScript can be used to close windows that were not opened via JavaScript.
    bool javascript_close_windows;

    /// Controls whether JavaScript can access the clipboard.
    bool javascript_access_clipboard;

    /// Controls whether DOM pasting is supported in the editor via execCommand("paste").
    bool javascript_dom_paste;

    /// Controls whether local storage can be used.
    bool local_storage;

    /// END values that map to WebPreferences settings.
    uint32_t background_color;

    /// The maximum rate in frames per second (fps) that CefRenderHandler::OnPaint will be called for a
    /// windowless browser.
    uint32_t windowless_frame_rate;

    /// External native window handle.
    RawWindowHandle window_handle;

    /// The request handler factory.
    const RequestHandlerFactory *request_handler_factory;
} WebViewSettings;

typedef enum
{
    WEW_BEFORE_LOAD = 1,
    WEW_LOADED = 2,
    WEW_LOAD_ERROR = 3,
    WEW_REQUEST_CLOSE = 4,
    WEW_CLOSE = 5,
} WebViewState;

typedef struct
{
    bool is_popup;
    const void *buffer;
    uint32_t width;
    uint32_t height;
    uint32_t x;
    uint32_t y;
} Frame;

typedef struct
{
    void (*on_cursor)(CursorType type, void *context);
    void (*on_state_change)(WebViewState state, void *context);
    void (*on_ime_rect)(Rect rect, void *context);
    void (*on_frame)(const Frame *frame, void *context);
    void (*on_title_change)(const char *title, void *context);
    void (*on_fullscreen_change)(bool fullscreen, void *context);
    void (*on_message)(const char *message, void *context);
    void *context;
} WebViewHandler;

///
/// Supported event bit flags.
///
typedef enum
{
    WEW_EVENTFLAG_NONE = 0,
    WEW_EVENTFLAG_CAPS_LOCK_ON = 1 << 0,
    WEW_EVENTFLAG_SHIFT_DOWN = 1 << 1,
    WEW_EVENTFLAG_CONTROL_DOWN = 1 << 2,
    WEW_EVENTFLAG_ALT_DOWN = 1 << 3,
    WEW_EVENTFLAG_LEFT_MOUSE_BUTTON = 1 << 4,
    WEW_EVENTFLAG_MIDDLE_MOUSE_BUTTON = 1 << 5,
    WEW_EVENTFLAG_RIGHT_MOUSE_BUTTON = 1 << 6,
    /// Mac OS-X command key.
    WEW_EVENTFLAG_COMMAND_DOWN = 1 << 7,
    WEW_EVENTFLAG_NUM_LOCK_ON = 1 << 8,
    WEW_EVENTFLAG_IS_KEY_PAD = 1 << 9,
    WEW_EVENTFLAG_IS_LEFT = 1 << 10,
    WEW_EVENTFLAG_IS_RIGHT = 1 << 11,
    WEW_EVENTFLAG_ALTGR_DOWN = 1 << 12,
    WEW_EVENTFLAG_IS_REPEAT = 1 << 13,
    WEW_EVENTFLAG_PRECISION_SCROLLING_DELTA = 1 << 14,
    WEW_EVENTFLAG_SCROLL_BY_PAGE = 1 << 15,
} EventFlags;

typedef struct
{
    ///
    /// X coordinate relative to the left side of the view.
    ///
    int x;

    ///
    /// Y coordinate relative to the top side of the view.
    ///
    int y;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;
} MouseEvent;

///
/// Mouse button types.
///
typedef enum
{
    WEW_MBT_LEFT = 0,
    WEW_MBT_MIDDLE,
    WEW_MBT_RIGHT,
} MouseButton;

///
/// Key event types.
///
typedef enum
{
    ///
    /// Notification that a key transitioned from "up" to "down".
    ///
    WEW_KEYEVENT_RAWKEYDOWN = 0,

    ///
    /// Notification that a key was pressed. This does not necessarily correspond
    /// to a character depending on the key and language. Use KEYEVENT_CHAR for
    /// character input.
    ///
    WEW_KEYEVENT_KEYDOWN,

    ///
    /// Notification that a key was released.
    ///
    WEW_KEYEVENT_KEYUP,

    ///
    /// Notification that a character was typed. Use this for text input. Key
    /// down events may generate 0, 1, or more than one character event depending
    /// on the key, locale, and operating system.
    ///
    WEW_KEYEVENT_CHAR
} KeyEventType;

///
/// Structure representing keyboard event information.
///
typedef struct
{
    ///
    /// The type of keyboard event.
    ///
    KeyEventType type;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;

    ///
    /// The Windows key code for the key event. This value is used by the DOM
    /// specification. Sometimes it comes directly from the event (i.e. on
    /// Windows) and sometimes it's determined using a mapping function. See
    /// WebCore/platform/chromium/KeyboardCodes.h for the list of values.
    ///
    int windows_key_code;

    ///
    /// The actual key code genenerated by the platform.
    ///
    int native_key_code;

    ///
    /// Indicates whether the event is considered a "system key" event (see
    /// http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
    /// This value will always be false on non-Windows platforms.
    ///
    int is_system_key;

    ///
    /// The character generated by the keystroke.
    ///
    uint16_t character;

    ///
    /// Same as |character| but unmodified by any concurrently-held modifiers
    /// (except shift). This is useful for working out shortcut keys.
    ///
    uint16_t unmodified_character;

    ///
    /// True if the focus is currently on an editable field on the page. This is
    /// useful for determining if standard key events should be intercepted.
    ///
    int focus_on_editable_field;
} KeyEvent;

///
/// Touch points states types.
///
typedef enum
{
    WEW_TET_RELEASED = 0,
    WEW_TET_PRESSED,
    WEW_TET_MOVED,
    WEW_TET_CANCELLED
} TouchEventType;

///
/// The device type that caused the event.
///
typedef enum
{
    WEW_POINTER_TYPE_TOUCH = 0,
    WEW_POINTER_TYPE_MOUSE,
    WEW_POINTER_TYPE_PEN,
    WEW_POINTER_TYPE_ERASER,
    WEW_POINTER_TYPE_UNKNOWN
} PointerType;

///
/// Structure representing touch event information.
///
typedef struct
{
    ///
    /// Id of a touch point. Must be unique per touch, can be any number except
    /// -1. Note that a maximum of 16 concurrent touches will be tracked; touches
    /// beyond that will be ignored.
    ///
    int id;

    ///
    /// X coordinate relative to the left side of the view.
    ///
    float x;

    ///
    /// Y coordinate relative to the top side of the view.
    ///
    float y;

    ///
    /// X radius in pixels. Set to 0 if not applicable.
    ///
    float radius_x;

    ///
    /// Y radius in pixels. Set to 0 if not applicable.
    ///
    float radius_y;

    ///
    /// Rotation angle in radians. Set to 0 if not applicable.
    ///
    float rotation_angle;

    ///
    /// The normalized pressure of the pointer input in the range of [0,1].
    /// Set to 0 if not applicable.
    ///
    float pressure;

    ///
    /// The state of the touch point. Touches begin with one CEF_TET_PRESSED event
    /// followed by zero or more CEF_TET_MOVED events and finally one
    /// CEF_TET_RELEASED or CEF_TET_CANCELLED event. Events not respecting this
    /// order will be ignored.
    ///
    TouchEventType type;

    ///
    /// Bit flags describing any pressed modifier keys. See
    /// cef_event_flags_t for values.
    ///
    uint32_t modifiers;

    ///
    /// The device type that caused the event.
    ///
    PointerType pointer_type;

} TouchEvent;

#ifdef __cplusplus
extern "C"
{

#endif

    EXPORT bool post_task_with_main_thread(void (*callback)(void *context), void *context);

    EXPORT int get_exit_code();

    EXPORT int execute_subprocess(int argc, const char **argv);

    EXPORT void run_message_loop();

    EXPORT void quit_message_loop();

    EXPORT void poll_message_loop();

    EXPORT void *create_runtime(const RuntimeSettings *settings, RuntimeHandler handler);

    EXPORT bool execute_runtime(void *runtime, int argc, const char **argv);

    ///
    /// This function should be called on the main application thread to shut down
    /// the CEF browser process before the application exits.
    ///
    EXPORT void close_runtime(void *runtime);

    EXPORT void *create_webview(void *runtime,
                                const char *url,
                                const WebViewSettings *settings,
                                WebViewHandler handler);

    EXPORT void close_webview(void *webview);

    ///
    /// Send a mouse click event to the browser.
    ///
    EXPORT void webview_mouse_click(void *webview, MouseEvent event, MouseButton button, bool pressed);

    ///
    /// Send a mouse wheel event to the browser.
    ///
    EXPORT void webview_mouse_wheel(void *webview, MouseEvent event, int x, int y);

    ///
    /// Send a mouse move event to the browser.
    ///
    EXPORT void webview_mouse_move(void *webview, MouseEvent event);

    ///
    /// Send a key event to the browser.
    ///
    EXPORT void webview_keyboard(void *webview, KeyEvent event);

    ///
    /// Send a touch event to the browser.
    ///
    EXPORT void webview_touch(void *webview, TouchEvent event);

    EXPORT void webview_ime_composition(void *webview, const char *input);

    EXPORT void webview_ime_set_composition(void *webview, const char *input, int x, int y);

    EXPORT void webview_send_message(void *webview, const char *message);

    EXPORT void webview_set_devtools_state(void *webview, bool is_open);

    EXPORT void webview_resize(void *webview, int width, int height);

    EXPORT RawWindowHandle webview_get_window_handle(void *webview);

    EXPORT void webview_set_focus(void *webview, bool enable);

    ///
    /// Cookie management functions
    ///
    EXPORT void *wew_get_global_cookie_manager();
    
    EXPORT void wew_destroy_cookie_manager(void *manager);
    
    EXPORT bool wew_set_cookie(void *manager, const char *url, const Cookie *cookie);
    
    EXPORT bool wew_delete_cookies(void *manager, const char *url, const char *name);
    
    EXPORT void wew_visit_all_cookies(void *manager, CookieVisitor *visitor);
    
    EXPORT void wew_visit_url_cookies(void *manager, const char *url, bool includeHttpOnly, CookieVisitor *visitor);
    
    EXPORT bool wew_flush_cookie_store(void *manager);

#ifdef __cplusplus
}
#endif

#endif /* wew_h */
