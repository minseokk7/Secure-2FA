<script lang="ts">
    /**
     * ScreenCapture — 스크린샷 배경 위에서 마우스 드래그로 QR 코드 영역을 선택합니다.
     * 다른 캡처 프로그램(ShareX, Snipping Tool)과 동일한 방식입니다.
     */
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    /** 스크린샷 base64 데이터 URI */
    export let screenshotSrc: string;

    let isDrawing = false;
    let startX = 0;
    let startY = 0;
    let currentX = 0;
    let currentY = 0;

    $: selectionLeft = Math.min(startX, currentX);
    $: selectionTop = Math.min(startY, currentY);
    $: selectionWidth = Math.abs(currentX - startX);
    $: selectionHeight = Math.abs(currentY - startY);

    function handleMouseDown(e: MouseEvent) {
        isDrawing = true;
        startX = e.clientX;
        startY = e.clientY;
        currentX = e.clientX;
        currentY = e.clientY;
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDrawing) return;
        currentX = e.clientX;
        currentY = e.clientY;
    }

    function handleMouseUp(_e: MouseEvent) {
        if (!isDrawing) return;
        isDrawing = false;

        if (selectionWidth < 30 || selectionHeight < 30) return;

        const dpr = window.devicePixelRatio || 1;
        dispatch("captured", {
            x: Math.round(selectionLeft * dpr),
            y: Math.round(selectionTop * dpr),
            w: Math.round(selectionWidth * dpr),
            h: Math.round(selectionHeight * dpr),
        });
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            dispatch("cancelled");
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    class="capture-overlay"
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseup={handleMouseUp}
    role="application"
    aria-label="QR 코드 영역 선택"
    tabindex="-1"
>
    <!-- 스크린샷 배경 (데스크톱처럼 보임) -->
    <img src={screenshotSrc} alt="" class="screenshot-bg" draggable="false" />

    <!-- 반투명 덮개 -->
    <div class="overlay-dim"></div>

    <!-- 선택 영역 -->
    {#if isDrawing || selectionWidth > 0}
        <div
            class="selection-box"
            style="left:{selectionLeft}px; top:{selectionTop}px; width:{selectionWidth}px; height:{selectionHeight}px;"
        >
            <div class="corner tl"></div>
            <div class="corner tr"></div>
            <div class="corner bl"></div>
            <div class="corner br"></div>
        </div>

        {#if isDrawing}
            <div
                class="size-label"
                style="left:{selectionLeft +
                    selectionWidth / 2}px; top:{selectionTop +
                    selectionHeight +
                    8}px;"
            >
                {selectionWidth} × {selectionHeight}
            </div>
        {/if}
    {/if}

    <!-- 안내 텍스트 -->
    {#if !isDrawing && selectionWidth === 0}
        <div class="guide-text">
            <div class="guide-box">
                <p class="guide-title">🔍 자동 인식에 실패했습니다</p>
                <p class="guide-sub">QR 코드 영역을 드래그로 선택해 주세요</p>
                <p class="guide-hint">ESC = 취소</p>
            </div>
        </div>
    {/if}
</div>

<style>
    .capture-overlay {
        position: fixed;
        inset: 0;
        z-index: 9999;
        cursor: crosshair;
        user-select: none;
    }

    .screenshot-bg {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
        pointer-events: none;
    }

    .overlay-dim {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        pointer-events: none;
    }

    .selection-box {
        position: absolute;
        border: 2px solid rgba(99, 102, 241, 0.9);
        background: transparent;
        box-shadow:
            0 0 0 9999px rgba(0, 0, 0, 0.35),
            0 0 15px rgba(99, 102, 241, 0.3);
        z-index: 10;
        pointer-events: none;
    }

    .corner {
        position: absolute;
        width: 14px;
        height: 14px;
        border-color: rgb(129, 140, 248);
        border-style: solid;
        border-width: 0;
    }
    .corner.tl {
        top: -2px;
        left: -2px;
        border-top-width: 3px;
        border-left-width: 3px;
    }
    .corner.tr {
        top: -2px;
        right: -2px;
        border-top-width: 3px;
        border-right-width: 3px;
    }
    .corner.bl {
        bottom: -2px;
        left: -2px;
        border-bottom-width: 3px;
        border-left-width: 3px;
    }
    .corner.br {
        bottom: -2px;
        right: -2px;
        border-bottom-width: 3px;
        border-right-width: 3px;
    }

    .size-label {
        position: absolute;
        transform: translateX(-50%);
        background: rgba(15, 23, 42, 0.85);
        color: rgb(165, 180, 252);
        padding: 2px 8px;
        border-radius: 4px;
        font-size: 11px;
        font-family: monospace;
        pointer-events: none;
        z-index: 11;
        white-space: nowrap;
    }

    .guide-text {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        pointer-events: none;
        z-index: 11;
    }

    .guide-box {
        background: rgba(15, 23, 42, 0.9);
        padding: 20px 32px;
        border-radius: 14px;
        border: 1px solid rgba(99, 102, 241, 0.3);
        text-align: center;
    }

    .guide-title {
        font-size: 16px;
        font-weight: 700;
        color: white;
        margin: 0;
    }

    .guide-sub {
        font-size: 13px;
        color: rgb(165, 180, 252);
        margin: 6px 0 0;
    }

    .guide-hint {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.4);
        margin: 10px 0 0;
    }
</style>
