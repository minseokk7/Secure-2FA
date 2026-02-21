<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import ScreenCapture from "./ScreenCapture.svelte";

    const dispatch = createEventDispatcher();

    let issuer = "";
    let accountName = "";
    let secretKey = "";
    let errorMessage = "";
    let isSubmitting = false;

    export let showModal = false;

    async function handleSubmit() {
        errorMessage = "";

        if (!issuer || !accountName || !secretKey) {
            errorMessage = "모든 항목을 입력해주세요.";
            return;
        }

        // 공백 제거 및 대문자 변환
        const cleanSecret = secretKey.replace(/\s+/g, "").toUpperCase();

        isSubmitting = true;
        try {
            await invoke("add_account", {
                issuer: issuer.trim(),
                accountName: accountName.trim(),
                secretKey: cleanSecret,
            });

            // 폼 초기화 후 닫기
            issuer = "";
            accountName = "";
            secretKey = "";
            dispatch("accountAdded");
            closeModal();
        } catch (error) {
            errorMessage =
                typeof error === "string"
                    ? error
                    : "계정 추가에 실패했습니다. 시크릿 키를 확인해주세요.";
        } finally {
            isSubmitting = false;
        }
    }

    function closeModal() {
        showModal = false;
        errorMessage = "";
    }

    /** 배경 클릭 시 모달 닫기 */
    function handleBackdropClick(e: MouseEvent) {
        if (e.target === e.currentTarget) {
            closeModal();
        }
    }

    /** ESC 키로 모달 닫기 */
    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            closeModal();
        }
    }

    /** 화면 QR 스캔 */
    let isScanning = false;
    let showScreenCapture = false;
    let screenshotData = "";

    async function handleQrScan() {
        try {
            isScanning = true;
            errorMessage = "";
            showModal = false;

            const win = getCurrentWindow();

            // 1. 창 숨기기 & 대기 (데스크톱 상태 노출)
            await win.hide();
            await new Promise((r) => setTimeout(r, 400));

            // 2. 전체 스크린샷 촬영
            screenshotData = await invoke("take_screenshot");

            // 3. (1차) 자동 QR 감지 시도
            try {
                const uri: string = await invoke("decode_screenshot_auto");
                const info: {
                    issuer: string;
                    account_name: string;
                    secret: string;
                } = await invoke("parse_otpauth_uri", { uri });

                // 성공하면 바로 정보 채우고 복귀
                issuer = info.issuer;
                accountName = info.account_name;
                secretKey = info.secret;
                errorMessage = "";

                await win.show();
                await win.setFocus();
                showModal = true;
                isScanning = false;
                return;
            } catch (autoErr) {
                // 자동 감지 실패 → (2차) 수동 선택 오버레이로 넘어가기 위해 무시
                console.log("자동 감지 실패, 수동 선택 모드로 진입:", autoErr);
            }

            // 4. (2차) 풀스크린 오버레이 표시
            await win.setDecorations(false);
            await win.setAlwaysOnTop(true);
            await win.setFullscreen(true);
            await win.show();
            await win.setFocus();

            showScreenCapture = true;
        } catch (e: any) {
            await restoreWindow();
            showModal = true;
            errorMessage = typeof e === "string" ? e : "스크린샷 캡처 실패";
            isScanning = false;
        }
    }

    /** 창을 원래 상태로 복원 */
    async function restoreWindow() {
        const win = getCurrentWindow();
        await win.setFullscreen(false);
        await win.setAlwaysOnTop(false);
        await win.setDecorations(true);
        await win.center();
    }

    /** (수동) 영역 선택 완료 시 크롭 후 디코딩 */
    async function handleCaptured(
        e: CustomEvent<{ x: number; y: number; w: number; h: number }>,
    ) {
        showScreenCapture = false;
        const { x, y, w, h } = e.detail;

        try {
            const uri: string = await invoke("decode_screenshot_region", {
                x,
                y,
                w,
                h,
            });
            const info: {
                issuer: string;
                account_name: string;
                secret: string;
            } = await invoke("parse_otpauth_uri", { uri });

            issuer = info.issuer;
            accountName = info.account_name;
            secretKey = info.secret;
            errorMessage = "";
            showModal = true;
        } catch (err: any) {
            errorMessage =
                typeof err === "string" ? err : "QR 코드를 인식할 수 없습니다.";
            showModal = true;
        } finally {
            await restoreWindow();
            isScanning = false;
        }
    }

    /** 영역 선택 취소 */
    async function handleCaptureCancelled() {
        showScreenCapture = false;
        await restoreWindow();
        showModal = true;
        isScanning = false;
    }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if showModal}
    <!-- 배경 오버레이 -->
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
        style="background: rgba(0, 0, 0, 0.6); backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);"
        on:click={handleBackdropClick}
        on:keydown={handleKeydown}
        role="dialog"
        aria-modal="true"
        aria-label="계정 추가"
        tabindex="-1"
    >
        <!-- 모달 카드 -->
        <div
            class="glass-card w-full max-w-md p-7 shadow-2xl relative animate-slide-up"
            style="background: rgba(15, 23, 42, 0.85); border: 1px solid rgba(255,255,255,0.12);"
        >
            <!-- 닫기 버튼 -->
            <button
                on:click={closeModal}
                class="absolute top-4 right-4 text-slate-500 hover:text-white transition-colors"
                aria-label="닫기"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M6 18L18 6M6 6l12 12"
                    />
                </svg>
            </button>

            <!-- 제목 -->
            <div class="mb-6">
                <h2 class="text-xl font-bold text-white">새 계정 추가</h2>
                <p class="text-sm text-slate-400 mt-1">
                    2단계 인증 시크릿 키를 등록합니다
                </p>
            </div>

            <!-- QR 스캔 버튼 -->
            <button
                type="button"
                on:click={handleQrScan}
                disabled={isScanning}
                class="w-full mb-5 flex items-center justify-center gap-2 px-4 py-3 rounded-xl text-sm font-medium transition-all"
                style="background: rgba(99, 102, 241, 0.1); border: 1px dashed rgba(129, 140, 248, 0.3); color: rgb(165, 180, 252);"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="1.5"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M3 9V5a2 2 0 012-2h4M15 3h4a2 2 0 012 2v4M21 15v4a2 2 0 01-2 2h-4M9 21H5a2 2 0 01-2-2v-4"
                    />
                    <rect
                        x="7"
                        y="7"
                        width="10"
                        height="10"
                        rx="1"
                        stroke-width="1.5"
                    />
                </svg>
                {#if isScanning}
                    화면 캐처 중...
                {:else}
                    화면에서 QR 코드 스캔
                {/if}
            </button>

            <div class="relative mb-5">
                <div class="absolute inset-0 flex items-center">
                    <div
                        class="w-full"
                        style="border-top: 1px solid rgba(255,255,255,0.08);"
                    ></div>
                </div>
                <div class="relative flex justify-center">
                    <span
                        class="px-3 text-xs text-slate-500"
                        style="background: rgba(15, 23, 42, 0.85);"
                        >또는 직접 입력</span
                    >
                </div>
            </div>

            <form on:submit|preventDefault={handleSubmit} class="space-y-4">
                <!-- 에러 메시지 -->
                {#if errorMessage}
                    <div
                        class="bg-red-500/10 text-red-400 p-3 rounded-lg text-sm border border-red-500/20 animate-fade-in"
                    >
                        {errorMessage}
                    </div>
                {/if}

                <!-- 발급자 -->
                <div>
                    <label
                        for="issuer"
                        class="block text-sm font-medium text-slate-300 mb-1.5"
                    >
                        서비스 (예: Google, GitHub)
                    </label>
                    <input
                        type="text"
                        id="issuer"
                        bind:value={issuer}
                        placeholder="서비스 이름"
                        class="w-full px-4 py-2.5 glass-input text-sm"
                        required
                    />
                </div>

                <!-- 계정명 -->
                <div>
                    <label
                        for="account"
                        class="block text-sm font-medium text-slate-300 mb-1.5"
                    >
                        계정명
                    </label>
                    <input
                        type="text"
                        id="account"
                        bind:value={accountName}
                        placeholder="user@example.com"
                        class="w-full px-4 py-2.5 glass-input text-sm"
                        required
                    />
                </div>

                <!-- 시크릿 키 -->
                <div>
                    <label
                        for="secret"
                        class="block text-sm font-medium text-slate-300 mb-1.5"
                    >
                        시크릿 키
                    </label>
                    <input
                        type="text"
                        id="secret"
                        bind:value={secretKey}
                        placeholder="JBSWY3DPEHPK3PXP"
                        class="w-full px-4 py-2.5 glass-input text-sm font-mono uppercase tracking-wider"
                        required
                    />
                    <p class="text-xs text-slate-500 mt-2">
                        서비스에서 제공한 설정 키를 입력하세요 (공백은 자동
                        제거됩니다).
                    </p>
                </div>

                <!-- 버튼 영역 -->
                <div class="pt-4 flex justify-end gap-2">
                    <button
                        type="button"
                        on:click={closeModal}
                        class="px-5 py-2.5 text-slate-400 hover:text-white hover:bg-white/5 rounded-lg transition-all text-sm"
                    >
                        취소
                    </button>
                    <button
                        type="submit"
                        disabled={isSubmitting}
                        class="px-5 py-2.5 bg-brand-600 hover:bg-brand-500 text-white rounded-lg transition-all font-medium text-sm flex items-center justify-center min-w-[100px] disabled:opacity-40 disabled:cursor-not-allowed shadow-lg shadow-brand-600/20"
                    >
                        {#if isSubmitting}
                            <svg
                                class="animate-spin h-4 w-4 text-white"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                            >
                                <circle
                                    class="opacity-25"
                                    cx="12"
                                    cy="12"
                                    r="10"
                                    stroke="currentColor"
                                    stroke-width="4"
                                ></circle>
                                <path
                                    class="opacity-75"
                                    fill="currentColor"
                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                ></path>
                            </svg>
                        {:else}
                            키 저장
                        {/if}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}

{#if showScreenCapture}
    <ScreenCapture
        screenshotSrc={screenshotData}
        on:captured={handleCaptured}
        on:cancelled={handleCaptureCancelled}
    />
{/if}
