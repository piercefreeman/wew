//! Cookie management functionality for browser sessions.
//!
//! This module provides cookie management capabilities including setting, deleting,
//! and visiting cookies programmatically. All cookie operations are thread-safe and
//! properly handle CEF's IO thread requirements.
//!
//! ## Thread Safety
//!
//! All cookie operations in CEF must run on the IO thread. This module handles
//! thread posting automatically, with synchronous waiting for results where needed.
//! Cookie visitor callbacks are executed on the IO thread and must be thread-safe.
//!
//! ## Example
//!
//! ```no_run
//! use wew::cookie::{Cookie, CookieManager, SameSite, Priority};
//!
//! // Get the global cookie manager
//! let manager = CookieManager::global();
//!
//! // Set a cookie
//! let cookie = Cookie {
//!     name: "session_id".to_string(),
//!     value: "abc123".to_string(),
//!     domain: Some("example.com".to_string()),
//!     path: Some("/".to_string()),
//!     secure: true,
//!     httponly: true,
//!     expires: None,
//!     same_site: SameSite::Lax,
//!     priority: Priority::Medium,
//! };
//!
//! manager.set_cookie("https://example.com", &cookie).unwrap();
//!
//! // Visit all cookies
//! manager.visit_all_cookies(|cookie| {
//!     println!("Cookie: {} = {}", cookie.name, cookie.value);
//!     true // Continue visiting
//! });
//!
//! // Delete a specific cookie
//! manager.delete_cookie("https://example.com", Some("session_id")).unwrap();
//! ```

use std::{
    ffi::{CString, c_void, c_char, c_int},
    ptr::null_mut,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    sys,
    utils::ThreadSafePointer,
};

/// Cookie same-site attribute values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// No SameSite attribute specified
    Unspecified = 0,
    /// Cookies will be sent in all contexts
    NoRestriction = 1,
    /// Cookies are not sent on normal cross-site subrequests
    Lax = 2,
    /// Cookies will only be sent in a first-party context
    Strict = 3,
}

/// Cookie priority values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Low priority
    Low = 0,
    /// Medium priority (default)
    Medium = 1,
    /// High priority
    High = 2,
}

/// Represents an HTTP cookie
#[derive(Debug, Clone)]
pub struct Cookie {
    /// The cookie name
    pub name: String,
    /// The cookie value
    pub value: String,
    /// The domain for which the cookie is valid
    pub domain: Option<String>,
    /// The path for which the cookie is valid
    pub path: Option<String>,
    /// If true, the cookie will only be sent over secure connections
    pub secure: bool,
    /// If true, the cookie will be inaccessible to JavaScript
    pub httponly: bool,
    /// Cookie expiration time in seconds since epoch. None means session cookie.
    pub expires: Option<i64>,
    /// SameSite attribute
    pub same_site: SameSite,
    /// Cookie priority
    pub priority: Priority,
}

impl Default for Cookie {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            domain: None,
            path: None,
            secure: false,
            httponly: false,
            expires: None,
            same_site: SameSite::Unspecified,
            priority: Priority::Medium,
        }
    }
}

impl Cookie {
    /// Create a new cookie with the given name and value
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            ..Default::default()
        }
    }
    
    /// Set the domain for this cookie
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }
    
    /// Set the path for this cookie
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    /// Set whether this cookie should only be sent over secure connections
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    
    /// Set whether this cookie should be inaccessible to JavaScript
    pub fn httponly(mut self, httponly: bool) -> Self {
        self.httponly = httponly;
        self
    }
    
    /// Set the expiration time for this cookie (seconds since epoch)
    pub fn expires_at(mut self, timestamp: i64) -> Self {
        self.expires = Some(timestamp);
        self
    }
    
    /// Set the cookie to expire after a duration from now
    pub fn expires_in(mut self, duration: std::time::Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.expires = Some(now + duration.as_secs() as i64);
        self
    }
    
    /// Set the SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }
    
    /// Set the priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

/// Cookie visitor callback wrapper
struct CookieVisitorWrapper<F> {
    callback: Arc<Mutex<F>>,
}

impl<F> CookieVisitorWrapper<F>
where
    F: FnMut(&Cookie) -> bool,
{
    fn new(callback: F) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
        }
    }
}

/// FFI callback for cookie visitor
unsafe extern "C" fn cookie_visitor_visit<F>(
    cookie: *const sys::Cookie,
    _count: c_int,
    _total: c_int,
    _delete_cookie: *mut bool,
    context: *mut c_void,
) -> bool
where
    F: FnMut(&Cookie) -> bool,
{
    // SAFETY: We check for null pointers before dereferencing
    unsafe {
        if cookie.is_null() || context.is_null() {
            return false;
        }
        
        let wrapper = &*(context as *const CookieVisitorWrapper<F>);
        let cookie = &*cookie;
        
        // Convert C cookie to Rust cookie
        let rust_cookie = Cookie {
            name: c_str_to_string(cookie.name),
            value: c_str_to_string(cookie.value),
            domain: if cookie.domain.is_null() { None } else { Some(c_str_to_string(cookie.domain)) },
            path: if cookie.path.is_null() { None } else { Some(c_str_to_string(cookie.path)) },
            secure: cookie.secure,
            httponly: cookie.httponly,
            expires: if cookie.has_expires { Some(cookie.expires) } else { None },
            same_site: match cookie.same_site {
                0 => SameSite::Unspecified,
                1 => SameSite::NoRestriction,
                2 => SameSite::Lax,
                3 => SameSite::Strict,
                _ => SameSite::Unspecified,
            },
            priority: match cookie.priority {
                0 => Priority::Low,
                1 => Priority::Medium,
                2 => Priority::High,
                _ => Priority::Medium,
            },
        };
        
        // Call the user's callback
        if let Ok(mut callback) = wrapper.callback.lock() {
            callback(&rust_cookie)
        } else {
            false
        }
    }
}

/// FFI callback for cookie visitor destruction
unsafe extern "C" fn cookie_visitor_destroy<F>(context: *mut c_void) {
    // SAFETY: We check for null before converting back to Box
    unsafe {
        if !context.is_null() {
            // Drop the wrapper
            let _ = Box::from_raw(context as *mut CookieVisitorWrapper<F>);
        }
    }
}

/// Helper to convert C string to Rust String
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    // SAFETY: We check for null before creating CStr
    unsafe {
        if ptr.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Cookie manager for managing browser cookies
pub struct CookieManager {
    inner: ThreadSafePointer<c_void>,
}

impl CookieManager {
    /// Get the global cookie manager instance
    pub fn global() -> Self {
        unsafe {
            let ptr = sys::wew_get_global_cookie_manager();
            Self {
                inner: ThreadSafePointer::new(ptr),
            }
        }
    }

    /// Set a cookie for the specified URL
    pub fn set_cookie(&self, url: &str, cookie: &Cookie) -> Result<(), CookieError> {
        let c_url = CString::new(url).map_err(|_| CookieError::InvalidUrl)?;
        let c_name = CString::new(cookie.name.as_str()).map_err(|_| CookieError::InvalidCookieName)?;
        let c_value = CString::new(cookie.value.as_str()).map_err(|_| CookieError::InvalidCookieValue)?;
        
        let c_domain = cookie.domain.as_ref()
            .map(|d| CString::new(d.as_str()))
            .transpose()
            .map_err(|_| CookieError::InvalidDomain)?;
        
        let c_path = cookie.path.as_ref()
            .map(|p| CString::new(p.as_str()))
            .transpose()
            .map_err(|_| CookieError::InvalidPath)?;

        // Get current time for creation/last_access if not specified
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let sys_cookie = sys::Cookie {
            name: c_name.as_ptr(),
            value: c_value.as_ptr(),
            domain: c_domain.as_ref().map(|d| d.as_ptr()).unwrap_or(null_mut()),
            path: c_path.as_ref().map(|p| p.as_ptr()).unwrap_or(null_mut()),
            secure: cookie.secure,
            httponly: cookie.httponly,
            expires: cookie.expires.unwrap_or(0),
            has_expires: cookie.expires.is_some(),
            creation: now,
            last_access: now,
            same_site: cookie.same_site as i32,
            priority: cookie.priority as i32,
        };

        unsafe {
            let result = sys::wew_set_cookie(
                self.inner.as_ptr(),
                c_url.as_ptr(),
                &sys_cookie,
            );
            
            if result {
                Ok(())
            } else {
                Err(CookieError::SetCookieFailed)
            }
        }
    }

    /// Delete cookies for the specified URL and optional name
    /// If name is None, all cookies for the URL are deleted
    pub fn delete_cookie(&self, url: &str, name: Option<&str>) -> Result<(), CookieError> {
        let c_url = CString::new(url).map_err(|_| CookieError::InvalidUrl)?;
        let c_name = name
            .map(|n| CString::new(n))
            .transpose()
            .map_err(|_| CookieError::InvalidCookieName)?;

        unsafe {
            let result = sys::wew_delete_cookies(
                self.inner.as_ptr(),
                c_url.as_ptr(),
                c_name.as_ref().map(|n| n.as_ptr()).unwrap_or(null_mut()),
            );
            
            if result {
                Ok(())
            } else {
                Err(CookieError::DeleteCookieFailed)
            }
        }
    }

    /// Flush the cookie store to disk
    pub fn flush_store(&self) -> Result<(), CookieError> {
        unsafe {
            let result = sys::wew_flush_cookie_store(self.inner.as_ptr());
            if result {
                Ok(())
            } else {
                Err(CookieError::FlushStoreFailed)
            }
        }
    }

    /// Visit all cookies with a callback
    /// The callback receives each cookie and should return true to continue visiting
    pub fn visit_all_cookies<F>(&self, callback: F)
    where
        F: FnMut(&Cookie) -> bool + Send + 'static,
    {
        let wrapper = Box::new(CookieVisitorWrapper::new(callback));
        let wrapper_ptr = Box::into_raw(wrapper);
        
        let visitor = sys::CookieVisitor {
            visit: Some(cookie_visitor_visit::<F>),
            destroy: Some(cookie_visitor_destroy::<F>),
            context: wrapper_ptr as *mut c_void,
        };
        
        unsafe {
            sys::wew_visit_all_cookies(self.inner.as_ptr(), &visitor as *const _ as *mut _);
        }
    }

    /// Visit cookies for a specific URL
    /// The callback receives each cookie and should return true to continue visiting
    pub fn visit_url_cookies<F>(&self, url: &str, include_http_only: bool, callback: F)
    where
        F: FnMut(&Cookie) -> bool + Send + 'static,
    {
        let c_url = match CString::new(url) {
            Ok(url) => url,
            Err(_) => return,
        };
        
        let wrapper = Box::new(CookieVisitorWrapper::new(callback));
        let wrapper_ptr = Box::into_raw(wrapper);
        
        let visitor = sys::CookieVisitor {
            visit: Some(cookie_visitor_visit::<F>),
            destroy: Some(cookie_visitor_destroy::<F>),
            context: wrapper_ptr as *mut c_void,
        };
        
        unsafe {
            sys::wew_visit_url_cookies(
                self.inner.as_ptr(),
                c_url.as_ptr(),
                include_http_only,
                &visitor as *const _ as *mut _,
            );
        }
    }
}

impl Drop for CookieManager {
    fn drop(&mut self) {
        unsafe {
            sys::wew_destroy_cookie_manager(self.inner.as_ptr());
        }
    }
}

unsafe impl Send for CookieManager {}
unsafe impl Sync for CookieManager {}

/// Errors that can occur during cookie operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieError {
    /// Invalid URL format
    InvalidUrl,
    /// Invalid cookie name
    InvalidCookieName,
    /// Invalid cookie value
    InvalidCookieValue,
    /// Invalid domain
    InvalidDomain,
    /// Invalid path
    InvalidPath,
    /// Failed to set cookie
    SetCookieFailed,
    /// Failed to delete cookie
    DeleteCookieFailed,
    /// Failed to flush cookie store
    FlushStoreFailed,
}

impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieError::InvalidUrl => write!(f, "Invalid URL format"),
            CookieError::InvalidCookieName => write!(f, "Invalid cookie name"),
            CookieError::InvalidCookieValue => write!(f, "Invalid cookie value"),
            CookieError::InvalidDomain => write!(f, "Invalid domain"),
            CookieError::InvalidPath => write!(f, "Invalid path"),
            CookieError::SetCookieFailed => write!(f, "Failed to set cookie"),
            CookieError::DeleteCookieFailed => write!(f, "Failed to delete cookie"),
            CookieError::FlushStoreFailed => write!(f, "Failed to flush cookie store"),
        }
    }
}

impl std::error::Error for CookieError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .domain("example.com")
            .path("/")
            .secure(true)
            .httponly(true)
            .same_site(SameSite::Strict)
            .priority(Priority::High);
        
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.secure);
        assert!(cookie.httponly);
        assert_eq!(cookie.same_site, SameSite::Strict);
        assert_eq!(cookie.priority, Priority::High);
    }
    
    #[test]
    fn test_cookie_expiration() {
        use std::time::Duration;
        
        let cookie = Cookie::new("test", "value")
            .expires_in(Duration::from_secs(3600));
        
        assert!(cookie.expires.is_some());
        
        // Check that expiration is roughly 1 hour from now
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let expires = cookie.expires.unwrap();
        assert!(expires >= now + 3599 && expires <= now + 3601);
    }
}