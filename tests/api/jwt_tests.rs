use super::shared::{Helpers, InvalidTestCases, auth::AuthResponseDto as AuthResponse};

use anzar::config::AuthStrategy;

#[actix_web::test]
async fn test_jwt_contains_correct_claims() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
        && let Some(tokens) = &auth_response.tokens
    {
        let access_token: &str = &tokens.access;
        let refresh_token: &str = &tokens.refresh;

        assert!(!access_token.is_empty() && !refresh_token.is_empty());
        let access_token_claims = helpers.decode_token(access_token);
        let refresh_token_claims = helpers.decode_token(refresh_token);

        assert!(access_token_claims.is_ok());
        assert!(refresh_token_claims.is_ok());

        assert_eq!(
            auth_response.user.id,
            Some(access_token_claims.unwrap().sub)
        );
    }
}

#[actix_web::test]
async fn test_protected_route_with_valid_jwt() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
        && let Some(tokens) = &auth_response.tokens
    {
        let response = test_app.user(&format!("Bearer {}", &tokens.access)).await;
        assert!(response.status().is_success());
    }
}

#[actix_web::test]
async fn test_protected_route_with_invalid_jwt() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
        && let Some(tokens) = &auth_response.tokens
    {
        let valid_token: &str = &tokens.access;

        for (token, err_msg, status_code) in InvalidTestCases::jwt_tokens(valid_token) {
            let response = test_app.user(&token).await;

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
async fn test_protected_route_with_refresh_token() {
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    let strategy = &test_app.app_state.configuration.auth.strategy;
    if matches!(strategy, AuthStrategy::Jwt(..))
        && let Some(tokens) = &auth_response.tokens
    {
        let response = test_app.user(&tokens.refresh).await;
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when refreshToken was used instead of accessToken",
        );
    }
}
