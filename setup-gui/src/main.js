const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.dialog;

const appWindow = getCurrentWindow();

// UI Elements
const minBtn = document.getElementById('titlebar-minimize');
const closeBtn = document.getElementById('titlebar-close');
const installPathEl = document.getElementById('install-path');
const selectFolderBtn = document.getElementById('select-folder-btn');
const installBtn = document.getElementById('install-btn');

const versionArea = document.getElementById('version-area');
const latestVersionBadge = document.getElementById('latest-version-badge');
const releaseNotes = document.getElementById('release-notes');

const progressArea = document.getElementById('progress-area');
const progressBar = document.getElementById('progress-bar');
const statusText = document.getElementById('status-text');

let selectedPath = "";
let downloadUrl = "";

// Init
async function init() {
  // Window controls
  minBtn.addEventListener('click', () => appWindow.minimize());
  closeBtn.addEventListener('click', () => appWindow.close());

  // Setup basic window size
  const factor = await appWindow.scaleFactor();
  appWindow.setSize(new window.__TAURI__.window.PhysicalSize(500 * factor, 460 * factor));

  // Get default path
  try {
    selectedPath = await invoke('get_default_install_path');
    installPathEl.textContent = selectedPath;
  } catch (e) {
    installPathEl.textContent = "C:\\Secure2FA";
    selectedPath = "C:\\Secure2FA";
  }

  // Check version
  try {
    const info = await invoke('check_latest_version');
    latestVersionBadge.textContent = "v" + info.latest;
    releaseNotes.textContent = info.release_notes || "릴리즈 노트 제공되지 않음";
    downloadUrl = info.download_url;
    versionArea.style.display = 'block';

    // Update button
    installBtn.textContent = '🚀 다운로드 및 설치';
    installBtn.classList.add('active');
    installBtn.disabled = false;

    // expand
    appWindow.setSize(new window.__TAURI__.window.PhysicalSize(500 * factor, 600 * factor));

  } catch (e) {
    releaseNotes.textContent = "버전 확인 실패: " + e;
    versionArea.style.display = 'block';
    installBtn.textContent = '오류 발생 (재시작 필요)';
  }

  // Listeners
  selectFolderBtn.addEventListener('click', async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: selectedPath,
      title: 'Secure 2FA 설치 폴더 선택'
    });

    if (selected) {
      selectedPath = selected;
      installPathEl.textContent = selectedPath;
    }
  });

  installBtn.addEventListener('click', runInstall);

  listen('install-progress', (event) => {
    const { stage, progress, message } = event.payload;
    progressArea.style.display = 'block';
    progressBar.style.width = progress + '%';
    statusText.textContent = message;
  });
}

async function runInstall() {
  if (!selectedPath || !downloadUrl) return;

  installBtn.disabled = true;
  installBtn.classList.remove('active');
  installBtn.textContent = '설치 진행 중...';
  selectFolderBtn.style.pointerEvents = 'none';

  progressArea.style.display = 'block';

  try {
    await invoke('run_install', {
      installPath: selectedPath,
      downloadUrl: downloadUrl
    });
    statusText.textContent = '✅ 설치가 완료되었습니다!';
    progressBar.style.width = '100%';
    installBtn.textContent = '닫기';
    installBtn.classList.add('active');
    installBtn.disabled = false;
    installBtn.removeEventListener('click', runInstall);
    installBtn.addEventListener('click', () => appWindow.close());
  } catch (e) {
    statusText.textContent = '❌ 오류: ' + e;
    installBtn.textContent = '다시 시도';
    installBtn.classList.add('active');
    installBtn.disabled = false;
    selectFolderBtn.style.pointerEvents = 'auto';
  }
}

init();
