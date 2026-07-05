//! Password hashing — PBKDF2-HMAC-SHA256, salted. Format stored in the DB:
//! `pbkdf2_sha256$<iterations>$<salt_hex>$<hash_hex>` (self-describing, so the
//! iteration count / algorithm can change later without breaking old rows).

use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use sha2::Sha256;

const ITERATIONS: u32 = 260_000;
const SALT_LEN: usize = 16;
const HASH_LEN: usize = 32;

pub fn hash_password(password: &str) -> String {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let mut out = [0u8; HASH_LEN];
    let _ = pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, ITERATIONS, &mut out);
    format!(
        "pbkdf2_sha256${}${}${}",
        ITERATIONS,
        hex::encode(salt),
        hex::encode(out)
    )
}

pub fn verify_password(password: &str, stored: &str) -> bool {
    let parts: Vec<&str> = stored.split('$').collect();
    if parts.len() != 4 || parts[0] != "pbkdf2_sha256" {
        return false;
    }
    let iterations: u32 = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let salt = match hex::decode(parts[2]) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let expected = match hex::decode(parts[3]) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut out = vec![0u8; expected.len()];
    let _ = pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, iterations, &mut out);
    // constant-time-ish compare
    out.len() == expected.len() && out.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let h = hash_password("admin");
        assert!(verify_password("admin", &h));
        assert!(!verify_password("wrong", &h));
    }
}
