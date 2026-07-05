<!-- Live order tracking, polled every 4s by the parent. Port of
     CustomerApp._body_status / _render_status. -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";
  import { money } from "../format.js";
  import { titleCase } from "../format.js";
  import { hoverGrow } from "../actions.js";

  export let order = null;
  export let currencySymbol = "$";
  export let onReply = (text) => {};
  export let onStartNew = () => {};

  const FLOW = ["NEW", "ACCEPTED", "PREPARING", "READY", "SERVED"];
  const LABELS = {
    NEW: "Sent to kitchen",
    ACCEPTED: "Kitchen accepted",
    PREPARING: "Being prepared",
    READY: "Ready to serve",
    SERVED: "Served — enjoy!",
    CANCELLED: "Cancelled",
  };

  $: idx = order ? FLOW.indexOf(order.status) : -1;
</script>

<div class="status">
  {#if order}
    <GlowCard padding={18}>
      <Heading text={`Order #${order.id}`} size={22} />
      <p class="meta">
        Table {order.tableId || "?"} • {money(currencySymbol, order.total)} • {titleCase(order.paymentStatus)}
      </p>
    </GlowCard>

    <GlowCard padding={18}>
      <div class="steps">
        {#each FLOW as step, i (step)}
          <div class="step" class:done={i <= idx} class:current={i === idx} use:hoverGrow={{ color: "var(--purple)" }}>
            <span class="dot">{i <= idx ? "✓" : "○"}</span>
            <span class="label">{LABELS[step]}</span>
          </div>
        {/each}
      </div>
    </GlowCard>

    {#if order.kitchenMessage}
      <GlowCard padding={18}>
        <div class="msg-header">
          <span class="icon">💬</span>
          <Heading text="Message from the kitchen" size={16} color="var(--purple)" />
        </div>
        <p class="msg-body">{order.kitchenMessage}</p>
        <div class="reply-row">
          <Button label="OK, sounds good" gradient="var(--grad-brand)" onClick={() => onReply("Guest: OK, sounds good")} />
          <Button label="Please change it" gradient="var(--grad-sunny)" onClick={() => onReply("Guest: please change the item")} />
        </div>
      </GlowCard>
    {/if}
  {/if}

  <Button label="Start a new order" gradient="var(--grad-brand)" height={50} icon="&#43;" expand onClick={onStartNew} />
</div>

<style>
  .status {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px;
    overflow-y: auto;
  }
  .meta {
    font-style: italic;
    color: var(--text-muted);
    margin: 4px 0 0;
  }
  .steps {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .step {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-muted);
  }
  .step .dot {
    color: var(--light-purple);
    width: 20px;
  }
  .step.done {
    color: var(--text);
  }
  .step.done .dot {
    color: var(--success);
  }
  .step.current .label {
    font-weight: 700;
  }
  .msg-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .msg-body {
    color: var(--text-muted);
    margin-bottom: 14px;
  }
  .reply-row {
    display: flex;
    gap: 10px;
  }
</style>
