use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::fs;
use std::path::Path;

pub struct Db {
    pool: SqlitePool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, FromRow)]
pub struct Account {
    pub id: Option<i64>,
    pub issuer: String,
    pub account_name: String,
    pub encrypted_secret: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub sync_id: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

/// 동기화용 계정 데이터 (네트워크 전송용)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncAccountData {
    pub sync_id: String,
    pub issuer: String,
    pub account_name: String,
    pub encrypted_secret: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub updated_at: String,
    pub deleted: bool,
}

/// 페어링된 기기 정보
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct PairedDevice {
    pub id: Option<i64>,
    pub device_id: String,
    pub device_name: String,
    pub session_token: String,
    pub last_sync_at: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl Db {
    pub async fn new(app_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !app_dir.exists() {
            fs::create_dir_all(app_dir)?;
        }

        let db_path = app_dir.join("vault.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let db = Self { pool };
        db.init().await?;

        Ok(db)
    }

    async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 계정 테이블 (동기화 필드 포함)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                issuer TEXT NOT NULL,
                account_name TEXT NOT NULL,
                encrypted_secret BLOB NOT NULL,
                secret_nonce BLOB NOT NULL,
                sync_id TEXT UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(issuer, account_name)
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // 기존 테이블에 sync_id, updated_at 컬럼 없으면 추가 (마이그레이션)
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN sync_id TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_sync_id ON accounts(sync_id)",
        )
        .execute(&self.pool)
        .await;
        let _ = sqlx::query(
            "ALTER TABLE accounts ADD COLUMN updated_at DATETIME DEFAULT CURRENT_TIMESTAMP",
        )
        .execute(&self.pool)
        .await;

        // sync_id가 NULL인 기존 레코드에 UUID 부여
        sqlx::query(
            "UPDATE accounts SET sync_id = lower(hex(randomblob(16))) WHERE sync_id IS NULL",
        )
        .execute(&self.pool)
        .await?;

        // 페어링된 기기 테이블
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS paired_devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL UNIQUE,
                device_name TEXT NOT NULL,
                session_token TEXT NOT NULL,
                last_sync_at DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        // 앱 설정 테이블 (PIN 등)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ── 앱 설정 (Settings) ──
    pub async fn get_setting(
        &self,
        key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let result: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;

        Ok(result.map(|r| r.0))
    }

    pub async fn set_setting(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"INSERT INTO app_settings (key, value, updated_at)
               VALUES (?, ?, CURRENT_TIMESTAMP)
               ON CONFLICT(key) DO UPDATE SET
                 value = excluded.value,
                 updated_at = excluded.updated_at"#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_setting(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM app_settings WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── 기본 CRUD ──

    pub async fn add_account(
        &self,
        issuer: &str,
        account_name: &str,
        encrypted_secret: &[u8],
        secret_nonce: &[u8],
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let sync_id = uuid::Uuid::new_v4().to_string();
        let result = sqlx::query(
            "INSERT INTO accounts (issuer, account_name, encrypted_secret, secret_nonce, sync_id, updated_at) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(issuer)
        .bind(account_name)
        .bind(encrypted_secret)
        .bind(secret_nonce)
        .bind(&sync_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        let accounts: Vec<Account> = sqlx::query_as(
            "SELECT id, issuer, account_name, encrypted_secret, secret_nonce, sync_id, created_at, updated_at FROM accounts ORDER BY issuer ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }

    pub async fn delete_account(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 계정의 발급자(issuer)와 계정명(account_name)을 수정합니다.
    pub async fn update_account(
        &self,
        id: i64,
        issuer: &str,
        account_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE accounts SET issuer = ?, account_name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(issuer)
        .bind(account_name)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── 동기화 관련 ──

    /// 특정 시점 이후 변경된 계정 목록 조회
    pub async fn get_accounts_since(
        &self,
        since: &str,
    ) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        let accounts: Vec<Account> = sqlx::query_as(
            "SELECT id, issuer, account_name, encrypted_secret, secret_nonce, sync_id, created_at, updated_at FROM accounts WHERE updated_at > ? ORDER BY updated_at ASC"
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }

    /// 동기화 데이터를 기반으로 계정 upsert (sync_id 기준)
    pub async fn upsert_sync_account(
        &self,
        data: &SyncAccountData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"INSERT INTO accounts (issuer, account_name, encrypted_secret, secret_nonce, sync_id, updated_at)
               VALUES (?, ?, ?, ?, ?, ?)
               ON CONFLICT(sync_id) DO UPDATE SET
                 issuer = excluded.issuer,
                 account_name = excluded.account_name,
                 encrypted_secret = excluded.encrypted_secret,
                 secret_nonce = excluded.secret_nonce,
                 updated_at = excluded.updated_at"#
        )
        .bind(&data.issuer)
        .bind(&data.account_name)
        .bind(&data.encrypted_secret)
        .bind(&data.secret_nonce)
        .bind(&data.sync_id)
        .bind(&data.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// sync_id로 계정 삭제
    pub async fn delete_account_by_sync_id(
        &self,
        sync_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM accounts WHERE sync_id = ?")
            .bind(sync_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── 기기 페어링 ──

    /// 페어링 기기 저장
    pub async fn save_paired_device(
        &self,
        device: &PairedDevice,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"INSERT INTO paired_devices (device_id, device_name, session_token)
               VALUES (?, ?, ?)
               ON CONFLICT(device_id) DO UPDATE SET
                 device_name = excluded.device_name,
                 session_token = excluded.session_token"#,
        )
        .bind(&device.device_id)
        .bind(&device.device_name)
        .bind(&device.session_token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 페어링된 기기 목록 조회
    pub async fn get_paired_devices(
        &self,
    ) -> Result<Vec<PairedDevice>, Box<dyn std::error::Error>> {
        let devices: Vec<PairedDevice> = sqlx::query_as(
            "SELECT id, device_id, device_name, session_token, last_sync_at, created_at FROM paired_devices ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(devices)
    }

    /// 세션 토큰으로 기기 인증
    pub async fn verify_session_token(
        &self,
        token: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM paired_devices WHERE session_token = ?",
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;

        Ok(result > 0)
    }

    /// 마지막 동기화 시간 업데이트
    pub async fn update_last_sync(
        &self,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE paired_devices SET last_sync_at = CURRENT_TIMESTAMP WHERE device_id = ?",
        )
        .bind(device_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 페어링 기기 삭제
    pub async fn remove_paired_device(
        &self,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM paired_devices WHERE device_id = ?")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
