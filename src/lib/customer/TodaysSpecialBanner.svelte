<!-- Persistent "Today's Special" banner shown above the body on every
     customer screen. Port of CustomerApp._todays_special_line(): dark
     brown normally, maroon on hover/mouseover, tapping jumps to the
     Today's Special page (the customer module's home screen). -->
<script>
  export let specials = [];
  export let onClick = () => {};

  let hovering = false;

  $: label = specials.length > 1 ? "Today's Specials: " : "Today's Special: ";
  $: text =
    specials.length === 0
      ? ""
      : specials.length === 1
      ? label + specials[0].title
      : label +
        specials
          .slice(0, -1)
          .map((s) => s.title)
          .join(", ") +
        ` & ${specials[specials.length - 1].title}`;
</script>

{#if specials.length > 0}
  <button
    type="button"
    class="banner"
    style="color: {hovering ? 'var(--maroon)' : 'var(--dark-brown)'};"
    on:mouseenter={() => (hovering = true)}
    on:mouseleave={() => (hovering = false)}
    on:click={onClick}
    title="See today's specials"
  >
    {text}
  </button>
{/if}

<style>
  .banner {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: var(--font-body);
    font-style: italic;
    font-weight: 700;
    font-size: 17px;
    padding: 8px 16px;
    border-radius: 8px;
    transition: color 120ms ease;
  }
  .banner:hover {
    background: rgba(128, 0, 0, 0.06);
  }
</style>
