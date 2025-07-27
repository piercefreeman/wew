//
//  cookie.cpp
//  webview
//
//  Cookie management functionality for CEF
//

#include "cookie.h"
#include "util.h"
#include "include/cef_app.h"
#include "include/wrapper/cef_helpers.h"
#include <assert.h>
#include <atomic>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <memory>

// Time conversion helpers removed for now
// CEF's time structures are complex and vary between versions

// Thread-safe result holder for async operations
template<typename T>
class AsyncResult {
public:
    void SetResult(T value) {
        std::lock_guard<std::mutex> lock(mutex_);
        result_ = value;
        ready_ = true;
        cv_.notify_all();
    }
    
    T WaitForResult(int timeout_ms = 5000) {
        std::unique_lock<std::mutex> lock(mutex_);
        if (!cv_.wait_for(lock, std::chrono::milliseconds(timeout_ms), 
                         [this] { return ready_; })) {
            // Timeout - return default value
            return T{};
        }
        return result_;
    }
    
private:
    std::mutex mutex_;
    std::condition_variable cv_;
    T result_{};
    bool ready_ = false;
};

// Task for posting cookie operations to IO thread
class CookieTask : public CefTask {
public:
    CookieTask(std::function<void()> func) : func_(func) {}
    
    void Execute() override {
        func_();
    }
    
private:
    std::function<void()> func_;
    IMPLEMENT_REFCOUNTING(CookieTask);
};

ICookieManager::ICookieManager()
{
    _manager = CefCookieManager::GetGlobalManager(nullptr);
    assert(_manager != nullptr);
}

ICookieManager::~ICookieManager()
{
}

bool ICookieManager::SetCookie(const char* url, const Cookie* cookie)
{
    if (!_manager || !url || !cookie || !cookie->name || !cookie->value) {
        return false;
    }

    CefCookie cef_cookie;
    CefString(&cef_cookie.name).FromASCII(cookie->name);
    CefString(&cef_cookie.value).FromASCII(cookie->value);
    
    if (cookie->domain) {
        CefString(&cef_cookie.domain).FromASCII(cookie->domain);
    }
    if (cookie->path) {
        CefString(&cef_cookie.path).FromASCII(cookie->path);
    }
    
    cef_cookie.secure = cookie->secure;
    cef_cookie.httponly = cookie->httponly;
    cef_cookie.has_expires = cookie->has_expires;
    
    // For simplicity, we'll skip time conversion for now
    // CEF's time structures are complex and vary between versions
    // The cookie will still work, just without proper expiration handling
    
    cef_cookie.same_site = static_cast<cef_cookie_same_site_t>(cookie->same_site);
    cef_cookie.priority = static_cast<cef_cookie_priority_t>(cookie->priority);

    CefString cef_url;
    cef_url.FromASCII(url);
    
    // Handle thread safety
    if (CefCurrentlyOn(TID_IO)) {
        return _manager->SetCookie(cef_url, cef_cookie, nullptr);
    } else {
        auto result = std::make_shared<AsyncResult<bool>>();
        
        CefPostTask(TID_IO, new CookieTask([this, cef_url, cef_cookie, result]() {
            bool success = _manager->SetCookie(cef_url, cef_cookie, nullptr);
            result->SetResult(success);
        }));
        
        return result->WaitForResult();
    }
}

bool ICookieManager::DeleteCookies(const char* url, const char* name)
{
    if (!_manager) {
        return false;
    }

    CefString cef_url;
    if (url) {
        cef_url.FromASCII(url);
    }
    
    CefString cef_name;
    if (name) {
        cef_name.FromASCII(name);
    }

    // Handle thread safety
    if (CefCurrentlyOn(TID_IO)) {
        return _manager->DeleteCookies(cef_url, cef_name, nullptr);
    } else {
        auto result = std::make_shared<AsyncResult<bool>>();
        
        CefPostTask(TID_IO, new CookieTask([this, cef_url, cef_name, result]() {
            bool success = _manager->DeleteCookies(cef_url, cef_name, nullptr);
            result->SetResult(success);
        }));
        
        return result->WaitForResult();
    }
}

void ICookieManager::VisitAllCookies(CookieVisitor* visitor)
{
    if (!_manager || !visitor) {
        return;
    }

    CefRefPtr<ICookieVisitor> cef_visitor = new ICookieVisitor(visitor);
    
    if (CefCurrentlyOn(TID_IO)) {
        _manager->VisitAllCookies(cef_visitor);
    } else {
        CefPostTask(TID_IO, new CookieTask([this, cef_visitor]() {
            _manager->VisitAllCookies(cef_visitor);
        }));
    }
}

void ICookieManager::VisitUrlCookies(const char* url, bool includeHttpOnly, CookieVisitor* visitor)
{
    if (!_manager || !url || !visitor) {
        return;
    }

    CefString cef_url;
    cef_url.FromASCII(url);
    
    CefRefPtr<ICookieVisitor> cef_visitor = new ICookieVisitor(visitor);
    
    if (CefCurrentlyOn(TID_IO)) {
        _manager->VisitUrlCookies(cef_url, includeHttpOnly, cef_visitor);
    } else {
        CefPostTask(TID_IO, new CookieTask([this, cef_url, includeHttpOnly, cef_visitor]() {
            _manager->VisitUrlCookies(cef_url, includeHttpOnly, cef_visitor);
        }));
    }
}

bool ICookieManager::FlushStore()
{
    if (!_manager) {
        return false;
    }

    // Handle thread safety
    if (CefCurrentlyOn(TID_IO)) {
        return _manager->FlushStore(nullptr);
    } else {
        auto result = std::make_shared<AsyncResult<bool>>();
        
        CefPostTask(TID_IO, new CookieTask([this, result]() {
            bool success = _manager->FlushStore(nullptr);
            result->SetResult(success);
        }));
        
        return result->WaitForResult();
    }
}

// ICookieVisitor implementation
ICookieVisitor::ICookieVisitor(CookieVisitor* visitor) : _visitor(visitor)
{
    assert(visitor != nullptr);
}

ICookieVisitor::~ICookieVisitor()
{
    if (_visitor && _visitor->destroy) {
        _visitor->destroy(_visitor->context);
    }
}

bool ICookieVisitor::Visit(const CefCookie& cef_cookie, int count, int total, bool& deleteCookie)
{
    if (!_visitor || !_visitor->visit) {
        return false;
    }

    // Create a thread-safe cookie data holder
    struct CookieData {
        std::string name;
        std::string value;
        std::string domain;
        std::string path;
        Cookie cookie;
    };
    
    auto data = std::make_unique<CookieData>();
    
    // Convert strings safely
    data->name = CefString(&cef_cookie.name).ToString();
    data->value = CefString(&cef_cookie.value).ToString();
    data->domain = CefString(&cef_cookie.domain).ToString();
    data->path = CefString(&cef_cookie.path).ToString();
    
    // Fill cookie structure
    data->cookie.name = data->name.c_str();
    data->cookie.value = data->value.c_str();
    data->cookie.domain = data->domain.c_str();
    data->cookie.path = data->path.c_str();
    data->cookie.secure = cef_cookie.secure;
    data->cookie.httponly = cef_cookie.httponly;
    data->cookie.has_expires = cef_cookie.has_expires;
    
    // Convert times - simplified for now
    // CEF's cookie time structure varies between versions
    // For maximum compatibility, we'll use 0 for times in the visitor
    data->cookie.expires = 0;
    data->cookie.creation = 0;
    data->cookie.last_access = 0;
    
    data->cookie.same_site = static_cast<int>(cef_cookie.same_site);
    data->cookie.priority = static_cast<int>(cef_cookie.priority);
    
    bool delete_cookie = false;
    bool continue_visiting = _visitor->visit(&data->cookie, count, total, &delete_cookie, _visitor->context);
    deleteCookie = delete_cookie;
    
    return continue_visiting;
}

// C API wrapper functions
extern "C" {

EXPORT void* wew_get_global_cookie_manager()
{
    return new ICookieManager();
}

EXPORT void wew_destroy_cookie_manager(void* manager)
{
    if (manager) {
        delete static_cast<ICookieManager*>(manager);
    }
}

EXPORT bool wew_set_cookie(void* manager, const char* url, const Cookie* cookie)
{
    if (!manager) {
        return false;
    }
    
    return static_cast<ICookieManager*>(manager)->SetCookie(url, cookie);
}

EXPORT bool wew_delete_cookies(void* manager, const char* url, const char* name)
{
    if (!manager) {
        return false;
    }
    
    return static_cast<ICookieManager*>(manager)->DeleteCookies(url, name);
}

EXPORT void wew_visit_all_cookies(void* manager, CookieVisitor* visitor)
{
    if (manager) {
        static_cast<ICookieManager*>(manager)->VisitAllCookies(visitor);
    }
}

EXPORT void wew_visit_url_cookies(void* manager, const char* url, bool includeHttpOnly, CookieVisitor* visitor)
{
    if (manager) {
        static_cast<ICookieManager*>(manager)->VisitUrlCookies(url, includeHttpOnly, visitor);
    }
}

EXPORT bool wew_flush_cookie_store(void* manager)
{
    if (!manager) {
        return false;
    }
    
    return static_cast<ICookieManager*>(manager)->FlushStore();
}

} // extern "C"