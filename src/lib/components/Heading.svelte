<!-- Shiny display-font heading with an offset-emboss shadow. Port of
     theme.heading(). -->
<script>
  export let text = "";
  export let size = 28;
  export let color = "var(--purple)";
</script>

<span class="heading" style="font-size: {size}px; color: {color};">{text}</span>

<style>
  .heading {
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 700;
    display: inline-block;
    text-shadow: 2px 3px 0 rgba(0, 0, 0, 0.09);
  }
</style>
