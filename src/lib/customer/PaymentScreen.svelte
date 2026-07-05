<!-- Checkout / payment method selection. Port of CustomerApp._body_payment. -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";
  import { money } from "../format.js";
  import { totalFor } from "./cart.js";
  import { hoverGrow } from "../actions.js";

  export let cart;
  export let taxRate = 0;
  export let currencySymbol = "$";
  export let paymentGateways = "cash"; // CSV
  export let onConfirm = (method) => {};

  const labels = { cash: "Pay with Cash", card: "Card at Table", stripe: "Stripe", square: "Square", paypal: "PayPal" };

  $: gateways = (paymentGateways || "")
    .split(",")
    .map((g) => g.trim())
    .filter(Boolean);
  $: selected = gateways[0] || "cash";
  $: total = totalFor(cart, taxRate);

  function label(g) {
    return labels[g.toLowerCase()] || g.charAt(0).toUpperCase() + g.slice(1);
  }
</script>

<div class="payment">
  <GlowCard padding={20}>
    <Heading text="Payment" size={24} />
    <p class="subtitle">Choose how you'd like to pay</p>
    <div class="options">
      {#each gateways as g (g)}
        <label class="option" class:active={selected === g} use:hoverGrow={{ color: "var(--purple)" }}>
          <input type="radio" name="gateway" value={g} bind:group={selected} />
          {label(g)}
        </label>
      {/each}
    </div>
    <div class="due-row">
      <span>Amount due</span>
      <span class="amount">{money(currencySymbol, total)}</span>
    </div>
    <p class="disclaimer">Demo build: payment is simulated.</p>
    <Button
      label="Confirm & Place Order"
      gradient="var(--grad-brand)"
      height={54}
      expand
      onClick={() => onConfirm(selected)}
    />
  </GlowCard>
</div>

<style>
  .payment {
    padding: 4px;
    max-width: 480px;
  }
  .subtitle {
    font-style: italic;
    color: var(--text-muted);
    margin: 4px 0 18px;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 18px;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 10px;
    background: var(--surface-2);
    cursor: pointer;
  }
  .option.active {
    outline: 2px solid var(--purple);
  }
  .due-row {
    display: flex;
    justify-content: space-between;
    font-weight: 700;
    font-size: 16px;
    margin-bottom: 6px;
  }
  .disclaimer {
    font-size: 11px;
    font-style: italic;
    color: var(--text-muted);
    margin-bottom: 18px;
  }
</style>
