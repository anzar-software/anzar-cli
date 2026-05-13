use super::shared::{Helpers, InvalidTestCases, RefreshTokenRequest, auth::AuthResponseDto};
use anzar::config::AuthStrategy;

#[actix_web::test]
async fn test_refresh_token_success() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponseDto = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
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

        let auth_response: AuthResponseDto = response.json().await.unwrap();
        if let Some(tokens) = &auth_response.tokens {
            let access_token: &str = &tokens.access;
            let refresh_token: &str = &tokens.refresh;

            // assert tokens are not empty
            assert!(!access_token.is_empty() && !refresh_token.is_empty());

            let access_token_claims = helpers.decode_token(access_token);
            let refresh_token_claims = helpers.decode_token(refresh_token);

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
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponseDto = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
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
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponseDto = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
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
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponseDto = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
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
