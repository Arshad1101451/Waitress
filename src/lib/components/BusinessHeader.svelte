<!-- Gradient banner with business identity + contact line. Port of
     chrome.business_header(). -->
<script>
  import { assetUrl } from "../api.js";

  export let restaurant;
  export let compact = false;
  export let subtitle = "";

  $: contactBits = [restaurant?.address, restaurant?.phone, restaurant?.email].filter(Boolean);
  $: contact = contactBits.length
    ? contactBits.join("   •   ")
    : "Add your address & contact in Admin ▸ Branding";
  $: fontSize = compact ? 22 : 32;
  $: logoSize = compact ? 44 : 58;
  $: subtitleText = subtitle || restaurant?.tagline;

  let logoSrc = "";
  $: (async () => {
    logoSrc = restaurant?.logoPath ? await assetUrl(restaurant.logoPath) : "";
  })();
</script>

<div class="header">
  <div class="logo" style="width:{logoSize}px;height:{logoSize}px;">
    {#if logoSrc}
      <img src={logoSrc} alt="logo" />
    {:else}
      <span class="logo-fallback">🍽</span>
    {/if}
  </div>
  <div class="text-col">
    <span class="name" style="font-size:{fontSize}px;">{restaurant?.name ?? "Your Restaurant"}</span>
    {#if subtitleText}
      <span class="subtitle">{subtitleText}</span>
    {/if}
    <span class="contact">{contact}</span>
  </div>
  <slot name="corner" />
</div>

<style>
  .header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--grad-header);
    box-shadow: 0 6px 16px rgba(123, 47, 247, 0.27);
    padding: 16px 20px;
  }
  .logo {
    border-radius: 50%;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .logo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .logo-fallback {
    font-size: 22px;
  }
  .text-col {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .name {
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 700;
    color: white;
    text-shadow: 1.5px 2px 0 #3a007a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .subtitle {
    font-style: italic;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.87);
  }
  .contact {
    font-style: italic;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.8);
  }
</style>
