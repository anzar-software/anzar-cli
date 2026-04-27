use super::shared::{Helpers, InvalidTestCases};

#[actix_web::test]
async fn test_register_success() {
    let test_app = Helpers::init_config().await;

    let response = test_app.register(None).await;
    assert!(response.status().is_success());
}

#[actix_web::test]
async fn test_register_failures() {
    // Arrange
    let test_app = Helpers::init_config().await;

    for (body, message, code) in InvalidTestCases::registration_credentials().into_iter() {
        // for duplication email test, need to create a valid user before
        if message == "duplication emails" {
            test_app.register(None).await;
        }

        // Act
        let response = test_app.register(Some(body)).await;

        // Assert
        assert_eq!(
            code,
            response.status().as_u16(),
            "The API did not fail when the payload was: {}",
            message
        );
    }
}
