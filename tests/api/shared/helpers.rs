use anzar::config::AppState;
use anzar::domain::model::Claims;
use reqwest::Response;

use crate::shared::TestApp;
use anzar::error::Result;

use super::common::Common;
use super::test_cases::ValidTestCases;

pub struct Helpers {
    pub app_state: AppState,
    pub test_app: TestApp,
}
impl Helpers {
    pub async fn init_config() -> Self {
        let test_app = Common.spawn_app().await.unwrap();

        Self {
            app_state: test_app.clone().app_state,
            test_app,
        }
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

    pub fn decode_token(&self, token: &str) -> Result<Claims> {
        self.app_state.crypto.jwt()?.decode(token)
    }
}
