use super::shared::{Helpers, InvalidTestCases, RefreshTokenRequest};
use anzar::{config::AuthStrategy, scopes::auth::AuthResponse};

#[actix_web::test]
async fn test_refresh_token_success() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        let refresh_token: &str = &tokens.refresh;
        assert!(!refresh_token.is_empty());
        // refresh access token
        let body = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        };
        let response = test_app.refresh(&body).await;
        assert!(response.status().is_success());

        let auth_response: AuthResponse = response.json().await.unwrap();
        if let Some(tokens) = &auth_response.tokens {
            let access_token: &str = &tokens.access;
            let refresh_token: &str = &tokens.refresh;

            // assert tokens are not empty
            assert!(!access_token.is_empty() && !refresh_token.is_empty());

            let access_token_claims = Helpers::decode_token(access_token, &test_app.configuration);
            let refresh_token_claims =
                Helpers::decode_token(refresh_token, &test_app.configuration);

            // assert new tokens are valid
            assert!(access_token_claims.is_ok());
            assert!(refresh_token_claims.is_ok());

            assert_eq!(
                access_token_claims.unwrap().sub,
                refresh_token_claims.unwrap().sub,
            );
        }
    }
}

#[actix_web::test]
async fn test_refresh_with_invalid_token() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        let valid_token: &str = &tokens.refresh;

        for (token, err_msg, status_code) in InvalidTestCases::refresh_tokens(valid_token) {
            let body = RefreshTokenRequest {
                refresh_token: token.to_string(),
            };
            let response = test_app.refresh(&body).await;

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
async fn test_refresh_token_single_use() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        let refresh_token: &str = &tokens.refresh;

        // refresh token
        let body = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        };
        let response = test_app.refresh(&body).await;
        assert!(response.status().is_success());

        // refresh token twice should fail
        let response = test_app.refresh(&body).await;
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            "refresh-token was used twice"
        );
    }
}

#[actix_web::test]
async fn test_refresh_token_route_using_access_token() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        // refresh token
        let body = RefreshTokenRequest {
            refresh_token: tokens.access.clone(),
        };
        let response = test_app.refresh(&body).await;
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            "access-token was used instead of refreshToken"
        );
    }
}
