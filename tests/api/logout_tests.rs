use super::shared::Helpers;
use anzar::{config::AuthStrategy, scopes::auth::AuthResponse};

use crate::shared::RefreshTokenRequest;

#[actix_web::test]
async fn test_logout_success() {
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
        let refresh_token: &str = &tokens.refresh;

        // Logout
        let body = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        };
        let response = test_app.logout(&tokens.access, &body).await;
        assert!(response.status().is_success());
    }
}

#[actix_web::test]
async fn test_logout_with_invalid_token() {
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

        let body = RefreshTokenRequest {
            refresh_token: access_token.into(),
        };
        let response = test_app.logout(access_token, &body).await;
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            "using accessToken instead of refresh_token"
        );

        let response = test_app
            .client
            .post(format!("{}/auth/logout", test_app.address))
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            401,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            "not sending a refresh_token"
        );
    }
}
