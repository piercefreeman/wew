//
//  webview.cpp
//  webview
//
//  Created by Mr.Panda on 2023/4/26.
//

#include "webview.h"
#include "app.h"

#ifdef MACOS
#include "include/wrapper/cef_library_loader.h"
#include "include/cef_sandbox_mac.h"
#endif

typedef struct
{
    CefRefPtr<IBrowser> ref;
} Browser;

CefMainArgs get_main_args(int argc, const char** argv)
{
#ifdef WIN32
    CefMainArgs main_args(::GetModuleHandleW(nullptr));
#else
    CefMainArgs main_args(argc, (char**)argv);
#endif

    return main_args;
}

void run_message_loop() {
    CefRunMessageLoop();
}

void quit_message_loop() {
    CefQuitMessageLoop();
}

void poll_message_loop() {
    CefDoMessageLoopWork();
}

void execute_sub_process(int argc, const char** argv)
{
#ifdef MACOS
    CefScopedSandboxContext sandbox_context;
    if (!sandbox_context.Initialize(argc, (char**)argv))
    {
        return;
    }

    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInHelper()) 
    {
        return;
    }
#endif
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, new IRenderApp, nullptr);
}

void* create_app(const WebviewOptions* settings, CreateWebviewCallback callback, void* ctx)
{
#ifdef MACOS
    CefScopedLibraryLoader library_loader;
    if (!library_loader.LoadInMain()) 
    {
        return nullptr;
    }
#endif

    assert(settings != nullptr);
    assert(callback != nullptr);

    App* app = new App;
    app->ref = new IApp(settings, callback, ctx);
    return app;
}

void start_app(void* app_ptr, int argc, const char** argv) {
    assert(app_ptr != nullptr);

    auto app = (App*)app_ptr;
    auto main_args = get_main_args(argc, argv);
    CefExecuteProcess(main_args, app->ref, nullptr);
    CefInitialize(main_args, app->ref->cef_settings, app->ref, nullptr);
}

void close_app(void* app_ptr)
{
    assert(app_ptr != nullptr);

    CefShutdown();

    auto app = (App*)app_ptr;
    delete app;
}

void* create_page(void* app_ptr,
                  const char* url,
                  const PageOptions* settings,
                  PageObserver observer,
                  void* ctx)
{
    assert(app_ptr != nullptr);
    assert(settings != nullptr);

    auto app = (App*)app_ptr;

    Browser* browser = new Browser;
    browser->ref = app->ref->CreateBrowser(std::string(url), settings, observer, ctx);
    return browser;
}

void close_page(void* browser)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->IClose();
    delete page;
}

void page_send_mouse_click(void* browser, MouseButtons button, bool pressed)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnMouseClick(button, pressed);
}

void page_send_mouse_click_with_pos(void* browser,
                                    MouseButtons button,
                                    bool pressed,
                                    int x,
                                    int y)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnMouseClickWithPosition(button, x, y, pressed);
}

void page_send_mouse_wheel(void* browser, int x, int y)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnMouseWheel(x, y);
}

void page_send_mouse_move(void* browser, int x, int y)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnMouseMove(x, y);
}

void page_send_keyboard(void* browser, int scan_code, bool pressed, Modifiers modifiers)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnKeyboard(scan_code, pressed, modifiers);
}

void page_send_touch(void* browser,
                     int id,
                     int x,
                     int y,
                     TouchEventType type,
                     TouchPointerType pointer_type)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    // TouchEventType have the same value with cef_touch_event_type_t.
    // Same as TouchPointerType.
    page->ref->OnTouch(id, x, y, (cef_touch_event_type_t)type, (cef_pointer_type_t)pointer_type);
}

void page_send_message(void* browser, const char* message)
{
    assert(browser != nullptr);
    assert(message != nullptr);

    auto page = (Browser*)browser;

    page->ref->ISendMessage(std::string(message));
}

void page_set_devtools_state(void* browser, bool is_open)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->SetDevToolsOpenState(is_open);
}

void page_resize(void* browser, int width, int height)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    page->ref->Resize(width, height);
}

const void* page_get_hwnd(void* browser)
{
    assert(browser != nullptr);

    auto page = (Browser*)browser;

    auto hwnd = page->ref->GetHWND();
    return (void*)hwnd;
}

void page_send_ime_composition(void* browser, const char* input)
{
    assert(browser != nullptr);
    assert(input != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnIMEComposition(std::string(input));
}

void page_send_ime_set_composition(void* browser, const char* input, int x, int y)
{
    assert(browser != nullptr);
    assert(input != nullptr);

    auto page = (Browser*)browser;

    page->ref->OnIMESetComposition(std::string(input), x, y);
}
