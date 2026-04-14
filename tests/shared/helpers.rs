#![allow(dead_code)]
use anzar::{
    config::{AnzarConfiguration, database::cache_driver::CacheDriver},
    extractors::{Claims, TokenType},
    services::jwt::JwtDecoder,
};
use redis::TypedCommands;
use reqwest::Response;
use uuid::Uuid;

use crate::shared::TestApp;
use anzar::error::Result;

use super::common::Common;
use super::test_cases::ValidTestCases;

pub struct Helpers;
impl Helpers {
    pub async fn init_config() -> TestApp {
        let test_app = Common::spawn_app().await.unwrap();

        // Clear the cache
        match test_app.configuration.database.cache.driver {
            CacheDriver::MemCached => {
                let client =
                    memcache::Client::connect(test_app.configuration.database.cache.clone().url)
                        .unwrap();
                client.flush().unwrap();
            }
            CacheDriver::Redis => {
                let client =
                    redis::Client::open(test_app.configuration.database.cache.clone().url).unwrap();
                let _ = client.get_connection().unwrap().flushall();
            }
        }

        test_app
    }

    pub async fn login(test_app: &TestApp) -> Response {
        let body = ValidTestCases::login_data();
        test_app
            .client
            .post(format!("{}/auth/login", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
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

    pub async fn create_user(test_app: &TestApp) -> Response {
        let body = ValidTestCases::register_data();
        test_app
            .client
            .post(format!("{}/auth/register", test_app.address))
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

    pub async fn create_user2(test_app: &TestApp) -> Response {
        let body = ValidTestCases::register_data2();
        test_app
            .client
            .post(format!("{}/auth/register", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn create_user_with_account_blocked(test_app: &TestApp) -> Response {
        let body = ValidTestCases::blocked_account();
        test_app
            .client
            .post(format!("{}/auth/register", test_app.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn get_user(test_app: &TestApp, token: String) -> Response {
        test_app
            .client
            .get(format!("{}/user", test_app.address))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn decode_token(token: &str, configuration: &AnzarConfiguration) -> Result<Claims> {
        JwtDecoder::new(token, configuration).decode()
    }
}
