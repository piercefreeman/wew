//! Comprehensive tests for cookie functionality

#[cfg(test)]
mod tests {
    use wew::cookie::{Cookie, SameSite, Priority, CookieError};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie {
            name: "test_cookie".to_string(),
            value: "test_value".to_string(),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            secure: true,
            httponly: true,
            expires: Some(1234567890),
            same_site: SameSite::Strict,
            priority: Priority::High,
        };

        assert_eq!(cookie.name, "test_cookie");
        assert_eq!(cookie.value, "test_value");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.secure);
        assert!(cookie.httponly);
        assert_eq!(cookie.expires, Some(1234567890));
        assert_eq!(cookie.same_site, SameSite::Strict);
        assert_eq!(cookie.priority, Priority::High);
    }

    #[test]
    fn test_cookie_default() {
        let cookie = Cookie::default();
        
        assert_eq!(cookie.name, "");
        assert_eq!(cookie.value, "");
        assert_eq!(cookie.domain, None);
        assert_eq!(cookie.path, None);
        assert!(!cookie.secure);
        assert!(!cookie.httponly);
        assert_eq!(cookie.expires, None);
        assert_eq!(cookie.same_site, SameSite::Unspecified);
        assert_eq!(cookie.priority, Priority::Medium);
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("test", "value123")
            .domain("example.com")
            .path("/api")
            .secure(true)
            .httponly(true)
            .same_site(SameSite::Lax)
            .priority(Priority::Low);
        
        assert_eq!(cookie.name, "test");
        assert_eq!(cookie.value, "value123");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/api".to_string()));
        assert!(cookie.secure);
        assert!(cookie.httponly);
        assert_eq!(cookie.same_site, SameSite::Lax);
        assert_eq!(cookie.priority, Priority::Low);
    }

    #[test]
    fn test_cookie_expiration_timestamp() {
        let timestamp = 1700000000;
        let cookie = Cookie::new("test", "value")
            .expires_at(timestamp);
        
        assert_eq!(cookie.expires, Some(timestamp));
    }

    #[test]
    fn test_cookie_expiration_duration() {
        let duration = Duration::from_secs(3600); // 1 hour
        let cookie = Cookie::new("test", "value")
            .expires_in(duration);
        
        assert!(cookie.expires.is_some());
        
        // Verify the expiration is approximately 1 hour from now
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let expires = cookie.expires.unwrap();
        let diff = expires - now;
        
        // Allow 2 seconds of variance for test execution time
        assert!(diff >= 3598 && diff <= 3602, 
                "Expected expiration ~3600s from now, got {}s", diff);
    }

    #[test]
    fn test_same_site_values() {
        assert_eq!(SameSite::Unspecified as i32, 0);
        assert_eq!(SameSite::NoRestriction as i32, 1);
        assert_eq!(SameSite::Lax as i32, 2);
        assert_eq!(SameSite::Strict as i32, 3);
    }

    #[test]
    fn test_priority_values() {
        assert_eq!(Priority::Low as i32, 0);
        assert_eq!(Priority::Medium as i32, 1);
        assert_eq!(Priority::High as i32, 2);
    }

    #[test]
    fn test_cookie_error_display() {
        let errors = vec![
            (CookieError::InvalidUrl, "Invalid URL format"),
            (CookieError::InvalidCookieName, "Invalid cookie name"),
            (CookieError::InvalidCookieValue, "Invalid cookie value"),
            (CookieError::InvalidDomain, "Invalid domain"),
            (CookieError::InvalidPath, "Invalid path"),
            (CookieError::SetCookieFailed, "Failed to set cookie"),
            (CookieError::DeleteCookieFailed, "Failed to delete cookie"),
            (CookieError::FlushStoreFailed, "Failed to flush cookie store"),
        ];

        for (error, expected_msg) in errors {
            assert_eq!(error.to_string(), expected_msg);
        }
    }

    #[test]
    fn test_cookie_builder_chaining() {
        // Test that all builder methods can be chained
        let cookie = Cookie::new("chain", "test")
            .domain("example.com")
            .path("/test")
            .secure(false)
            .httponly(false)
            .expires_at(1234567890)
            .same_site(SameSite::NoRestriction)
            .priority(Priority::High)
            .secure(true) // Override previous value
            .httponly(true); // Override previous value
        
        assert_eq!(cookie.name, "chain");
        assert_eq!(cookie.value, "test");
        assert!(cookie.secure);
        assert!(cookie.httponly);
        assert_eq!(cookie.expires, Some(1234567890));
    }

    #[test]
    fn test_cookie_with_empty_strings() {
        // Test that cookies can be created with empty strings
        let cookie = Cookie::new("", "");
        assert_eq!(cookie.name, "");
        assert_eq!(cookie.value, "");
    }

    #[test]
    fn test_cookie_clone() {
        let original = Cookie::new("test", "value")
            .domain("example.com")
            .secure(true);
        
        let cloned = original.clone();
        
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.value, original.value);
        assert_eq!(cloned.domain, original.domain);
        assert_eq!(cloned.secure, original.secure);
    }

    #[test]
    fn test_cookie_debug_format() {
        let cookie = Cookie::new("debug", "test");
        let debug_str = format!("{:?}", cookie);
        
        assert!(debug_str.contains("Cookie"));
        assert!(debug_str.contains("debug"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(CookieError::InvalidUrl, CookieError::InvalidUrl);
        assert_ne!(CookieError::InvalidUrl, CookieError::InvalidCookieName);
    }

    #[test]
    fn test_multiple_domain_cookies() {
        // Test creating cookies for different domains
        let cookies = vec![
            Cookie::new("google", "value1").domain("google.com"),
            Cookie::new("github", "value2").domain("github.com"),
            Cookie::new("stackoverflow", "value3").domain("stackoverflow.com"),
        ];
        
        for (i, cookie) in cookies.iter().enumerate() {
            match i {
                0 => assert_eq!(cookie.domain, Some("google.com".to_string())),
                1 => assert_eq!(cookie.domain, Some("github.com".to_string())),
                2 => assert_eq!(cookie.domain, Some("stackoverflow.com".to_string())),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_session_vs_persistent_cookies() {
        let session_cookie = Cookie::new("session", "value");
        assert_eq!(session_cookie.expires, None);
        
        let persistent_cookie = Cookie::new("persistent", "value")
            .expires_in(Duration::from_secs(86400));
        assert!(persistent_cookie.expires.is_some());
    }

    #[test]
    fn test_cookie_attributes_combinations() {
        // Test various combinations of cookie attributes
        let combinations = vec![
            (true, true, SameSite::Strict),
            (true, false, SameSite::Lax),
            (false, true, SameSite::NoRestriction),
            (false, false, SameSite::Unspecified),
        ];
        
        for (secure, httponly, same_site) in combinations {
            let cookie = Cookie::new("test", "value")
                .secure(secure)
                .httponly(httponly)
                .same_site(same_site);
            
            assert_eq!(cookie.secure, secure);
            assert_eq!(cookie.httponly, httponly);
            assert_eq!(cookie.same_site, same_site);
        }
    }
}