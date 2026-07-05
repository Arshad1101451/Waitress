<!-- Right-aligned Back / Forward / Home / Exit dock. Port of
     chrome.nav_dock(). -->
<script>
  import Button from "./Button.svelte";

  export let canBack = true;
  export let canForward = true;
  export let onBack = () => {};
  export let onForward = () => {};
  export let onHome = () => {};
  export let onExit = () => {};
</script>

<div class="dock">
  <Button label="Back" width={100} height={52} size={14} gradient="var(--grad-cool)"
    disabled={!canBack} onClick={onBack} tooltip="Back" />
  <Button label="Forward" width={100} height={52} size={14} gradient="var(--grad-cool)"
    disabled={!canForward} onClick={onForward} tooltip="Forward" />
  <Button label="Home" width={100} height={52} size={14} gradient="var(--grad-brand)"
    onClick={onHome} tooltip="Home" />
  <Button label="Exit" width={100} height={52} size={14} gradient="var(--grad-sunny)"
    onClick={onExit} tooltip="Exit" />
</div>

<style>
  .dock {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 10px 16px;
  }
</style>
