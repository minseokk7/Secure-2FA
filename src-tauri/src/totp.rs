use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

/// TOTP 코드를 생성합니다.
/// `secret_str`은 Base32 인코딩된 시크릿 키입니다.
pub fn generate_totp_code(secret_str: &str) -> Result<(String, u64), String> {
    let secret = Secret::Encoded(secret_str.to_string())
        .to_bytes()
        .map_err(|e| format!("유효하지 않은 TOTP 시크릿: {}", e))?;

    // new_unchecked: 시크릿 길이 제한을 완화 (실제 서비스에서 짧은 키가 자주 사용됨)
    let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, secret);

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let code = totp.generate(current_time);

    // 남은 시간 계산
    let period: u64 = 30;
    let time_step = current_time / period;
    let next_step = (time_step + 1) * period;
    let remaining_seconds = next_step - current_time;

    Ok((code, remaining_seconds))
}

/// 시크릿 키 형식을 검증합니다.
/// 빈 문자열은 무효로 처리합니다.
pub fn validate_secret_format(secret_str: &str) -> bool {
    if secret_str.is_empty() {
        return false;
    }
    Secret::Encoded(secret_str.to_string()).to_bytes().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 유효한 Base32 시크릿으로 6자리 코드가 생성되는지 검증
    #[test]
    fn test_generate_totp_code_valid_secret() {
        // 충분한 길이의 Base32 시크릿 (20바이트 = 160비트)
        let secret = "HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ";

        let result = generate_totp_code(secret);
        assert!(result.is_ok(), "유효한 시크릿으로 TOTP 생성에 실패했습니다");

        let (code, remaining) = result.unwrap();
        assert_eq!(code.len(), 6, "TOTP 코드는 6자리여야 합니다");
        assert!(
            code.chars().all(|c| c.is_ascii_digit()),
            "TOTP 코드는 숫자로만 구성되어야 합니다"
        );
        assert!(remaining <= 30, "남은 시간은 30초 이하여야 합니다");
        assert!(remaining >= 1, "남은 시간은 1초 이상이어야 합니다");
    }

    /// 짧은 시크릿도 코드를 생성할 수 있는지 검증 (new_unchecked 사용)
    #[test]
    fn test_generate_totp_code_short_secret() {
        let secret = "JBSWY3DPEHPK3PXP"; // 10바이트

        let result = generate_totp_code(secret);
        assert!(
            result.is_ok(),
            "짧은 시크릿으로도 TOTP 생성이 가능해야 합니다"
        );

        let (code, _) = result.unwrap();
        assert_eq!(code.len(), 6);
    }

    /// 유효한 시크릿 형식 검증
    #[test]
    fn test_validate_secret_format_valid() {
        assert!(validate_secret_format("JBSWY3DPEHPK3PXP"));
        assert!(validate_secret_format("HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ"));
    }

    /// 무효한 시크릿 형식 검증
    #[test]
    fn test_validate_secret_format_invalid() {
        assert!(!validate_secret_format("invalid!@#$%"));
        assert!(!validate_secret_format("")); // 빈 문자열
    }
}
