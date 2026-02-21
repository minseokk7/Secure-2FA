<script context="module">
    import { scale } from "svelte/transition";
</script>

<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { fade, slide } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    import PinPad from "./PinPad.svelte";

    export let showModal = false;

    const dispatch = createEventDispatcher();
    let toastRef: any;
    let pinPadRef: PinPad;

    let mode: "select" | "setup" | "remove" = "select";
    let hasPin = false;

    $: if (showModal) {
        checkPinStatus();
        mode = "select";
    }

    async function checkPinStatus() {
        try {
            hasPin = await invoke("has_pin");
        } catch (e) {
            hasPin = false;
        }
    }

    function close() {
        showModal = false;
        setTimeout(() => {
            mode = "select";
        }, 300);
    }

    async function handleSetup(e: CustomEvent<{ pin: string }>) {
        try {
            await invoke("set_pin", { pin: e.detail.pin });
            dispatch("toast", {
                message: "PIN 번호가 설정되었습니다",
                type: "success",
            });
            hasPin = true;
            close();
        } catch (err: any) {
            pinPadRef?.triggerError(err.toString());
        }
    }

    async function handleRemove(e: CustomEvent<{ pin: string }>) {
        try {
            await invoke("remove_pin", { currentPin: e.detail.pin });
            dispatch("toast", {
                message: "앱 잠금이 해제되었습니다",
                type: "success",
            });
            hasPin = false;
            close();
        } catch (err: any) {
            pinPadRef?.triggerError("현재 PIN이 일치하지 않습니다");
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape" && showModal) {
            if (mode !== "select") {
                mode = "select";
            } else {
                close();
            }
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if showModal}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 animate-fade-in"
        style="background: rgba(15, 23, 42, 0.85); backdrop-filter: blur(8px);"
        role="dialog"
        aria-modal="true"
        on:click|self={mode === "select" ? close : () => {}}
    >
        <div
            transition:scale={{ duration: 250, start: 0.95 }}
            class="glass-panel w-full max-w-md p-6 sm:p-8 relative flex flex-col max-h-[90vh] overflow-y-auto"
            style="border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);"
        >
            <!-- 닫기 버튼 또는 뒤로 가기 버튼 -->
            <button
                on:click={mode === "select" ? close : () => (mode = "select")}
                class="absolute top-4 right-4 text-slate-400 hover:text-white p-2 rounded-full hover:bg-white/10 transition-colors z-10"
            >
                {#if mode === "select"}
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                {:else}
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M10 19l-7-7m0 0l7-7m-7 7h18"
                        />
                    </svg>
                {/if}
            </button>

            {#if mode === "select"}
                <div class="text-center mb-8 pt-4">
                    <div
                        class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center bg-slate-800/50 border border-slate-700"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-8 w-8 text-slate-300"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            stroke-width="1.5"
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
                    </div>
                    <h2 class="text-2xl font-bold text-white mb-2">
                        보안 설정
                    </h2>
                    <p class="text-sm text-slate-400">
                        앱 시작 시 암호 입력을 요구하여 보안을 강화합니다.
                    </p>
                </div>

                <div class="space-y-4">
                    <div
                        class="glass-panel p-5 rounded-xl border border-white/5 flex items-center justify-between"
                    >
                        <div>
                            <h3 class="text-white font-medium mb-1">
                                앱 잠금 활성화
                            </h3>
                            <p class="text-sm text-slate-400">
                                {hasPin
                                    ? "현재 앱이 PIN으로 보호되고 있습니다."
                                    : "잠금이 해제되어 있습니다."}
                            </p>
                        </div>

                        {#if hasPin}
                            <span
                                class="px-3 py-1 bg-emerald-500/20 text-emerald-400 text-xs font-bold rounded-full border border-emerald-500/30"
                            >
                                사용 중
                            </span>
                        {:else}
                            <span
                                class="px-3 py-1 bg-slate-700/50 text-slate-400 text-xs font-bold rounded-full border border-slate-600"
                            >
                                사용 안 함
                            </span>
                        {/if}
                    </div>

                    {#if hasPin}
                        <button
                            on:click={() => (mode = "setup")}
                            class="w-full text-left p-5 rounded-xl glass hover:bg-white/10 transition-colors border border-white/5 flex items-center gap-4 group"
                        >
                            <div
                                class="w-10 h-10 rounded-lg bg-brand-500/20 text-brand-400 flex items-center justify-center group-hover:bg-brand-500 group-hover:text-white transition-colors"
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
                                        d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
                                    />
                                </svg>
                            </div>
                            <div class="flex-1">
                                <h4 class="text-white font-medium">PIN 변경</h4>
                                <p class="text-xs text-slate-400">
                                    등록된 숫자 암호를 새롭게 변경합니다.
                                </p>
                            </div>
                        </button>

                        <button
                            on:click={() => (mode = "remove")}
                            class="w-full text-left p-5 rounded-xl glass hover:bg-red-500/10 transition-colors border border-white/5 flex items-center gap-4 group"
                        >
                            <div
                                class="w-10 h-10 rounded-lg bg-red-500/20 text-red-400 flex items-center justify-center group-hover:bg-red-500 group-hover:text-white transition-colors"
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
                                        d="M13 7a4 4 0 11-8 0 4 4 0 018 0zM9 14a6 6 0 00-6 6v1h12v-1a6 6 0 00-6-6zM21 12h-6"
                                    />
                                </svg>
                            </div>
                            <div class="flex-1">
                                <h4
                                    class="text-white font-medium group-hover:text-red-400 transition-colors"
                                >
                                    잠금 끄기
                                </h4>
                                <p
                                    class="text-xs text-slate-400 group-hover:text-red-300 transition-colors"
                                >
                                    기존 PIN을 확인한 후 잠금을 해제합니다.
                                </p>
                            </div>
                        </button>
                    {:else}
                        <button
                            on:click={() => (mode = "setup")}
                            class="w-full text-center py-4 rounded-xl btn-brand text-white font-bold tracking-wide mt-4"
                        >
                            새 PIN 설정하기
                        </button>
                    {/if}
                </div>
            {:else if mode === "setup"}
                <div class="py-4">
                    <PinPad
                        bind:this={pinPadRef}
                        mode="setup"
                        on:setup={handleSetup}
                    />
                </div>
            {:else}
                <div class="py-4">
                    <PinPad
                        bind:this={pinPadRef}
                        mode="remove"
                        on:submit={handleRemove}
                    />
                </div>
            {/if}
        </div>
    </div>
{/if}
