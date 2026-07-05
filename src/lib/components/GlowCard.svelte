<!-- White surface, rounded, soft shadow, optional hover-lift/click. Port of
     theme.glow_card(). -->
<script>
  export let padding = 16;
  export let radius = 18;
  export let hover = false;
  export let onClick = null;
</script>

<div
  class="glow-card"
  class:clickable={!!onClick}
  style="padding: {padding}px; border-radius: {radius}px;"
  class:hoverable={hover || !!onClick}
  on:click={onClick}
  on:keypress={() => {}}
  role={onClick ? "button" : undefined}
  tabindex={onClick ? 0 : undefined}
>
  <slot />
</div>

<style>
  .glow-card {
    background: var(--surface);
    box-shadow: 0 6px 14px 1px rgba(123, 47, 247, 0.13);
    border: 1px solid #efe6ff;
    transition: transform 130ms ease-out, box-shadow 130ms ease-out;
  }
  .clickable {
    cursor: pointer;
  }
  .hoverable:hover {
    transform: scale(1.02);
    box-shadow: 0 10px 24px 2px rgba(123, 47, 247, 0.2);
  }
</style>
