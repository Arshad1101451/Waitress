<!-- Fancy gradient button — port of theme.fancy_button(). -->
<script>
  import { pressable } from "../actions.js";

  export let label = "";
  export let icon = null; // optional inline SVG snippet string, or leave null
  export let gradient = "var(--grad-brand)";
  export let bgcolor = null;
  export let width = null;
  export let height = 48;
  export let expand = false;
  export let textColor = "var(--white)";
  export let size = 15;
  export let tooltip = "";
  export let disabled = false;
  export let onClick = () => {};
</script>

<button
  class="fancy-btn"
  class:expand
  style="
    --grad: {bgcolor ? 'none' : gradient};
    --bg: {bgcolor || 'transparent'};
    --w: {width ? width + 'px' : 'auto'};
    --h: {height}px;
    --tc: {textColor};
    --fs: {size}px;
  "
  title={tooltip}
  disabled={disabled}
  use:pressable
  on:click={onClick}
>
  {#if icon}
    <span class="icon">{@html icon}</span>
  {/if}
  {#if label}
    <span class="label">{label}</span>
  {/if}
  <slot />
</button>

<style>
  .fancy-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: var(--w);
    height: var(--h);
    padding: 0 22px;
    border: none;
    border-radius: 14px;
    background: var(--grad);
    background-color: var(--bg);
    box-shadow: var(--shadow-soft);
    color: var(--tc);
  }
  .fancy-btn.expand {
    width: 100%;
  }
  .fancy-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .label {
    font-style: italic;
    font-weight: 700;
    font-size: var(--fs);
    font-family: var(--font-body);
    color: var(--tc);
    white-space: nowrap;
  }
  .icon {
    display: inline-flex;
    color: var(--tc);
  }
</style>
