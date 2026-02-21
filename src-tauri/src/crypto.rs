use ring::{
    aead::{self, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use std::error::Error;

const NONCE_LEN: usize = 12;

struct RandomNonceSequence {
    nonce: [u8; NONCE_LEN],
}

impl RandomNonceSequence {
    fn new(nonce: [u8; NONCE_LEN]) -> Self {
        Self { nonce }
    }
}

impl NonceSequence for RandomNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

pub fn encrypt_secret(
    secret: &str,
    key_bytes: &[u8; 32],
) -> Result<(Vec<u8>, [u8; NONCE_LEN]), Box<dyn Error>> {
    let unbound_key =
        UnboundKey::new(&aead::AES_256_GCM, key_bytes).map_err(|_| "Invalid key length")?;

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| "Failed to generate nonce")?;

    let nonce_sequence = RandomNonceSequence::new(nonce_bytes);
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    let mut in_out = secret.as_bytes().to_vec();
    sealing_key
        .seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to encrypt")?;

    Ok((in_out, nonce_bytes))
}

pub fn decrypt_secret(
    encrypted_data: &[u8],
    nonce_bytes: &[u8; NONCE_LEN],
    key_bytes: &[u8; 32],
) -> Result<String, Box<dyn Error>> {
    let unbound_key =
        UnboundKey::new(&aead::AES_256_GCM, key_bytes).map_err(|_| "Invalid key length")?;
    let nonce_sequence = RandomNonceSequence::new(*nonce_bytes);
    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

    let mut in_out = encrypted_data.to_vec();
    let decrypted_data = opening_key
        .open_in_place(aead::Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to decrypt")?;

    let decrypted_str = String::from_utf8(decrypted_data.to_vec())
        .map_err(|_| "Invalid UTF-8 in decrypted data")?;

    Ok(decrypted_str)
}

// ── PIN 해싱 및 검증 로직 ──
use ring::pbkdf2;
use std::num::NonZeroU32;

pub fn hash_pin(pin: &str) -> Result<(String, String), Box<dyn Error>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).map_err(|_| "Failed to generate salt")?;

    let iterations = NonZeroU32::new(100_000).unwrap();
    let mut pbkdf2_hash = [0u8; 32];

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        &salt,
        pin.as_bytes(),
        &mut pbkdf2_hash,
    );

    let hash_b64 = STANDARD.encode(&pbkdf2_hash);
    let salt_b64 = STANDARD.encode(&salt);

    Ok((hash_b64, salt_b64))
}

pub fn verify_pin_hash(pin: &str, saved_hash_b64: &str, saved_salt_b64: &str) -> bool {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let Ok(saved_hash) = STANDARD.decode(saved_hash_b64) else {
        return false;
    };
    let Ok(salt) = STANDARD.decode(saved_salt_b64) else {
        return false;
    };

    let iterations = NonZeroU32::new(100_000).unwrap();
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        &salt,
        pin.as_bytes(),
        &saved_hash,
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 암호화 → 복호화 라운드트립이 정상 동작하는지 검증
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key: [u8; 32] = *b"test_key_for_crypto_roundtrip__!";
        let original = "JBSWY3DPEHPK3PXP";

        let (encrypted, nonce) = encrypt_secret(original, &key).expect("암호화에 실패했습니다");

        // 암호화된 데이터는 원본과 달라야 합니다
        assert_ne!(encrypted, original.as_bytes());

        let decrypted = decrypt_secret(&encrypted, &nonce, &key).expect("복호화에 실패했습니다");

        assert_eq!(decrypted, original);
    }

    /// 잘못된 키로 복호화 시 실패해야 합니다
    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key: [u8; 32] = *b"correct_key_for_this_test_ok!!!_";
        let wrong_key: [u8; 32] = *b"wrong___key_for_this_test_ok!!!_";
        let original = "SECRETBASE32VALUE";

        let (encrypted, nonce) = encrypt_secret(original, &key).expect("암호화에 실패했습니다");

        let result = decrypt_secret(&encrypted, &nonce, &wrong_key);
        assert!(result.is_err(), "잘못된 키로 복호화가 성공해서는 안 됩니다");
    }

    /// 빈 문자열 암호화/복호화
    #[test]
    fn test_empty_string_roundtrip() {
        let key: [u8; 32] = *b"key_for_empty_string_test_ok!!!_";
        let original = "";

        let (encrypted, nonce) = encrypt_secret(original, &key).expect("암호화에 실패했습니다");

        let decrypted = decrypt_secret(&encrypted, &nonce, &key).expect("복호화에 실패했습니다");

        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_pin_hash_and_verify() {
        let pin = "123456";
        let (hash, salt) = hash_pin(pin).expect("PIN 해싱 실패");

        assert!(
            verify_pin_hash(pin, &hash, &salt),
            "올바른 PIN으로 검증 성공해야 함"
        );
        assert!(
            !verify_pin_hash("654321", &hash, &salt),
            "잘못된 PIN으로 검증 실패해야 함"
        );
    }
}
