<script lang="ts">
  import { onDestroy } from "svelte";

  /** 토스트 메시지 타입 */
  type ToastType = "success" | "error";

  /** 토스트 항목 */
  type ToastItem = {
    id: number;
    message: string;
    type: ToastType;
    visible: boolean;
  };

  let toasts: ToastItem[] = [];
  let nextId = 0;

  /**
   * 토스트 알림을 표시합니다.
   * @param message 표시할 메시지
   * @param type 토스트 유형 (success | error)
   */
  export function show(message: string, type: ToastType = "success") {
    const id = nextId++;
    const toast: ToastItem = { id, message, type, visible: true };
    toasts = [...toasts, toast];

    // 2초 후 페이드아웃 시작
    setTimeout(() => {
      toasts = toasts.map((t) => (t.id === id ? { ...t, visible: false } : t));
      // 페이드아웃 애니메이션 후 제거
      setTimeout(() => {
        toasts = toasts.filter((t) => t.id !== id);
      }, 300);
    }, 2000);
  }

  onDestroy(() => {
    toasts = [];
  });
</script>

{#if toasts.length > 0}
  <div class="fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] flex flex-col items-center gap-2 pointer-events-none">
    {#each toasts as toast (toast.id)}
      <div
        class="pointer-events-auto px-5 py-3 rounded-xl font-medium text-sm shadow-lg transition-all duration-300"
        class:opacity-0={!toast.visible}
        class:translate-y-2={!toast.visible}
        class:opacity-100={toast.visible}
        class:translate-y-0={toast.visible}
        style="
          background: {toast.type === 'success'
            ? 'rgba(16, 185, 129, 0.15)'
            : 'rgba(239, 68, 68, 0.15)'};
          backdrop-filter: blur(16px);
          -webkit-backdrop-filter: blur(16px);
          border: 1px solid {toast.type === 'success'
            ? 'rgba(52, 211, 153, 0.3)'
            : 'rgba(248, 113, 113, 0.3)'};
          color: {toast.type === 'success' ? '#6ee7b7' : '#fca5a5'};
        "
      >
        <span class="inline-flex items-center gap-2">
          {#if toast.type === "success"}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
          {/if}
          {toast.message}
        </span>
      </div>
    {/each}
  </div>
{/if}
