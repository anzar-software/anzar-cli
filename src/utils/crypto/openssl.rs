use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;

use crate::config::AlgorithmConfig;

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
    pub fn gen_prv_pub_key(&self) -> (String, String) {
        match self.algorithm {
            AlgorithmConfig::EdDSA => self.gen_ed(),
            AlgorithmConfig::ES256 => self.gen_ec(Nid::X9_62_PRIME256V1),
            AlgorithmConfig::ES384 => self.gen_ec(Nid::SECP384R1),
            AlgorithmConfig::RS256
            | AlgorithmConfig::RS384
            | AlgorithmConfig::RS512
            | AlgorithmConfig::PS256
            | AlgorithmConfig::PS384
            | AlgorithmConfig::PS512 => self.gen_rsa(),
        }
    }

    fn gen_rsa(&self) -> (String, String) {
        let rsa = Rsa::generate(4096).expect("Failed to generate RSA key");

        // Get private key as PEM
        let private_key_pem = rsa
            .private_key_to_pem()
            .expect("Failed to encode private key");
        // Get public key as PEM
        let public_key_pem = rsa
            .public_key_to_pem()
            .expect("Failed to encode public key");

        (
            BASE64_URL_SAFE_NO_PAD.encode(private_key_pem),
            BASE64_URL_SAFE_NO_PAD.encode(public_key_pem),
        )
    }

    fn gen_ec(&self, nid: Nid) -> (String, String) {
        let group = EcGroup::from_curve_name(nid).unwrap();

        let ec_key = EcKey::generate(&group).unwrap();
        let pkey = PKey::from_ec_key(ec_key).unwrap();

        let private_key_pem = pkey.private_key_to_pem_pkcs8().unwrap();
        let public_key_pem = pkey.public_key_to_pem().unwrap();

        (
            BASE64_URL_SAFE_NO_PAD.encode(private_key_pem),
            BASE64_URL_SAFE_NO_PAD.encode(public_key_pem),
        )
    }

    fn gen_ed(&self) -> (String, String) {
        let pkey = PKey::generate_ed25519().unwrap();

        let private_key_pem = pkey.private_key_to_pem_pkcs8().unwrap();
        let public_key_pem = pkey.public_key_to_pem().unwrap();

        (
            BASE64_URL_SAFE_NO_PAD.encode(private_key_pem),
            BASE64_URL_SAFE_NO_PAD.encode(public_key_pem),
        )
    }
}
