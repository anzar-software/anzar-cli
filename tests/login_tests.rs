mod shared;
use shared::{Helpers, InvalidTestCases};

#[actix_web::test]
async fn test_login_success() {
    let test_app = Helpers::init_config().await;

    // Create User
    let response = Helpers::create_user(&test_app).await;
    assert!(response.status().is_success());

    // Login
    let response = Helpers::login(&test_app).await;
    assert!(response.status().is_success());
}

#[actix_web::test]
async fn test_login_failure() {
    // Arrange
    let test_app = Helpers::init_config().await;

    for (body, message, code) in InvalidTestCases::login_credentials().into_iter() {
        // Act
        let response = test_app
            .client
            .post(format!("{}/auth/login", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            code,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            message
        );
    }
}

#[actix_web::test]
async fn test_account_lockout() {
    // Arrange
    let test_app = Helpers::init_config().await;

    // Create User
    let response = Helpers::create_user2(&test_app).await;
    assert!(response.status().is_success());

    for _ in 0..test_app
        .configuration
        .auth
        .password
        .security
        .max_failed_attempts
    {
        // Act
        let response = Helpers::login(&test_app).await;
        // Assert
        assert_eq!(401, response.status().as_u16());
    }

    // Act
    let response = Helpers::login(&test_app).await;
    // Assert
    assert_eq!(403, response.status().as_u16());
}
