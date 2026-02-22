pub mod crypto;
pub mod db;
pub mod totp;

use db::{Account, Db};
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, State, WindowEvent};
use tokio::sync::Mutex;

struct AppState {
    db: Arc<Mutex<Db>>,
    last_screenshot: Arc<Mutex<Option<image::DynamicImage>>>,
    /// 기기별 고유 암호화 키 (앱 최초 실행 시 랜덤 생성, 이후 파일에서 로드)
    master_key: [u8; 32],
}

// ── 기존 계정 관리 커맨드 ──

#[tauri::command]
async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    let db = state.db.lock().await;
    db.get_accounts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_account(
    issuer: String,
    account_name: String,
    secret_key: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    if !totp::validate_secret_format(&secret_key) {
        return Err("유효하지 않은 TOTP 시크릿 키 형식입니다".into());
    }

    let (encrypted_secret, nonce) =
        crypto::encrypt_secret(&secret_key, &state.master_key).map_err(|e| e.to_string())?;

    let db = state.db.lock().await;
    db.add_account(&issuer, &account_name, &encrypted_secret, &nonce)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_account(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_account(id).await.map_err(|e| e.to_string())
}

/// 계정의 발급자(issuer)와 계정명(account_name)을 수정합니다.
#[tauri::command]
async fn update_account(
    id: i64,
    issuer: String,
    account_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if issuer.trim().is_empty() || account_name.trim().is_empty() {
        return Err("발급자와 계정명은 비어있을 수 없습니다".into());
    }
    let db = state.db.lock().await;
    db.update_account(id, issuer.trim(), account_name.trim())
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct OtpResponse {
    code: String,
    remaining_seconds: u64,
}

#[tauri::command]
async fn get_current_otp(
    encrypted_secret: Vec<u8>,
    nonce: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<OtpResponse, String> {
    let mut nonce_array = [0u8; 12];
    if nonce.len() == 12 {
        nonce_array.copy_from_slice(&nonce);
    } else {
        return Err("유효하지 않은 nonce 길이입니다".into());
    }

    let secret_str = crypto::decrypt_secret(&encrypted_secret, &nonce_array, &state.master_key)
        .map_err(|e| e.to_string())?;

    let (code, remaining_seconds) = totp::generate_totp_code(&secret_str)?;

    Ok(OtpResponse {
        code,
        remaining_seconds,
    })
}

// ── 앱 잠금 (PIN) ──

#[tauri::command]
async fn has_pin(state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().await;
    let pin_hash = db
        .get_setting("pin_hash")
        .await
        .map_err(|e| e.to_string())?;
    Ok(pin_hash.is_some())
}

#[tauri::command]
async fn verify_pin(pin: String, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().await;
    let hash_b64 = db
        .get_setting("pin_hash")
        .await
        .map_err(|e| e.to_string())?;
    let salt_b64 = db
        .get_setting("pin_salt")
        .await
        .map_err(|e| e.to_string())?;

    if let (Some(hash), Some(salt)) = (hash_b64, salt_b64) {
        Ok(crypto::verify_pin_hash(&pin, &hash, &salt))
    } else {
        Ok(false) // 설정된 PIN이 없음
    }
}

#[tauri::command]
async fn set_pin(pin: String, state: State<'_, AppState>) -> Result<bool, String> {
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN은 4자리의 숫자여야 합니다".into());
    }

    let (hash, salt) = crypto::hash_pin(&pin).map_err(|e| e.to_string())?;

    let db = state.db.lock().await;
    db.set_setting("pin_hash", &hash)
        .await
        .map_err(|e| e.to_string())?;
    db.set_setting("pin_salt", &salt)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
async fn remove_pin(current_pin: String, state: State<'_, AppState>) -> Result<bool, String> {
    // 먼저 기존 PIN이 맞는지 확인합니다.
    let is_valid = verify_pin(current_pin, state.clone()).await?;
    if !is_valid {
        return Err("현재 PIN이 일치하지 않습니다".into());
    }

    let db = state.db.lock().await;
    db.delete_setting("pin_hash")
        .await
        .map_err(|e| e.to_string())?;
    db.delete_setting("pin_salt")
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}
// ── 백업 및 복원 (내보내기 / 불러오기) ──

#[tauri::command]
async fn export_backup(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().await;
    let accounts = db.get_accounts().await.map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&accounts).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn import_backup(path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let accounts: Vec<Account> = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let db = state.db.lock().await;
    let mut imported = 0;
    for acc in accounts {
        if db
            .add_account(
                &acc.issuer,
                &acc.account_name,
                &acc.encrypted_secret,
                &acc.secret_nonce,
            )
            .await
            .is_ok()
        {
            imported += 1;
        }
    }
    Ok(imported)
}

// ── QR 코드 스캔 (화면 캐처 및 파일) ──

#[derive(serde::Serialize)]
struct OtpAuthInfo {
    issuer: String,
    account_name: String,
    secret: String,
}

/// QR 코드 이미지에서 디코딩하는 공통 로직
/// 원본 → 다양한 스케일 → 이진화(흑백 대비 강화) 순으로 재시도하며, 모든 감지된 그리드를 순회합니다.
fn decode_qr_from_image(img: &image::DynamicImage) -> Result<String, String> {
    let mut last_err = String::from("QR 코드를 찾을 수 없습니다");

    // 여러 스케일로 시도
    let scale_factors: &[f32] = &[1.0, 0.5, 0.75, 1.5, 2.0, 3.0];

    for &scale in scale_factors {
        let w = (img.width() as f32 * scale) as u32;
        let h = (img.height() as f32 * scale) as u32;
        if w == 0 || h == 0 || w > 8000 || h > 8000 {
            continue;
        }

        let resized = if (scale - 1.0).abs() < 0.01 {
            img.clone()
        } else {
            img.resize_exact(w, h, image::imageops::FilterType::Lanczos3)
        };

        // 1차 시도: 그레이스케일 직접
        let gray = resized.to_luma8();
        if let Some(content) = try_decode_luma(&gray, &mut last_err) {
            return Ok(content);
        }

        // 2차 시도: Otsu 이진화 (대비 강화)
        let binarized = otsu_binarize(&gray);
        if let Some(content) = try_decode_luma(&binarized, &mut last_err) {
            return Ok(content);
        }
    }

    Err(last_err)
}

/// 그레이스케일 이미지에서 QR 그리드 감지 + 디코딩 시도
fn try_decode_luma(gray: &image::GrayImage, last_err: &mut String) -> Option<String> {
    let mut prepared = rqrr::PreparedImage::prepare(gray.clone());
    let grids = prepared.detect_grids();
    for grid in &grids {
        match grid.decode() {
            Ok((_meta, content)) => return Some(content),
            Err(e) => *last_err = format!("QR 디코딩 실패: {}", e),
        }
    }
    None
}

/// Otsu 임계값 이진화 — QR 코드의 흑백 대비를 극대화합니다.
fn otsu_binarize(gray: &image::GrayImage) -> image::GrayImage {
    // 히스토그램 계산
    let mut histogram = [0u32; 256];
    for p in gray.pixels() {
        histogram[p.0[0] as usize] += 1;
    }
    let total = gray.width() * gray.height();

    // Otsu 최적 임계값 계산
    let mut sum_total: f64 = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum_total += i as f64 * count as f64;
    }

    let mut sum_bg: f64 = 0.0;
    let mut weight_bg: f64 = 0.0;
    let mut max_variance: f64 = 0.0;
    let mut threshold: u8 = 0;

    for (i, &count) in histogram.iter().enumerate() {
        weight_bg += count as f64;
        if weight_bg == 0.0 {
            continue;
        }
        let weight_fg = total as f64 - weight_bg;
        if weight_fg == 0.0 {
            break;
        }

        sum_bg += i as f64 * count as f64;
        let mean_bg = sum_bg / weight_bg;
        let mean_fg = (sum_total - sum_bg) / weight_fg;
        let variance = weight_bg * weight_fg * (mean_bg - mean_fg).powi(2);

        if variance > max_variance {
            max_variance = variance;
            threshold = i as u8;
        }
    }

    // 이진화 적용
    let mut result = gray.clone();
    for p in result.pixels_mut() {
        p.0[0] = if p.0[0] > threshold { 255 } else { 0 };
    }
    result
}

/// otpauth:// URI 파싱
#[tauri::command]
fn parse_otpauth_uri(uri: String) -> Result<OtpAuthInfo, String> {
    // otpauth://totp/Issuer:account@example.com?secret=BASE32&issuer=Issuer
    let url = url::Url::parse(&uri).map_err(|e| format!("유효하지 않은 URI: {}", e))?;

    if url.scheme() != "otpauth" {
        return Err("otpauth:// 형식이 아닙니다".into());
    }

    let path = url.path().trim_start_matches('/');
    let (issuer_from_path, account_name) = if let Some(idx) = path.find(':') {
        let iss = &path[..idx];
        let acc = &path[idx + 1..];
        (iss.to_string(), acc.to_string())
    } else {
        (String::new(), path.to_string())
    };

    // URL 디코딩
    let account_name = urlencoding::decode(&account_name)
        .unwrap_or(std::borrow::Cow::Borrowed(&account_name))
        .to_string();

    let mut secret = String::new();
    let mut issuer = issuer_from_path;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "secret" => secret = value.to_string(),
            "issuer" => issuer = value.to_string(),
            _ => {}
        }
    }

    if secret.is_empty() {
        return Err("URI에 secret 파라미터가 없습니다".into());
    }

    Ok(OtpAuthInfo {
        issuer,
        account_name,
        secret,
    })
}

/// 전체 화면 스크린샷을 찍고 base64 PNG 데이터를 반환합니다.
/// 이미지는 내부 상태에 저장되어 이후 decode_screenshot_region에서 사용합니다.
#[tauri::command]
async fn take_screenshot(state: State<'_, AppState>) -> Result<String, String> {
    // xcap::Monitor는 Send를 구현하지 않으므로 blocking 스레드에서 실행
    let (img, b64) = tokio::task::spawn_blocking(|| {
        use image::ImageEncoder;
        use std::io::Cursor;
        use xcap::Monitor;

        let monitors = Monitor::all().map_err(|e| format!("모니터 정보 조회 실패: {}", e))?;
        let monitor = monitors
            .into_iter()
            .next()
            .ok_or("모니터를 찾을 수 없습니다".to_string())?;

        let screenshot = monitor
            .capture_image()
            .map_err(|e| format!("스크린 캐처 실패: {}", e))?;
        let img = image::DynamicImage::ImageRgba8(screenshot);

        // base64 PNG 인코딩
        let mut buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buf));
        encoder
            .write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                img.color().into(),
            )
            .map_err(|e| format!("PNG 인코딩 실패: {}", e))?;

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);

        Ok::<_, String>((img, b64))
    })
    .await
    .map_err(|e| format!("스레드 실행 실패: {}", e))??;

    // 내부 상태에 스크린샷 저장
    let mut lock = state.last_screenshot.lock().await;
    *lock = Some(img);

    Ok(format!("data:image/png;base64,{}", b64))
}

/// 저장된 스크린샷에서 지정 영역을 크롭하여 QR 코드를 디코딩합니다.
#[tauri::command]
async fn decode_screenshot_region(
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let lock = state.last_screenshot.lock().await;
    let img = lock
        .as_ref()
        .ok_or("저장된 스크린샷이 없습니다. 먼저 스크린샷을 찍어주세요.")?;

    // 1차: 크롭된 영역에서 디코딩 시도
    let cropped = img.crop_imm(x, y, w, h);
    if let Ok(content) = decode_qr_from_image(&cropped) {
        return Ok(content);
    }

    // 2차 fallback: 전체 스크린샷에서 디코딩 시도
    decode_qr_from_image(img)
}

/// 저장된 젼체 스크린샷에서 바로 QR 코드를 디코딩합니다. (자동 감지용)
#[tauri::command]
async fn decode_screenshot_auto(state: State<'_, AppState>) -> Result<String, String> {
    let lock = state.last_screenshot.lock().await;
    let img = lock
        .as_ref()
        .ok_or("저장된 스크린샷이 없습니다. 먼저 스크린샷을 찍어주세요.")?;

    decode_qr_from_image(img)
}

/// 이미지 파일에서 QR 코드 디코딩
#[tauri::command]
fn scan_qr_from_file(path: String) -> Result<String, String> {
    let img = image::open(&path).map_err(|e| format!("이미지 열기 실패: {}", e))?;
    decode_qr_from_image(&img)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        let shortcut_str = shortcut.into_string();
                        if shortcut_str.contains("Shift") && shortcut_str.contains("KeyA") {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 다른 인스턴스가 실행되려 할 때 처리: 기존 창을 보여주고 포커스
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let app_dir = dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("secure2fa");

                std::fs::create_dir_all(&app_dir).unwrap();

                // 기기별 고유 마스터 키 로드 또는 생성
                let master_key =
                    crypto::load_or_create_master_key(&app_dir).expect("마스터 키 초기화 실패");

                let db = Db::new(&app_dir).await.unwrap();
                let db_arc = Arc::new(Mutex::new(db));

                app_handle.manage(AppState {
                    db: db_arc,
                    last_screenshot: Arc::new(Mutex::new(None)),
                    master_key,
                });
            });

            // 트레이 아이콘 설정
            let quit_i = MenuItemBuilder::with_id("quit", "종료").build(app)?;
            let show_i = MenuItemBuilder::with_id("show", "창 열기").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_i)
                .separator()
                .item(&quit_i)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 글로벌 단축키 등록 (Ctrl+Shift+A)
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            app.global_shortcut()
                .register("ctrl+shift+a")
                .unwrap_or_else(|e| eprintln!("단축키 등록 실패: {}", e));

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            add_account,
            delete_account,
            update_account,
            get_current_otp,
            export_backup,
            import_backup,
            take_screenshot,
            decode_screenshot_auto,
            decode_screenshot_region,
            parse_otpauth_uri,
            scan_qr_from_file,
            has_pin,
            verify_pin,
            set_pin,
            remove_pin,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 앱 실행 중 에러 발생");
}
