<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";

    const appWindow = getCurrentWindow();

    function minimize() {
        appWindow.minimize();
    }

    function toggleMaximize() {
        appWindow.toggleMaximize();
    }

    function close() {
        appWindow.close();
    }
</script>

<!-- Drag Region Background -->
<div data-tauri-drag-region class="fixed top-0 left-0 right-0 h-16 z-40"></div>

<!-- Window Controls Foreground -->
<div
    class="fixed top-0 right-0 h-16 flex items-start justify-end z-50 p-4 pointer-events-none select-none"
>
    <div class="flex items-center gap-3 h-8 pointer-events-auto">
        <!-- Minimize -->
        <button
            on:click={minimize}
            class="h-full w-8 rounded-full flex items-center justify-center bg-white/5 text-slate-400 hover:bg-white/10 hover:text-white transition-colors border border-white/10 shadow-sm"
            title="최소화"
        >
            <svg
                width="0.8em"
                height="0.8em"
                viewBox="0 0 10 1"
                fill="currentColor"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M0 0h10v1H0z" />
            </svg>
        </button>

        <!-- Maximize -->
        <button
            on:click={toggleMaximize}
            class="h-full w-8 rounded-full flex items-center justify-center bg-white/5 text-slate-400 hover:bg-white/10 hover:text-white transition-colors border border-white/10 shadow-sm"
            title="최대화"
        >
            <svg
                width="0.8em"
                height="0.8em"
                viewBox="0 0 10 10"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M1 1h8v8H1V1z"
                    stroke="currentColor"
                    stroke-width="1"
                />
            </svg>
        </button>

        <!-- Close -->
        <button
            on:click={close}
            class="h-full w-8 rounded-full flex items-center justify-center bg-white/5 text-slate-400 hover:bg-red-500 hover:border-red-500 hover:text-white transition-colors border border-white/10 shadow-sm"
            title="닫기"
        >
            <svg
                width="0.8em"
                height="0.8em"
                viewBox="0 0 10 10"
                fill="currentColor"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M5.707 5l3.646-3.646-.707-.707L5 4.293 1.354.646l-.707.707L4.293 5 .646 8.646l.707.707L5 5.707l3.646 3.646.707-.707L5.707 5z"
                />
            </svg>
        </button>
    </div>
</div>
