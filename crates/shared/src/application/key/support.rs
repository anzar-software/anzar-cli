use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use openssl::rsa::Rsa;
use sha2::{Digest, Sha256};

fn extract_key_infos(pub_key: &str) -> (String, String) {
    let public_key_pem: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(pub_key).unwrap();

    // NOTE add support for EC too
    let rsa = Rsa::public_key_from_pem(&public_key_pem).expect("Failed to parse public key PEM");

    let e = BASE64_URL_SAFE_NO_PAD.encode(rsa.e().to_vec());
    let n = BASE64_URL_SAFE_NO_PAD.encode(rsa.n().to_vec());
    (e, n)
}

pub fn jwk_thumbprint_rsa(pub_key: &str) -> String {
    let (e, n) = extract_key_infos(pub_key);
    let canonical = format!(r#"{{"e":"{}","kty":"RSA","n":"{}"}}"#, e, n);

    // Step 2: SHA-256 hash
    let hash = Sha256::digest(canonical.as_bytes());

    // Step 3: Base64url encode (no padding)
    BASE64_URL_SAFE_NO_PAD.encode(hash)
}

fn _jwk_thumbprint_ec_p256(crv: &str, x: &str, y: &str) -> String {
    // Required members in lexicographic order: crv, kty, x, y
    let canonical = format!(r#"{{"crv":"{}","kty":"EC","x":"{}","y":"{}"}}"#, crv, x, y);

    let hash = Sha256::digest(canonical.as_bytes());
    BASE64_URL_SAFE_NO_PAD.encode(hash)
}

fn _jwk_thumbprint_okp(crv: &str, x: &str) -> String {
    // Required members in lexicographic order: crv, kty, x
    let canonical = format!(r#"{{"crv":"{}","kty":"OKP","x":"{}"}}"#, crv, x);

    let hash = Sha256::digest(canonical.as_bytes());
    BASE64_URL_SAFE_NO_PAD.encode(hash)
}
