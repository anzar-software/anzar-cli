use super::shared::{Helpers, InvalidTestCases};

use anzar::config::AuthStrategy;

use std::collections::HashMap;

#[derive(Debug)]
struct Cookie {
    id: Option<String>,
    _path: Option<String>,
    same_site: Option<String>,
    http_only: bool,
    secure: bool,
    _extras: HashMap<String, Option<String>>,
}

impl Cookie {
    fn parse(raw: &str) -> Result<Self, &'static str> {
        let mut parts = raw.split(';');

        // First segment is ALWAYS the name=value pair
        let session_id = parts.next().ok_or("empty cookie string")?;
        // let (name, value) = name_value
        //     .split_once('=')
        //     .ok_or("missing '=' in cookie name=value")?;

        let mut path = None;
        let mut same_site = None;
        let mut http_only = false;
        let mut secure = false;
        let mut extras = HashMap::new();

        for part in parts {
            let part = part.trim();
            match part.split_once('=') {
                Some((k, v)) => match k.trim().to_lowercase().as_str() {
                    "path" => path = Some(v.trim().to_string()),
                    "samesite" => same_site = Some(v.trim().to_string()),
                    _ => {
                        extras.insert(k.trim().to_string(), Some(v.trim().to_string()));
                    }
                },
                None => match part.to_lowercase().as_str() {
                    "httponly" => http_only = true,
                    "secure" => secure = true,
                    _ => {
                        extras.insert(part.to_string(), None);
                    }
                },
            }
        }

        Ok(Cookie {
            id: Some(session_id.to_string()),
            _path: path,
            same_site,
            http_only,
            secure,
            _extras: extras,
        })
    }
}

#[actix_web::test]
async fn test_session_cookie_attributes() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let cookie_header = response.headers().get("set-cookie");
    assert!(cookie_header.is_some());

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Session(..))
        && let Some(raw) = cookie_header
    {
        let cookie = Cookie::parse(raw.to_str().unwrap()).unwrap();
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert!(cookie.id.is_some());
        assert!(cookie.same_site.is_some_and(|v| v == "Strict"));
    }
}

#[actix_web::test]
async fn test_protected_route_with_valid_session() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let cookie_header = response.headers().get("set-cookie");
    assert!(cookie_header.is_some());

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Session(..))
        && let Some(raw) = cookie_header
    {
        let cookie = Cookie::parse(raw.to_str().unwrap()).unwrap();
        assert!(cookie.id.is_some());

        let response = test_app.user(&cookie.id.unwrap(), strategy).await;
        assert!(response.status().is_success());
    }
}

#[actix_web::test]
async fn test_protected_route_with_invalid_session() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let cookie_header = response.headers().get("set-cookie");
    assert!(cookie_header.is_some());

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Session(..))
        && let Some(raw) = cookie_header
    {
        let cookie = Cookie::parse(raw.to_str().unwrap()).unwrap();
        assert!(cookie.id.is_some());

        for (token, err_msg, status_code) in InvalidTestCases::session_cookies() {
            let response = test_app.user(&token, strategy).await;

            assert_eq!(
                status_code,
                response.status().as_u16(),
                "The API did not fail when the payload was: {}",
                err_msg
            );
        }
    }
}

#[actix_web::test]
async fn test_session_cookie_not_reusable_after_logout() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    // extract cookie
    let cookie_header = response.headers().get("set-cookie");
    assert!(cookie_header.is_some());

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Session(..))
        && let Some(raw) = cookie_header
    {
        let cookie = Cookie::parse(raw.to_str().unwrap()).unwrap();
        assert!(cookie.id.is_some());
        let cookie_id = cookie.id.unwrap();

        // Find user
        let response = test_app.user(&cookie_id, strategy).await;
        assert!(response.status().is_success());

        // Logout
        let response = test_app.session_logout(&cookie_id).await;
        assert!(response.status().is_success());

        let response = test_app.user(&cookie_id, strategy).await;
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when refreshToken was used instead of accessToken",
        );
    }
}
