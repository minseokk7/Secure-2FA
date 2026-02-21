# Secure 2FA (오프라인 암호화 인증기)

![Secure 2FA App Preview](https://github.com/minseokk7/Secure-2FA/assets/placeholder_image)

**Secure 2FA**는 클라우드 동기화 없이 사용자의 기기 로컬 환경에서만 안전하게 동작하는 오프라인 전용 2단계 인증(2FA/OTP) 데스크톱 애플리케이션입니다. 외부 서버로의 데이터 유출 걱정 없이, 사용자가 설정한 마스터 PIN을 통해서만 모든 백업과 시크릿 키에 접근할 수 있도록 설계된 강력한 보안 도구입니다.

## ✨ 주요 기능

* **완벽한 오프라인 작동**: 서버 통신이 일절 없으며, 모든 중요 데이터는 사용자의 PC(로컬 DB)에만 암호화되어 저장됩니다.
* **강력한 PIN 잠금 시스템 (AES & PBKDF2)**: 
  * 마스터 PIN으로 앱 전체가 잠깁니다.
  * `ring::pbkdf2` 알고리즘을 사용해 랜덤 Salt와 10만 번 이상의 해시 반복(Iterations)으로 PIN을 안전하게 검증합니다.
* **커스텀 타이틀바 기반 모던 UI**: SvelteKit과 Tailwind CSS의 미려한 글래스모피즘(Glassmorphism) 기반 UI로, 투명하고 매끄러운 윈도우 스타일을 제공합니다.
* **드래그 앤 드롭 백업/복원**: 이전에 내보낸 JSON 백업 데이터를 드래그 앤 드롭만으로 즉시 안전하게 복원할 수 있습니다.
* **편리한 OTP 관리 & 실시간 카운트다운**: 원형 프로그레스바를 통해 시간에 따른 OTP 코드 만료 시간을 직관적으로 보여주며 생명 주기에 따라 즉각적으로 클립보드에 코드를 복사할 수 있습니다.
* **QR 코드 이미지 스캔 지원 (예정/진행중)**: 데스크톱 화면의 QR 코드나 이미지 파일을 스캔하여 손쉽게 계정을 추가할 수 있습니다.

## 🛠 시스템 아키텍처 및 보안

- **프론트엔드 (UI)**: Svelte 5 + SvelteKit + Tailwind CSS
- **백엔드 (Core 시스템)**: Rust
- **프레임워크**: Tauri v2
- **데이터베이스**: SQLite (Sqlx 연동)
- **보안/암호화**: 
  - Rust의 `ring` 크레이트를 이용한 PBKDF2 HMACA-SHA256 해시 검증
  - 모든 계정의 시크릿 키는 암호화(ChaCha20-Poly1305 등 AES-GCM 방식 활용) 처리되어 내부 파일 시스템에 저장됨

## 🚀 설치 및 시작 방법

### 사전 요구 사항
- Node.js (v18 이상)
- Rust (Cargo) 메인 체인 프로파일
- Tauri v2 개발 환경 (C++ Build Tools, Windows SDK 등)

### 프로젝트 클론 및 실행
```bash
# 레포지토리 클론
git clone https://github.com/minseokk7/Secure-2FA.git
cd Secure-2FA

# Node 의존성 설치
npm install

# 개발 서버 실행 (Tauri 앱)
npm run tauri dev
```

### 릴리즈 빌드 
```bash
npm run tauri build
```
빌드가 완료되면 `/src-tauri/target/release/bundle` 경로에 `.msi` 및 디버그용 설치 파일이 생성됩니다.

## 📦 데이터 내보내기/불러오기 가이드

- **내보내기**: 홈 화면 우측 상단의 다운로드 화살표 아이콘을 클릭하여 모든 OTP 계정 데이터를 `.json` 형식으로 안전하게 백업할 수 있습니다. 
- **불러오기**: 백업한 `.json` 파일을 화면에 바로 드래그하거나, 업로드 아이콘을 통해 손쉽게 이전할 수 있습니다. (PIN 잠금을 해제한 상태에서만 가능합니다.)

---

> **주의 사항**: 앱을 초기화하거나 삭제하실 경우 SQLite가 설치된 내부 AppData 폴더 및 백업 키가 함께 소실될 수 있습니다. 설치 환경을 포맷하시거나 변경하실 때는 반드시 사전에 [내보내기] 기능을 사용해 주세요.

## 📄 라이센스
MIT License
