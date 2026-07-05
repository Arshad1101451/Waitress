<script>
  export let text = "";
  export let size = 14;
  export let color = "var(--text)";
  export let bold = true;
</script>

<span class="shiny" style="font-size: {size}px; color: {color}; font-weight: {bold ? 700 : 400};">
  {text}
</span>

<style>
  .shiny {
    font-style: italic;
  }
</style>
