use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use super::validation::validate_password;
use crate::config::PasswordRequirements;

#[derive(Debug, Validate, Deserialize, ToSchema)]
#[schema(example = json!({"token": "edc365fa5e13751XXXXXXX"}))]
pub struct TokenQuery {
    #[schema(value_type = String, format = Password)]
    pub token: secrecy::SecretString,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"email": "example@email.com", "password": "password"}))]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"username": "name", "email": "example@email.com", "password": "password"}))]
pub struct RegisterRequest {
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"token": String::default(), "csrf_token": String::default(), "password": String::default()}))]
pub struct ResetPasswordRequest {
    #[schema(value_type = String, format = Password)]
    pub token: secrecy::SecretString,

    #[schema(value_type = String, format = Password)]
    pub csrf_token: secrecy::SecretString,

    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}
