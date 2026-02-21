<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fade, slide } from "svelte/transition";

    export let mode: "verify" | "setup" | "remove" = "verify";
    export let errorMsg: string | null = null;
    export let successMsg: string | null = null;

    const dispatch = createEventDispatcher();

    let pin = "";
    let confirmPin = "";
    let isConfirming = false;
    let isShaking = false;

    const MAX_PIN_LENGTH = 4;
    const MIN_PIN_LENGTH = 4;

    $: title =
        mode === "verify"
            ? "PIN 입력"
            : mode === "remove"
              ? "PIN 제거"
              : isConfirming
                ? "PIN 확인"
                : "새 PIN 등록";

    $: subtitle =
        mode === "verify"
            ? "앱 잠금을 해제하려면 키보드로 PIN을 입력하세요"
            : mode === "remove"
              ? "현재 PIN을 키보드로 입력하여 잠금을 해제합니다"
              : isConfirming
                ? "설정한 PIN을 한 번 더 키보드로 입력하세요"
                : "앱을 보호할 4자리 숫자를 키보드로 입력하세요";

    function shake() {
        isShaking = true;
        setTimeout(() => {
            isShaking = false;
        }, 400);
    }

    export function triggerError(msg: string) {
        errorMsg = msg;
        shake();
        pin = "";
        if (mode === "setup") {
            isConfirming = false;
            confirmPin = "";
        }
    }

    function handleInput(num: number) {
        if (pin.length < MAX_PIN_LENGTH) {
            pin += num.toString();
            errorMsg = null;

            if (pin.length === MAX_PIN_LENGTH) {
                setTimeout(handleSubmit, 200);
            }
        }
    }

    function handleDelete() {
        if (pin.length > 0) {
            pin = pin.slice(0, -1);
            errorMsg = null;
        }
    }

    function handleSubmit() {
        if (pin.length < MIN_PIN_LENGTH) {
            triggerError("PIN은 4자리여야 합니다");
            return;
        }

        if (mode === "verify" || mode === "remove") {
            dispatch("submit", { pin });
        } else if (mode === "setup") {
            if (!isConfirming) {
                confirmPin = pin;
                pin = "";
                isConfirming = true;
            } else {
                if (pin === confirmPin) {
                    dispatch("setup", { pin });
                } else {
                    triggerError("입력한 PIN이 일치하지 않습니다");
                }
            }
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (/^[0-9]$/.test(e.key)) {
            handleInput(parseInt(e.key, 10));
        } else if (e.key === "Backspace") {
            handleDelete();
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="flex flex-col items-center justify-center w-full max-w-sm mx-auto animate-fade-in"
>
    <div class="text-center mb-8">
        <!-- 자물쇠 아이콘 -->
        <div
            class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center"
            style="background: linear-gradient(135deg, rgba(79, 70, 229, 0.2) 0%, rgba(99, 102, 241, 0.1) 100%); border: 1px solid rgba(129, 140, 248, 0.2);"
        >
            {#if mode === "setup"}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-8 w-8 text-brand-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="1.5"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                    />
                </svg>
            {:else if mode === "remove"}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-8 w-8 text-red-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="1.5"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"
                    />
                </svg>
            {:else}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-8 w-8 text-brand-400"
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
            {/if}
        </div>

        <h2 class="text-2xl font-bold text-white mb-2">{title}</h2>
        <p class="text-sm text-slate-400">{subtitle}</p>
    </div>

    <!-- 인디케이터 박스 -->
    <div class="flex gap-4 mb-4" class:animate-shake={isShaking}>
        {#each Array(MAX_PIN_LENGTH) as _, i}
            <div
                class="w-4 h-4 rounded-full border-2 transition-all duration-200"
                class:bg-brand-500={i < pin.length}
                class:border-brand-500={i < pin.length}
                class:border-slate-600={i >= pin.length && i < MIN_PIN_LENGTH}
                class:border-slate-800={i >= MIN_PIN_LENGTH && i >= pin.length}
                class:opacity-50={i >= MIN_PIN_LENGTH && i >= pin.length}
            ></div>
        {/each}
    </div>

    <!-- 에러/성공 메시지 -->
    <div class="h-6 mb-6 w-full text-center">
        {#if errorMsg}
            <p transition:slide class="text-sm font-medium text-red-400">
                {errorMsg}
            </p>
        {:else if successMsg}
            <p transition:slide class="text-sm font-medium text-emerald-400">
                {successMsg}
            </p>
        {/if}
    </div>

    <!-- 초기화 버튼 (새 PIN 등록 확인 시 표출) -->
    {#if mode === "setup" && isConfirming}
        <div class="mt-4 text-center">
            <button
                class="text-sm text-slate-400 hover:text-white transition-colors px-4 py-2"
                on:click={() => {
                    isConfirming = false;
                    pin = "";
                    confirmPin = "";
                    errorMsg = null;
                }}
            >
                초기화
            </button>
        </div>
    {/if}
</div>

<style>
    @keyframes shake {
        0%,
        100% {
            transform: translateX(0);
        }
        20%,
        60% {
            transform: translateX(-5px);
        }
        40%,
        80% {
            transform: translateX(5px);
        }
    }
    .animate-shake {
        animation: shake 0.4s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
    }
</style>
