use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use openssl::bn::{BigNum, BigNumContext};
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::config::AlgorithmConfig;
use crate::domain::model::SigningKey;
use crate::error::{CoreError, InternalError, Result};

#[derive(Clone, Default)]
pub struct Openssl {
    pub algorithm: AlgorithmConfig,
}

impl Openssl {
    pub fn new(algorithm: &AlgorithmConfig) -> Self {
        Self {
            algorithm: algorithm.clone(),
        }
    }
}

impl Openssl {
    pub fn gen_prv_pub_key(&self) -> Result<(String, String)> {
        match self.algorithm {
            AlgorithmConfig::EdDSA => Self::gen_ed(),
            AlgorithmConfig::ES256 => Self::gen_ec(Nid::X9_62_PRIME256V1),
            AlgorithmConfig::ES384 => Self::gen_ec(Nid::SECP384R1),
            _ => Self::gen_rsa(),
        }
    }

    fn encode_keypair(priv_pem: Vec<u8>, pub_pem: Vec<u8>) -> (String, String) {
        (
            BASE64_URL_SAFE_NO_PAD.encode(priv_pem),
            BASE64_URL_SAFE_NO_PAD.encode(pub_pem),
        )
    }

    fn gen_rsa() -> Result<(String, String)> {
        let rsa = Rsa::generate(4096)?;
        Ok(Self::encode_keypair(
            rsa.private_key_to_pem()?,
            rsa.public_key_to_pem()?,
        ))
    }

    fn gen_ec(nid: Nid) -> Result<(String, String)> {
        let group = EcGroup::from_curve_name(nid)?;

        let ec_key = EcKey::generate(&group)?;
        let pkey = PKey::from_ec_key(ec_key)?;

        let private_key_pem = pkey.private_key_to_pem_pkcs8()?;
        let public_key_pem = pkey.public_key_to_pem()?;

        Ok(Self::encode_keypair(private_key_pem, public_key_pem))
    }

    fn gen_ed() -> Result<(String, String)> {
        let pkey = PKey::generate_ed25519()?;
        Ok(Self::encode_keypair(
            pkey.private_key_to_pem_pkcs8()?,
            pkey.public_key_to_pem()?,
        ))
    }
}

impl Openssl {
    fn extract_rsa(rsa: Rsa<openssl::pkey::Public>) -> (String, String) {
        let n = BASE64_URL_SAFE_NO_PAD.encode(rsa.n().to_vec());
        let e = BASE64_URL_SAFE_NO_PAD.encode(rsa.e().to_vec());

        (e, n)
    }
    fn extract_ec(ec: EcKey<openssl::pkey::Public>) -> Result<(String, String, String)> {
        let group = ec.group();
        let point = ec.public_key();
        let mut ctx = BigNumContext::new()?;
        let mut x = BigNum::new()?;
        let mut y = BigNum::new()?;
        point.affine_coordinates_gfp(group, &mut x, &mut y, &mut ctx)?;

        let crv = match group.curve_name() {
            Some(Nid::X9_62_PRIME256V1) => "P-256",
            Some(Nid::SECP384R1) => "P-384",
            _ => {
                return Err(CoreError::Internal(InternalError::Crypto(
                    "unknown curve function".to_string(),
                )));
            }
        };

        let x_enc = BASE64_URL_SAFE_NO_PAD.encode(x.to_vec());
        let y_enc = BASE64_URL_SAFE_NO_PAD.encode(y.to_vec());

        Ok((crv.to_string(), x_enc, y_enc))
    }

    pub fn pem_to_jwk(&self, key: SigningKey) -> Result<serde_json::Value> {
        let public_key_pem: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(key.public_key).unwrap();
        let pkey = PKey::public_key_from_pem(&public_key_pem)?;

        if let Ok(rsa) = pkey.rsa() {
            let (e, n) = Self::extract_rsa(rsa);
            Ok(json!({
                "kty": key.kty,
                "alg": key.algorithm,
                "use": "sig",
                "kid": key.kid,
                "n": n,
                "e": e,
            }))
        } else if let Ok(ec) = pkey.ec_key() {
            let (crv, x, y) = Self::extract_ec(ec)?;
            Ok(json!({
                "kty": key.kty,
                "alg": key.algorithm,
                "use": "sig",
                "kid": key.kid,
                "crv": crv,
                "x": x,
                "y": y,
            }))
        } else if pkey.id() == openssl::pkey::Id::ED25519 {
            let raw = pkey.raw_public_key()?;
            Ok(json!({
                "kty": key.kty,
                "alg": key.algorithm,
                "use": "sig",
                "kid": key.kid,
                "crv": "Ed25519",
                "x": BASE64_URL_SAFE_NO_PAD.encode(raw),
            }))
        } else {
            Err(CoreError::Internal(crate::error::InternalError::Crypto(
                "Failed to read public key contents".to_string(),
            )))
        }
    }
}

impl Openssl {
    fn thumbprint(canonical: &str) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes()))
    }

    pub fn build_rsa(&self, public_key_pem: Vec<u8>) -> Result<(String, String)> {
        let rsa = Rsa::public_key_from_pem(&public_key_pem)?;
        let (e, n) = Self::extract_rsa(rsa);

        let canonical = format!(r#"{{"e":"{}","kty":"RSA","n":"{}"}}"#, e, n);
        Ok((String::from("RSA"), Self::thumbprint(&canonical))) // (kty, kid)
    }
    pub fn build_ec(&self, public_key_pem: Vec<u8>) -> Result<(String, String)> {
        let ec = EcKey::public_key_from_pem(&public_key_pem)?;
        let (crv, x, y) = Self::extract_ec(ec)?;

        let canonical = format!(r#"{{"crv":"{}","kty":"EC","x":"{}","y":"{}"}}"#, crv, x, y);
        Ok((String::from("EC"), Self::thumbprint(&canonical)))
    }
    pub fn build_okp(&self, public_key_pem: Vec<u8>) -> Result<(String, String)> {
        let pkey = PKey::public_key_from_pem(&public_key_pem)?;
        let raw = pkey.raw_public_key()?;
        let x = BASE64_URL_SAFE_NO_PAD.encode(raw);

        let canonical = format!(r#"{{"crv":"{}","kty":"OKP","x":"{}"}}"#, "Ed25519", x);
        Ok((String::from("OKP"), Self::thumbprint(&canonical)))
    }
}
