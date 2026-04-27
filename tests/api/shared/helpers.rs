use anzar::config::AnzarConfiguration;
use anzar::{extractors::Claims, services::jwt::JwtDecoder};
use reqwest::Response;

use crate::shared::TestApp;
use anzar::error::Result;

use super::common::Common;
use super::test_cases::ValidTestCases;

pub struct Helpers;
impl Helpers {
    pub async fn init_config() -> TestApp {
        Common.spawn_app().await.unwrap()
    }

    pub async fn login_with_email(test_app: &TestApp, email: &str) -> Response {
        let body = ValidTestCases::login_data_with_email(email);
        test_app
            .client
            .post(format!("{}/auth/login", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_user_with_email(test_app: &TestApp, email: &str) -> Response {
        let body = ValidTestCases::register_data_with_email(email);
        test_app
            .client
            .post(format!("{}/auth/register", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn decode_token(token: &str, configuration: &AnzarConfiguration) -> Result<Claims> {
        JwtDecoder::new(token, configuration).decode()
    }
}
