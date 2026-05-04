use super::shared::Helpers;
use crate::shared::EmailRequest;

#[actix_web::test]
async fn test_passing_rate_limits() {
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
    for _ in 0..5 {
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
