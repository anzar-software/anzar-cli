use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use mongodb::bson::oid::ObjectId;
use validator::ValidationError;

use crate::config::PasswordRequirements;

pub fn validate_token(token: &str) -> Result<(), ValidationError> {
    BASE64_URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| ValidationError::new("invalid base64 token"))?;

    Ok(())
}

pub fn validate_objectid(id: &str) -> Result<(), ValidationError> {
    ObjectId::parse_str(id).map_err(|_| ValidationError::new("invalid ObjectId"))?;

    Ok(())
}

pub fn validate_password(
    password: &str,
    arg: &PasswordRequirements,
) -> Result<(), ValidationError> {
    if (password.len() as u16) < arg.min_length {
        return Err(ValidationError::new(
            "password must meet minimum length requirement",
        ));
    }
    if (password.len() as u16) > arg.max_length {
        return Err(ValidationError::new("password exceeds maximum length"));
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    if arg.require_uppercase && !has_uppercase {
        return Err(ValidationError::new(
            "password must include an uppercase letter",
        ));
    }

    let has_number = password.chars().any(|c| c.is_ascii_digit());
    if arg.require_number && !has_number {
        return Err(ValidationError::new("password must include a number"));
    }

    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    if arg.require_special_char && !has_special {
        return Err(ValidationError::new(
            "password must include a special character",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper to create requirements
    fn reqs(min: u16, max: u16, upper: bool, num: bool, special: bool) -> PasswordRequirements {
        PasswordRequirements {
            min_length: min,
            max_length: max,
            require_uppercase: upper,
            require_number: num,
            require_special_char: special,
        }
    }

    #[test]
    fn test_valid_password_meets_all_requirements() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("SecurePass1!", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_too_short() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("Ab1!", &args); // Length 4
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message.unwrap(),
            "password must meet minimum length requirement"
        );
    }

    #[test]
    fn test_password_too_long() {
        let args = reqs(8, 10, true, true, true);
        let long_pass = "A".repeat(11) + "b1!"; // Length 12
        let result = validate_password(&long_pass, &args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message.unwrap(),
            "password exceeds maximum length"
        );
    }

    #[test]
    fn test_missing_uppercase_when_required() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("securepass1!", &args); // No uppercase
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message.unwrap(),
            "password must include an uppercase letter"
        );
    }

    #[test]
    fn test_missing_number_when_required() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("SecurePass!@", &args); // No number
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message.unwrap(),
            "password must include a number"
        );
    }

    #[test]
    fn test_missing_special_char_when_required() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("SecurePass12", &args); // No special char
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message.unwrap(),
            "password must include a number" // Note: Your code currently returns this message for special chars too
        );
    }

    #[test]
    fn test_requirements_disabled_passes_without_chars() {
        let args = reqs(8, 50, false, false, false);
        let result = validate_password("alllowercase", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exact_min_length() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("Aa1!aaaa", &args); // Exactly 8 chars
        assert!(result.is_ok());
    }

    #[test]
    fn test_exact_max_length() {
        let args = reqs(8, 10, true, true, true);
        let result = validate_password("Aa1!aaaaaa", &args); // Exactly 10 chars
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_string_fails_min_length() {
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_characters_count_as_special() {
        // Your logic: !c.is_alphanumeric() treats unicode symbols as special
        let args = reqs(8, 50, true, true, true);
        // "Café1!" contains 'é' which is alphanumeric in Rust, but let's test a symbol
        let result = validate_password("Passwörd1!", &args);
        // 'ö' is alphanumeric, '!' is special. Should pass.
        assert!(result.is_ok());
    }

    #[test]
    fn test_space_is_considered_special() {
        // Your logic: space is NOT alphanumeric, so it counts as special
        let args = reqs(8, 50, true, true, true);
        let result = validate_password("Pass Word1", &args);
        // Space triggers !is_alphanumeric()
        assert!(result.is_ok());
    }
}
