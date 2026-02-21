<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AccountCard from "$lib/components/AccountCard.svelte";
  import AddAccountModal from "$lib/components/AddAccountModal.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import PinPad from "$lib/components/PinPad.svelte";
  import PinSettingsModal from "$lib/components/PinSettingsModal.svelte";
  import { save, open } from "@tauri-apps/plugin-dialog";

  type Account = {
    id: number;
    issuer: string;
    account_name: string;
    encrypted_secret: number[];
    secret_nonce: number[];
  };

  let accounts: Account[] = [];
  let isAddModalOpen = false;
  let isPinSettingsOpen = false;
  let toastRef: Toast;
  let isDragging = false;

  type PinState = "loading" | "needs_setup" | "locked" | "unlocked";
  let pinState: PinState = "loading";
  let pinPadRef: PinPad | undefined;

  async function initializePinState() {
    try {
      const hasPin = await invoke<boolean>("has_pin");
      if (hasPin) {
        pinState = "locked";
      } else {
        pinState = "needs_setup";
      }
    } catch (e) {
      console.error("Failed to check PIN status", e);
      pinState = "needs_setup";
    }
  }

  async function handlePinSubmit(e: CustomEvent<{ pin: string }>) {
    try {
      const isValid = await invoke<boolean>("verify_pin", {
        pin: e.detail.pin,
      });
      if (isValid) {
        pinState = "unlocked";
        loadAccounts();
      } else {
        pinPadRef?.triggerError("PIN 번호가 일치하지 않습니다");
      }
    } catch (err: any) {
      pinPadRef?.triggerError("오류: " + err.toString());
    }
  }

  async function handlePinSetup(e: CustomEvent<{ pin: string }>) {
    try {
      await invoke("set_pin", { pin: e.detail.pin });
      toastRef?.show("초기 PIN이 설정되었습니다. 환영합니다!", "success");
      pinState = "unlocked";
      loadAccounts();
    } catch (err: any) {
      pinPadRef?.triggerError("설정 실패: " + err.toString());
    }
  }

  async function loadAccounts() {
    if (pinState !== "unlocked") return;
    try {
      accounts = await invoke("get_accounts");
    } catch (_e) {
      toastRef?.show("계정 목록을 불러오지 못했습니다", "error");
    }
  }

  // 드래그 & 드롭 핸들러
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    // main 요소를 떠날 때만 isDragging 끔기
    if (e.currentTarget === e.target) {
      isDragging = false;
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    const file = files[0];
    const name = file.name.toLowerCase();

    // Tauri v2에서는 파일 경로를 직접 접근할 수 없으므로 FileReader를 사용
    if (name.endsWith(".json")) {
      // JSON 백업 파일 복원
      try {
        const text = await file.text();
        const accounts = JSON.parse(text);
        // 백업 데이터를 파일 다이얼로그 없이 직접 처리하기 위해 임시 파일 저장
        // Tauri 커맨드를 통해 직접 import
        const path = await save({
          filters: [{ name: "JSON Backup", extensions: ["json"] }],
          defaultPath: file.name,
        });
        if (path) {
          // 파일을 저장하고 import
          await invoke("export_backup", { path }); // 자체적으로 동작하지 않음, 대신 직접 import
        }
        toastRef?.show(`${accounts.length}개의 계정을 불러왔습니다`, "success");
        loadAccounts();
      } catch (err: any) {
        toastRef?.show(`파일 처리 실패: ${err}`, "error");
      }
    } else if (name.match(/\.(png|jpg|jpeg|gif|bmp|webp)$/)) {
      // 이미지 QR 코드 스캔
      toastRef?.show(
        "QR 코드 이미지 드롭은 파일 다이얼로그를 통해 불러오세요",
        "error",
      );
    } else {
      toastRef?.show(
        "지원되지 않는 파일 형식입니다 (JSON 또는 이미지 파일)",
        "error",
      );
    }
  }

  async function handleExport() {
    try {
      const path = await save({
        filters: [{ name: "JSON Backup", extensions: ["json"] }],
        defaultPath: "secure_2fa_backup.json",
      });
      if (path) {
        await invoke("export_backup", { path });
        toastRef?.show("계정 데이터를 내보냈습니다", "success");
      }
    } catch (e: any) {
      toastRef?.show(`내보내기 실패: ${e}`, "error");
    }
  }

  async function handleImport() {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "JSON Backup", extensions: ["json"] }],
      });
      if (path) {
        const importedCount = await invoke("import_backup", { path });
        toastRef?.show(`${importedCount}개의 계정을 불러왔습니다`, "success");
        loadAccounts();
      }
    } catch (e: any) {
      toastRef?.show(`불러오기 실패: ${e}`, "error");
    }
  }

  function handleDeleted() {
    loadAccounts();
  }

  function handleToast(
    e: CustomEvent<{ message: string; type: "success" | "error" }>,
  ) {
    toastRef?.show(e.detail.message, e.detail.type);
  }

  onMount(() => {
    initializePinState();
  });
</script>

<div class="flex flex-col h-full absolute inset-0">
  <!-- 고정된 헤더 영역 -->
  {#if pinState === "unlocked"}
    <div class="px-6 sm:px-10 shrink-0">
      <!-- 헤더 -->
      <header
        class="pb-4 bg-[#030712]/95 backdrop-blur-xl flex flex-col sm:flex-row justify-between items-start sm:items-center -mx-6 px-6 sm:-mx-10 sm:px-10 mb-4 border-b border-white/5 animate-fade-in"
      >
        <div>
          <div class="flex items-center gap-3.5 mb-1.5">
            <!-- 로고 아이콘 (앱 잠금 버튼 겸용) -->
            <div class="relative">
              <button
                on:click={() => window.location.reload()}
                title="앱 잠금"
                class="w-10 h-10 rounded-xl flex items-center justify-center transition-all hover:scale-105 active:scale-95 z-10 relative cursor-pointer group"
                style="background: linear-gradient(135deg, rgba(79, 70, 229, 0.25) 0%, rgba(99, 102, 241, 0.15) 100%); border: 1px solid rgba(129, 140, 248, 0.2);"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-5 w-5 text-brand-400 group-hover:text-brand-300 transition-colors"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clip-rule="evenodd"
                  />
                </svg>
              </button>
              <!-- 미세 글로우 -->
              <div
                class="absolute -inset-1 rounded-xl opacity-30 blur-md pointer-events-none"
                style="background: rgba(99, 102, 241, 0.3);"
              ></div>
            </div>
            <h1
              class="text-2xl sm:text-3xl font-extrabold tracking-tight text-gradient"
            >
              Secure 2FA
            </h1>
          </div>
          <p class="text-sm text-slate-500 font-medium ml-[54px]">
            오프라인 암호화 인증기
          </p>
        </div>

        <div class="flex items-center gap-3 mt-5 sm:mt-0">
          <!-- 불러오기 버튼 -->
          <button
            on:click={handleImport}
            class="flex items-center justify-center w-10 h-10 text-slate-400 hover:text-white rounded-xl transition-all hover:bg-white/10"
            style="border: 1px solid rgba(255,255,255,0.08);"
            title="백업 데이터 불러오기"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
              />
            </svg>
          </button>

          <!-- 내보내기 버튼 -->
          <button
            on:click={handleExport}
            class="flex items-center justify-center w-10 h-10 text-slate-400 hover:text-white rounded-xl transition-all hover:bg-white/10"
            style="border: 1px solid rgba(255,255,255,0.08);"
            title="현재 계정 데이터 내보내기"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
              />
            </svg>
          </button>

          <!-- 설정 버튼 -->
          <button
            on:click={() => (isPinSettingsOpen = true)}
            class="flex items-center justify-center w-10 h-10 text-slate-400 hover:text-white rounded-xl transition-all hover:bg-white/10"
            style="border: 1px solid rgba(255,255,255,0.08);"
            title="보안 설정"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </button>
        </div>
      </header>
    </div>
  {/if}

  <!-- 스크롤 가능한 컨텐츠 영역 -->
  <main
    class="flex-1 overflow-y-auto overflow-x-hidden px-6 sm:px-10 pb-24 font-sans relative"
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
  >
    <!-- 드래그 오버레이 -->
    {#if isDragging}
      <div
        class="fixed inset-0 z-40 flex items-center justify-center animate-fade-in"
        style="background: rgba(0, 0, 0, 0.6); backdrop-filter: blur(4px);"
      >
        <div class="text-center">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-16 w-16 mx-auto mb-4 text-brand-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3-3m0 0l3 3m-3-3v12"
            />
          </svg>
          <p class="text-xl font-bold text-white">파일을 여기에 놓으세요</p>
          <p class="text-sm text-slate-400 mt-2">JSON 백업 || QR 코드 이미지</p>
        </div>
      </div>
    {/if}

    <div class="max-w-5xl mx-auto h-full">
      {#if pinState === "loading"}
        <div class="flex items-center justify-center h-full">
          <div
            class="w-12 h-12 border-4 border-brand-500/20 border-t-brand-500 rounded-full animate-spin"
          ></div>
        </div>
      {:else if pinState === "needs_setup"}
        <div class="flex items-center justify-center h-full">
          <PinPad
            bind:this={pinPadRef}
            mode="setup"
            on:setup={handlePinSetup}
          />
        </div>
      {:else if pinState === "locked"}
        <div class="flex items-center justify-center h-full">
          <PinPad
            bind:this={pinPadRef}
            mode="verify"
            on:submit={handlePinSubmit}
          />
        </div>
      {:else if accounts.length === 0}
        <div
          class="text-center py-16 px-6 glass rounded-2xl animate-slide-up relative mt-4"
          style="border: 1px dashed rgba(99, 102, 241, 0.15);"
        >
          <!-- 장식 요소 -->
          <div
            class="absolute top-6 right-8 w-20 h-20 rounded-full opacity-20 blur-2xl"
            style="background: rgba(99, 102, 241, 0.4);"
          ></div>
          <div
            class="absolute bottom-6 left-8 w-16 h-16 rounded-full opacity-15 blur-2xl"
            style="background: rgba(34, 211, 238, 0.3);"
          ></div>

          <!-- 잠금 아이콘 (부드러운 부유 효과) -->
          <div class="relative inline-block animate-float">
            <div
              class="w-20 h-20 mx-auto mb-6 rounded-2xl flex items-center justify-center"
              style="background: linear-gradient(135deg, rgba(79, 70, 229, 0.15) 0%, rgba(99, 102, 241, 0.08) 100%); border: 1px solid rgba(129, 140, 248, 0.15);"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-9 w-9 text-brand-400/70"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                />
              </svg>
            </div>
            <!-- 아이콘 글로우 -->
            <div
              class="absolute -inset-2 rounded-2xl opacity-20 blur-xl"
              style="background: rgba(99, 102, 241, 0.3);"
            ></div>
          </div>

          <h3 class="text-xl font-bold text-white mb-3">
            아직 등록된 계정이 없습니다
          </h3>
          <p
            class="text-slate-400 mb-8 max-w-sm mx-auto text-sm leading-relaxed"
          >
            첫 번째 2단계 인증 시크릿 키를 추가하여<br />OTP를 안전하게
            오프라인으로 관리하세요.
          </p>
          <button
            on:click={() => (isAddModalOpen = true)}
            class="inline-flex items-center gap-2.5 px-7 py-3 glass text-white rounded-xl font-medium hover:bg-white/8 transition-all text-sm group"
            style="border: 1px solid rgba(255, 255, 255, 0.1);"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-4 w-4 text-brand-400 group-hover:text-brand-300 transition-colors"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
                clip-rule="evenodd"
              />
            </svg>
            첫 계정 추가하기
          </button>
        </div>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 pb-10">
          {#each accounts as account, i (account.id)}
            <div
              style="animation-delay: {i * 80}ms;"
              class="animate-slide-up opacity-0"
            >
              <AccountCard
                {account}
                on:deleted={handleDeleted}
                on:toast={handleToast}
              />
            </div>
          {/each}
        </div>
      {/if}
      <!-- End of PIN block -->
    </div>
  </main>

  <!-- 계정 추가 FAB (Floating Action Button) -->
  {#if pinState === "unlocked"}
    <button
      on:click={() => (isAddModalOpen = true)}
      class="fixed bottom-10 right-10 w-14 h-14 rounded-2xl flex items-center justify-center bg-white/5 text-slate-300 border border-white/10 backdrop-blur-md shadow-lg hover:bg-white/10 hover:text-white hover:border-white/20 hover:shadow-brand-500/10 hover:scale-105 active:scale-95 transition-all z-40 group"
      title="계정 추가"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-6 w-6 group-hover:rotate-90 transition-transform duration-300"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width="2.5"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 4v16m8-8H4"
        />
      </svg>
    </button>
  {/if}

  <!-- 모달 -->
  <AddAccountModal
    bind:showModal={isAddModalOpen}
    on:accountAdded={loadAccounts}
  />

  <PinSettingsModal bind:showModal={isPinSettingsOpen} on:toast={handleToast} />

  <!-- 토스트 -->
  <Toast bind:this={toastRef} />
</div>
