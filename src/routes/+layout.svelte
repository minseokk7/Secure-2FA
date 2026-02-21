<script>
    import Titlebar from "$lib/components/Titlebar.svelte";
    import "../app.css";
</script>

<div class="h-screen w-screen flex flex-col overflow-hidden bg-[#030712]">
    <Titlebar />
    <div class="flex-1 w-full pt-16 flex flex-col min-h-0 relative">
        <div
            class="flex-1 w-full overflow-y-auto overflow-x-hidden relative"
            id="main-scroll-view"
        >
            <slot />
        </div>
    </div>
</div>
