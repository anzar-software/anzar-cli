use utoipa::ToSchema;

#[derive(
    Debug, Default, Clone, Copy, serde::Deserialize, serde::Serialize, Eq, PartialEq, ToSchema,
)]
pub enum AuthDriver {
    #[default]
    Jwt,
    Session,
}

impl AuthDriver {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthDriver::Jwt => "jwt",
            AuthDriver::Session => "session",
        }
    }
}
impl std::fmt::Display for AuthDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl TryFrom<String> for AuthDriver {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "jwt" => Ok(Self::Jwt),
            "session" => Ok(Self::Session),
            other => Err(format!(
                "{} is not supported database. Use either `jwt` or `session`",
                other
            )),
        }
    }
}
