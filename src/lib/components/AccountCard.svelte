<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ServiceIcon from "./ServiceIcon.svelte";

  const dispatch = createEventDispatcher();

  export let account: {
    id: number;
    issuer: string;
    account_name: string;
    encrypted_secret: number[];
    secret_nonce: number[];
  };

  let currentCode = "------";
  let remainingSeconds = 30;
  let progressPercentage = 100;
  let intervalId: ReturnType<typeof setInterval>;
  let copied = false;
  /** 인라인 삭제 확인 모드 */
  let confirmingDelete = false;
  /** 현재 TOTP 주기 번호 (30초 단위) - 주기 변경 감지용 */
  let lastTimeStep = -1;
  /** 주기 전환 시 되감기 애니메이션 방지용 */
  let noTransition = false;

  async function fetchOtp() {
    try {
      const response: { code: string; remaining_seconds: number } =
        await invoke("get_current_otp", {
          encryptedSecret: account.encrypted_secret,
          nonce: account.secret_nonce,
        });
      currentCode = response.code;
    } catch (_e) {
      currentCode = "오류";
    }
  }

  /** 매 틱마다 시스템 시간 기반으로 남은 시간 갱신 및 주기 변경 시 OTP 재요청 */
  function tick() {
    const now = Math.floor(Date.now() / 1000);
    const currentTimeStep = Math.floor(now / 30);
    remainingSeconds = 30 - (now % 30);

    // TOTP 30초 주기가 변경되면 되감기 없이 즉시 리셋
    if (currentTimeStep !== lastTimeStep) {
      if (lastTimeStep !== -1) {
        // 트랜지션 끄고 즉시 100%로 점프
        noTransition = true;
        progressPercentage = 100;
        // 다음 프레임에 트랜지션 켜고 0%로 부드럽게 애니메이션
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            noTransition = false;
            progressPercentage = 0;
          });
        });
      } else {
        // 최초 로드: 남은 시간에 맞춰 중간부터 시작
        progressPercentage = 0;
      }
      lastTimeStep = currentTimeStep;
      fetchOtp();
    }
  }

  onMount(() => {
    tick();
    intervalId = setInterval(tick, 1000); // 초 표시 갱신용
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });

  async function copyToClipboard() {
    if (currentCode === "------" || currentCode === "오류") return;
    try {
      await navigator.clipboard.writeText(currentCode);
      copied = true;
      dispatch("toast", {
        message: "코드가 클립보드에 복사되었습니다",
        type: "success",
      });
      setTimeout(() => (copied = false), 2000);
    } catch (_err) {
      dispatch("toast", { message: "복사에 실패했습니다", type: "error" });
    }
  }

  /** 삭제 확인 시작 */
  function requestDelete() {
    confirmingDelete = true;
    // 3초 후 자동으로 확인 모드 해제
    setTimeout(() => {
      confirmingDelete = false;
    }, 3000);
  }

  /** 실제 삭제 수행 */
  async function confirmDelete() {
    try {
      await invoke("delete_account", { id: account.id });
      dispatch("deleted");
      dispatch("toast", {
        message: `${account.issuer} 계정이 삭제되었습니다`,
        type: "success",
      });
    } catch (_e) {
      dispatch("toast", { message: "삭제에 실패했습니다", type: "error" });
    }
  }
  /** 인라인 편집 모드 */
  let isEditing = false;
  let editIssuer = "";
  let editAccountName = "";

  /** 편집 모드 시작 */
  function startEdit() {
    editIssuer = account.issuer;
    editAccountName = account.account_name;
    isEditing = true;
  }

  /** 편집 저장 */
  async function saveEdit() {
    try {
      await invoke("update_account", {
        id: account.id,
        issuer: editIssuer.trim(),
        accountName: editAccountName.trim(),
      });
      dispatch("toast", {
        message: `${editIssuer.trim()} 계정이 수정되었습니다`,
        type: "success",
      });
      dispatch("deleted"); // 계정 목록 새로고침
      isEditing = false;
    } catch (_e) {
      dispatch("toast", { message: "수정에 실패했습니다", type: "error" });
    }
  }

  /** 편집 취소 */
  function cancelEdit() {
    isEditing = false;
  }
</script>

<div
  class="glass-card p-5 flex flex-col justify-between relative group animate-fade-in"
>
  <!-- 삭제 버튼 영역 -->
  <div class="absolute top-3 right-3">
    {#if confirmingDelete}
      <div class="flex items-center gap-1.5 animate-fade-in">
        <button
          on:click={() => (confirmingDelete = false)}
          class="text-xs px-2 py-1 rounded-md text-slate-400 hover:text-white hover:bg-white/10 transition-all"
          title="취소"
        >
          취소
        </button>
        <button
          on:click={confirmDelete}
          class="text-xs px-2.5 py-1 rounded-md bg-red-500/20 text-red-400 hover:bg-red-500/40 hover:text-red-300 transition-all font-medium"
          title="삭제 확인"
        >
          삭제
        </button>
      </div>
    {:else}
      <button
        on:click={requestDelete}
        class="text-slate-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all duration-200"
        title="계정 삭제"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4.5 w-4.5"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            fill-rule="evenodd"
            d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
            clip-rule="evenodd"
          />
        </svg>
      </button>
      <!-- 편집 버튼 -->
      <button
        on:click={startEdit}
        class="text-slate-500 hover:text-brand-400 opacity-0 group-hover:opacity-100 transition-all duration-200"
        title="계정 편집"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4 w-4"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"
          />
        </svg>
      </button>
    {/if}
  </div>

  <!-- 계정 정보 -->
  <div class="mb-4 flex items-center gap-3">
    <ServiceIcon issuer={isEditing ? editIssuer : account.issuer} size={36} />
    <div class="min-w-0 flex-1">
      {#if isEditing}
        <input
          type="text"
          bind:value={editIssuer}
          class="w-full bg-white/5 border border-white/10 rounded-md px-2 py-1 text-white text-sm font-bold focus:outline-none focus:border-brand-400 mb-1"
          placeholder="서비스 이름"
        />
        <input
          type="text"
          bind:value={editAccountName}
          class="w-full bg-white/5 border border-white/10 rounded-md px-2 py-1 text-slate-300 text-xs focus:outline-none focus:border-brand-400"
          placeholder="계정명"
        />
        <div class="flex gap-1.5 mt-1.5">
          <button
            on:click={saveEdit}
            class="text-xs px-2 py-0.5 rounded bg-brand-600/30 text-brand-300 hover:bg-brand-600/50 transition-all"
          >
            저장
          </button>
          <button
            on:click={cancelEdit}
            class="text-xs px-2 py-0.5 rounded text-slate-400 hover:text-white hover:bg-white/10 transition-all"
          >
            취소
          </button>
        </div>
      {:else}
        <h3 class="text-lg font-bold text-white truncate pr-16">
          {account.issuer}
        </h3>
        <p class="text-sm text-slate-400 truncate">{account.account_name}</p>
      {/if}
    </div>
  </div>

  <!-- OTP 코드 + 타이머 -->
  <div class="flex items-center justify-between mt-2">
    <button
      on:click={copyToClipboard}
      class="flex flex-col items-start focus:outline-none group/code cursor-pointer"
      title="클릭하여 복사"
    >
      <div class="flex items-center gap-3">
        <span
          class="text-3xl font-mono font-black tracking-widest transition-colors"
          class:text-brand-400={!copied}
          class:text-emerald-400={copied}
        >
          {currentCode.slice(0, 3)}
          {currentCode.slice(3, 6)}
        </span>
        {#if copied}
          <span
            class="text-xs font-semibold text-emerald-400 animate-pulse bg-emerald-500/10 px-2 py-1 rounded-md"
          >
            복사됨!
          </span>
        {/if}
      </div>
    </button>

    <!-- 프로그레스 링 -->
    <div
      class="relative w-10 h-10 flex items-center justify-center flex-shrink-0"
    >
      <svg class="w-10 h-10" style="transform: rotate(-90deg);">
        <circle
          cx="20"
          cy="20"
          r="16"
          stroke="currentColor"
          stroke-width="3"
          fill="transparent"
          class="text-white/5"
        />
        <circle
          cx="20"
          cy="20"
          r="16"
          stroke="currentColor"
          stroke-width="3"
          fill="transparent"
          stroke-dasharray="100.53"
          stroke-dashoffset={100.53 * (progressPercentage / 100)}
          stroke-linecap="round"
          class={remainingSeconds < 5 ? "text-red-400" : "text-brand-400"}
          style="transition: {noTransition
            ? 'none'
            : `stroke-dashoffset ${remainingSeconds}s linear`};"
        />
      </svg>
      <span
        class="absolute text-xs font-bold {remainingSeconds < 5
          ? 'text-red-400'
          : 'text-slate-300'}"
      >
        {remainingSeconds}
      </span>
    </div>
  </div>
</div>
