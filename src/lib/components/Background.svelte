<!-- Faint dining-room backdrop behind every screen. Port of
     chrome.with_background(). -->
<script>
  import { assetUrl } from "../api.js";
  export let restaurant;

  let bgSrc = "";
  $: (async () => {
    bgSrc = restaurant?.activeBackground ? await assetUrl(restaurant.activeBackground) : "";
  })();
</script>

<div class="wrap">
  {#if bgSrc}
    <img class="bg" src={bgSrc} alt="" />
  {/if}
  <div class="scrim" />
  <div class="content"><slot /></div>
</div>

<style>
  .wrap {
    position: relative;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }
  .bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    /* Was 0.22, then 0.5 — still reported as too faint. Requested value is
       70%. */
    opacity: 0.7;
  }
  .scrim {
    position: absolute;
    inset: 0;
    /* Nudged up slightly alongside the opacity bump above so foreground
       cards/text stay readable against a much more visible photo. */
    background: #faf6ff99;
  }
  .content {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
</style>
