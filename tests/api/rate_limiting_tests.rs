use crate::shared::EmailRequest;

use super::shared::Helpers;

#[actix_web::test]
async fn test_rate_limiting_for_login() {
    // Arrange
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    let limit = &helpers.app_state.configuration.security.rate_limit;

    // Act
    for _ in 0..limit.strict.capacity - 1 {
        let response = test_app.login(None).await;
        assert!(response.status().is_success());
    }

    let response = test_app.login(None).await;
    // Assert
    assert_eq!(
        429,
        response.status().as_u16(),
        "The API did not fail when the payload was: {}",
        "Passed the rate limit of 5 attemps per hour"
    );
}

#[actix_web::test]
async fn test_rate_limiting_for_forgot_password() {
    // Arrange
    let helpers = Helpers::init_config().await;
    let test_app = helpers.test_app.clone();

    // Create User
    let response = test_app.register(None).await;
    assert!(response.status().is_success());

    let body = EmailRequest {
        email: "hakouguelfen@gmail.com".into(),
    };

    // Act
    let limit = &helpers.app_state.configuration.security.rate_limit;
    for _ in 0..limit.strict.capacity - 1 {
        let response = test_app.forgot_password(&body).await;
        assert!(response.status().is_success());
    }

    let response = test_app.forgot_password(&body).await;
    // Assert
    assert_eq!(
        429,
        response.status().as_u16(),
        "The API did not fail when the payload was: {}",
        "Passed the rate limit of 5 attemps per hour"
    );
}
