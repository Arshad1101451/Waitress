<!-- Today's Special / home screen — also reused as the "nothing selected"
     state of the menu detail pane. Port of CustomerApp._welcome_pane(). -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";
  import OrderSummary from "./OrderSummary.svelte";
  import { assetUrl } from "../api.js";
  import { money } from "../format.js";
  import { hoverGrow } from "../actions.js";

  export let greetingText = "";
  export let speaking = false;
  export let onToggleHear = () => {};
  export let specials = [];
  export let currencySymbol = "$";
  export let cart;
  export let taxRate = 0;
  export let onBump = () => {};
  export let onPlaceOrder = () => {};
  export let onBrowseMenu = null;
</script>

<div class="welcome">
  <GlowCard padding={18}>
    <div class="greeting-row" use:hoverGrow={{ color: "var(--purple)", scale: 1.015 }}>
      <span class="greeting-icon">🗣️</span>
      <p class="greeting-text">{greetingText}</p>
      {#if onBrowseMenu}
        <Button label="Browse Menu" icon="&#127869;" gradient="var(--grad-brand)" width={150} onClick={onBrowseMenu} />
      {/if}
      <Button
        label={speaking ? "Stop" : "Hear"}
        icon={speaking ? "&#9209;" : "&#128266;"}
        gradient={speaking ? "var(--grad-cool)" : "var(--grad-sunny)"}
        width={110}
        onClick={onToggleHear}
      />
    </div>
  </GlowCard>

  {#if specials.length > 0}
    <Heading text="Today's Specials" size={22} />
    <div class="specials-row">
      {#each specials as sp (sp.id)}
        <GlowCard hover padding={0} radius={16}>
          <div class="special-card">
            {#await assetUrl(sp.imagePath) then src}
              {#if src}
                <img class="special-img" {src} alt={sp.title} />
              {:else}
                <div class="special-img placeholder">🍽</div>
              {/if}
            {/await}
            <div class="special-body">
              <span class="special-title">{sp.title}</span>
              <span class="special-desc">{sp.description}</span>
              <span class="special-price">{money(currencySymbol, sp.price)}</span>
            </div>
          </div>
        </GlowCard>
      {/each}
    </div>
  {/if}

  <OrderSummary {cart} {taxRate} {currencySymbol} {onBump} {onPlaceOrder} />
</div>

<style>
  .welcome {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px;
    overflow-y: auto;
  }
  .greeting-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .greeting-icon {
    font-size: 20px;
    color: var(--pink);
  }
  .greeting-text {
    flex: 1;
    font-style: italic;
    color: var(--text);
    margin: 0;
  }
  .specials-row {
    display: flex;
    gap: 14px;
    overflow-x: auto;
    padding-bottom: 6px;
  }
  .special-card {
    width: 240px;
  }
  .special-img {
    width: 240px;
    height: 130px;
    object-fit: cover;
    border-radius: 16px 16px 0 0;
  }
  .special-img.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--grad-soft);
    color: white;
    font-size: 32px;
  }
  .special-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
  }
  .special-title {
    font-weight: 700;
    color: var(--purple);
    font-size: 16px;
  }
  .special-desc {
    color: var(--text-muted);
    font-style: italic;
    font-size: 12px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .special-price {
    color: var(--pink);
    font-weight: 700;
    font-size: 15px;
    font-style: italic;
  }
</style>
