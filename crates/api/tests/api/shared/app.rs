use api::state::AppState;
use shared::config::AuthStrategy;

use crate::shared::{
    EmailRequest, LoginRequest, RefreshTokenRequest, RegisterRequest, ValidTestCases,
};

#[derive(Clone)]
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    #[allow(dead_code)]
    pub app_state: AppState,
}
impl TestApp {
    pub async fn register(&self, body: Option<RegisterRequest>) -> reqwest::Response {
        let data = match body {
            Some(v) => v,
            None => ValidTestCases::register_data(),
        };

        self.client
            .post(format!("{}/auth/register", self.address))
            .json(&data)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn login(&self, body: Option<LoginRequest>) -> reqwest::Response {
        let data = match body {
            Some(v) => v,
            None => ValidTestCases::login_data(),
        };

        self.client
            .post(format!("{}/auth/login", self.address))
            .json(&data)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn user(&self, token: &str, strategy: &AuthStrategy) -> reqwest::Response {
        let mut req = self.client.get(format!("{}/user", self.address));

        req = match strategy {
            AuthStrategy::Session(_) => req.header("Cookie", token),
            AuthStrategy::Jwt(_) => req.header("authorization", token),
        };

        req.send().await.expect("Failed to execute request.")
    }
    pub async fn logout(
        &self,
        bearer_token: &str,
        refresh_token: &RefreshTokenRequest,
    ) -> reqwest::Response {
        self.client
            .post(format!("{}/auth/logout", self.address))
            .bearer_auth(bearer_token)
            .json(refresh_token)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn session_logout(&self, token: &str) -> reqwest::Response {
        self.client
            .post(format!("{}/auth/logout", self.address))
            .header("Cookie", token)
            .json(&RefreshTokenRequest {
                refresh_token: String::default(),
            })
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn refresh(&self, refresh_token: &RefreshTokenRequest) -> reqwest::Response {
        self.client
            .post(format!("{}/auth/refresh-token", self.address))
            .json(refresh_token)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn forgot_password(&self, body: &EmailRequest) -> reqwest::Response {
        self.client
            .post(format!("{}/auth/password/forgot", self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
