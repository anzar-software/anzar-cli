use super::shared::{Helpers, RefreshTokenRequest};
use anzar::{config::AuthStrategy, scopes::auth::AuthResponse};

#[actix_web::test]
async fn test_password_not_returned_in_responses() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());
    let auth_response = response.json::<AuthResponse>().await;
    assert!(auth_response.is_ok());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());
    let auth_response = response.json::<AuthResponse>().await;
    assert!(auth_response.is_ok());

    // Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        let refresh_token: &str = &tokens.refresh;

        // Logout
        let body = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        };
        let response = test_app.logout(&tokens.access, &body).await;
        assert!(response.status().is_success());

        let user = response.json::<()>().await;
        assert!(user.is_err());
    }
}

#[actix_web::test]
async fn test_complete_auth_flow() {
    let test_app = Helpers::init_config().await;

    // [1] Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    // [2] Login
    let response = test_app.login(None).await;
    assert!(response.status().is_success());

    let auth_response: AuthResponse = response.json().await.unwrap();

    if test_app.configuration.auth.strategy == AuthStrategy::Jwt
        && let Some(tokens) = &auth_response.tokens
    {
        let old_refresh_token: &str = &tokens.refresh;
        let old_access_token: &str = &tokens.access;
        assert!(!old_refresh_token.is_empty());

        // [3] Refresh access token
        let body = RefreshTokenRequest {
            refresh_token: old_refresh_token.to_string(),
        };
        let response = test_app.refresh(&body).await;
        assert!(response.status().is_success());

        let auth_response: AuthResponse = response.json().await.unwrap();

        // assert tokens are not empty
        assert!(&auth_response.tokens.is_some());

        let tokens = auth_response.tokens.as_ref().unwrap();
        let new_access_token: &str = &tokens.access;
        let new_refresh_token: &str = &tokens.refresh;

        // assert tokens are not empty
        assert!(!new_access_token.is_empty() && !new_refresh_token.is_empty());

        let access_token_claims = Helpers::decode_token(new_access_token, &test_app.configuration);
        let refresh_token_claims =
            Helpers::decode_token(new_refresh_token, &test_app.configuration);

        // assert new tokens are valid
        assert!(access_token_claims.is_ok());
        assert!(refresh_token_claims.is_ok());

        assert_eq!(
            access_token_claims.unwrap().sub,
            refresh_token_claims.unwrap().sub,
        );

        // [5] Access protected route with valid token
        let response = test_app.user(new_access_token).await;
        assert!(response.status().is_success());

        // [6] Logout with invalid refreshToken

        let body = RefreshTokenRequest {
            refresh_token: old_refresh_token.to_string(),
        };
        let response = test_app.logout(old_access_token, &body).await;

        // this operation should successed even if refreshToken is invalid
        // logout is a safe operation
        assert!(response.status().is_success());

        // [7] Logout with valid refreshToken
        let body = RefreshTokenRequest {
            refresh_token: new_refresh_token.to_string(),
        };
        let response = test_app.logout(new_access_token, &body).await;
        assert!(response.status().is_success());
    }
}
