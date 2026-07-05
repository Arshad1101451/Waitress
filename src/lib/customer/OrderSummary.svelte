<!-- Cart line list + totals + "Place Order" button. Appears under the item
     detail pane AND the welcome/Today's-Special pane. Port of
     CustomerApp._order_summary(). -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Button from "../components/Button.svelte";
  import { money } from "../format.js";
  import { subtotal, taxFor, totalFor, lineTotal } from "./cart.js";
  import { hoverGrow } from "../actions.js";

  export let cart;
  export let taxRate = 0;
  export let currencySymbol = "$";
  export let onBump = (itemId, delta) => {};
  export let onPlaceOrder = () => {};

  $: lines = Array.from(cart.values());
  $: sub = subtotal(cart);
  $: tax = taxFor(cart, taxRate);
  $: total = totalFor(cart, taxRate);
</script>

<GlowCard padding={18}>
  {#if lines.length === 0}
    <div class="empty">
      <span class="icon">🛒</span>
      <span class="msg">Your order is empty — add something tasty!</span>
    </div>
  {:else}
    <div class="lines">
      {#each lines as line (line.item.id)}
        <div class="line" use:hoverGrow={{ color: "var(--pink)" }}>
          <span class="qty">{line.quantity}×</span>
          <span class="name">{line.item.name}</span>
          <span class="total">{money(currencySymbol, lineTotal(line))}</span>
          <button class="stepper" use:hoverGrow={{ color: "var(--purple)", scale: 1.15 }} on:click={() => onBump(line.item.id, -1)}>−</button>
          <button class="stepper accent" use:hoverGrow={{ color: "var(--pink)", scale: 1.15 }} on:click={() => onBump(line.item.id, 1)}>+</button>
        </div>
      {/each}
    </div>
    <div class="totals">
      <div class="row"><span>Subtotal</span><span>{money(currencySymbol, sub)}</span></div>
      <div class="row"><span>Tax ({Math.round(taxRate * 100)}%)</span><span>{money(currencySymbol, tax)}</span></div>
      <div class="row grand"><span>Total</span><span>{money(currencySymbol, total)}</span></div>
    </div>
    <Button
      label={`Place Order • ${money(currencySymbol, total)}`}
      gradient="var(--grad-sunny)"
      height={54}
      expand
      icon="&#10003;"
      onClick={onPlaceOrder}
    />
  {/if}
</GlowCard>

<style>
  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-muted);
    font-style: italic;
  }
  .empty .icon {
    font-size: 22px;
  }
  .lines {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 14px;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .qty {
    font-weight: 700;
    color: var(--pink);
    width: 34px;
  }
  .name {
    flex: 1;
  }
  .total {
    font-weight: 700;
    width: 76px;
    text-align: right;
  }
  .stepper {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 1px solid var(--surface-2);
    background: white;
    color: var(--purple);
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
  }
  .stepper.accent {
    color: var(--pink);
  }
  .totals {
    border-top: 1px solid var(--surface-2);
    padding-top: 10px;
    margin-bottom: 16px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }
  .row.grand {
    font-size: 18px;
    font-weight: 700;
    color: var(--text);
    font-style: italic;
  }
</style>
