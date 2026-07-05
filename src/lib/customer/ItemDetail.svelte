<!-- Selected item's detail pane + quantity stepper + Add to Order + the
     order summary beneath it. Port of CustomerApp._detail_pane(). -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";
  import OrderSummary from "./OrderSummary.svelte";
  import { assetUrl } from "../api.js";
  import { money } from "../format.js";
  import { showToast } from "../stores.js";
  import { hoverGrow } from "../actions.js";

  export let item;
  export let qty = 1;
  export let onBumpQty = (delta) => {};
  export let onAdd = () => {};
  export let currencySymbol = "$";
  export let cart;
  export let taxRate = 0;
  export let onBumpCart = () => {};
  export let onPlaceOrder = () => {};

  function add() {
    onAdd();
    showToast(`Added ${qty} × ${item.name} to your order`);
  }
</script>

<div class="detail">
  <GlowCard padding={18}>
    {#await assetUrl(item.imagePath) then src}
      {#if src}
        <img class="hero" {src} alt={item.name} />
      {:else}
        <div class="hero placeholder">🍽</div>
      {/if}
    {/await}
    <Heading text={item.name} size={26} />
    <p class="desc">{item.description || ""}</p>
    <div class="qty-row">
      <span class="price">{money(currencySymbol, item.price)}</span>
      <div class="spacer" />
      <button class="stepper" use:hoverGrow={{ color: "var(--purple)", scale: 1.15 }} on:click={() => onBumpQty(-1)}>−</button>
      <span class="qty">{qty}</span>
      <button class="stepper accent" use:hoverGrow={{ color: "var(--pink)", scale: 1.15 }} on:click={() => onBumpQty(1)}>+</button>
    </div>
    <Button label="Add to Order" gradient="var(--grad-brand)" expand height={52} icon="&#128722;" onClick={add} />
  </GlowCard>

  <OrderSummary {cart} {taxRate} {currencySymbol} onBump={onBumpCart} {onPlaceOrder} />
</div>

<style>
  .detail {
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
    padding: 4px;
  }
  .hero {
    width: 100%;
    height: 220px;
    object-fit: cover;
    border-radius: 14px;
    margin-bottom: 12px;
  }
  .hero.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--grad-soft);
    color: white;
    font-size: 44px;
  }
  .desc {
    color: var(--text-muted);
    font-style: italic;
    font-size: 14px;
  }
  .qty-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 14px 0;
  }
  .price {
    font-style: italic;
    font-weight: 700;
    font-size: 22px;
    color: var(--pink);
  }
  .spacer {
    flex: 1;
  }
  .stepper {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 1px solid var(--surface-2);
    background: white;
    color: var(--purple);
    font-size: 18px;
    cursor: pointer;
  }
  .stepper.accent {
    color: var(--pink);
  }
  .qty {
    font-weight: 700;
    font-size: 18px;
    min-width: 20px;
    text-align: center;
  }
</style>
