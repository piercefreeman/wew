//
//  cookie.h
//  webview
//
//  Cookie management functionality for CEF
//

#ifndef cookie_h
#define cookie_h
#pragma once

#include <string>
#include <functional>

#include "include/cef_cookie.h"
#include "wew.h"

class ICookieManager
{
  public:
    ICookieManager();
    ~ICookieManager();

    ///
    /// Set a cookie for the specified URL
    ///
    bool SetCookie(const char* url, const Cookie* cookie);

    ///
    /// Delete cookies matching the specified URL and name
    /// If name is nullptr, all cookies for the URL are deleted
    ///
    bool DeleteCookies(const char* url, const char* name);

    ///
    /// Visit all cookies. The visitor will be called for each cookie.
    ///
    void VisitAllCookies(CookieVisitor* visitor);

    ///
    /// Visit cookies for a specific URL
    ///
    void VisitUrlCookies(const char* url, bool includeHttpOnly, CookieVisitor* visitor);

    ///
    /// Flush the backing store (if any) to disk
    ///
    bool FlushStore();

  private:
    CefRefPtr<CefCookieManager> _manager;
};

///
/// Cookie visitor callback wrapper
///
class ICookieVisitor : public CefCookieVisitor
{
  public:
    ICookieVisitor(CookieVisitor* visitor);
    ~ICookieVisitor();

    ///
    /// Called for each cookie. Return false to stop visiting cookies.
    ///
    bool Visit(const CefCookie& cookie, int count, int total, bool& deleteCookie) override;

  private:
    CookieVisitor* _visitor;

    IMPLEMENT_REFCOUNTING(ICookieVisitor);
    DISALLOW_COPY_AND_ASSIGN(ICookieVisitor);
};

#endif /* cookie_h */