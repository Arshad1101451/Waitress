<!-- One order's ticket card: header (id/table/age), total + payment pill,
     item list, latest kitchen message (if any), and the action row
     (advance / message / print / cancel). -->
<script>
  import { api } from "../api.js";
  import { showToast, confirmAction } from "../stores.js";
  import { money, agoText, titleCase } from "../format.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Button from "../components/Button.svelte";
  import MessageDialog from "./MessageDialog.svelte";
  import { hoverGrow } from "../actions.js";

  export let order;
  export let currencySymbol = "$";
  export let onChanged = () => {};

  const ADVANCE = {
    NEW: { label: "Accept", next: "ACCEPTED" },
    ACCEPTED: { label: "Start preparing", next: "PREPARING" },
    PREPARING: { label: "Mark ready", next: "READY" },
    READY: { label: "Mark served", next: "SERVED" },
  };

  let showMessageDialog = false;

  $: advance = ADVANCE[order.status] ?? null;
  $: isGuestMessage = (order.kitchenMessage ?? "").startsWith("Guest:");

  async function handleAdvance() {
    if (!advance) return;
    await api.updateOrderStatus(order.id, advance.next);
    showToast(`Order #${order.id} → ${titleCase(advance.next)}`);
    onChanged();
  }

  async function handlePrint() {
    const result = await api.printOrderTicket(order.id);
    showToast(result.message, result.printed ? "var(--brand)" : "var(--text-muted)");
  }

  function handleCancel() {
    confirmAction({
      title: "Cancel order?",
      body: `Cancel order #${order.id}? This notifies nobody automatically — message the table first if needed.`,
      yesLabel: "Cancel order",
      danger: true,
      onYes: async () => {
        await api.updateOrderStatus(order.id, "CANCELLED");
        showToast(`Order #${order.id} cancelled`, "var(--danger)");
        onChanged();
      },
    });
  }

  function openMessageDialog() {
    showMessageDialog = true;
  }

  function closeMessageDialog() {
    showMessageDialog = false;
  }

  function handleMessageSent() {
    showMessageDialog = false;
    onChanged();
  }
</script>

<GlowCard padding={14} radius={16} hover>
  <div class="row top" use:hoverGrow={{ color: "var(--brand)", scale: 1.015 }}>
    <span class="ticket-id">#{order.id} &middot; Table {order.tableId || "?"}</span>
    <span class="ago">{agoText(order.createdAt)}</span>
  </div>

  <div class="row totals">
    <span class="total">{money(currencySymbol, order.total)}</span>
    <span class="pill" class:paid={order.paymentStatus === "PAID"}>{order.paymentStatus}</span>
  </div>

  <div class="divider" />

  <div class="items">
    {#each order.items ?? [] as item (item.id)}
      <div class="item" use:hoverGrow={{ color: "var(--pink)", scale: 1.02 }}>
        <span class="item-line"><strong>{item.quantity} &times; {item.name}</strong></span>
        {#if item.notes}
          <span class="item-notes">&nbsp;&nbsp;&nbsp;&nbsp;↳ {item.notes}</span>
        {/if}
      </div>
    {/each}
  </div>

  {#if order.kitchenMessage}
    <div class="message-box" use:hoverGrow={{ color: "var(--brand)", scale: 1.015 }}>
      <span class="message-icon" style="color: {isGuestMessage ? 'var(--accent)' : 'var(--brand)'};">
        💬
      </span>
      <span class="message-text">{order.kitchenMessage}</span>
    </div>
  {/if}

  <div class="actions">
    {#if advance}
      <Button label={advance.label} expand height={40} size={13} onClick={handleAdvance} />
    {/if}
    <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.15 }} title="Message table" on:click={openMessageDialog}>💬</button>
    <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.15 }} title="Print ticket" on:click={handlePrint}>🖨</button>
    <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.15 }} title="Cancel order" on:click={handleCancel}>✕</button>
  </div>
</GlowCard>

{#if showMessageDialog}
  <MessageDialog {order} onClose={closeMessageDialog} onSent={handleMessageSent} />
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .top {
    margin-bottom: 6px;
  }
  .ticket-id {
    font-weight: 700;
    color: var(--text);
    font-size: 14px;
  }
  .ago {
    font-size: 12px;
    color: var(--text-muted);
  }
  .totals {
    margin-bottom: 8px;
  }
  .total {
    font-weight: 700;
    color: var(--brand);
    font-size: 15px;
  }
  .pill {
    font-size: 11px;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text-muted);
  }
  .pill.paid {
    background: var(--success);
    color: white;
  }
  .divider {
    height: 1px;
    background: #efe6ff;
    margin: 8px 0;
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 6px;
  }
  .item {
    display: flex;
    flex-direction: column;
  }
  .item-line {
    font-size: 13px;
    color: var(--text);
  }
  .item-notes {
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
  }
  .message-box {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    background: var(--surface-2);
    border-radius: 10px;
    padding: 8px 10px;
    margin: 6px 0;
  }
  .message-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
  .message-text {
    font-size: 12px;
    color: var(--text);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    flex-shrink: 0;
    border-radius: 10px;
    border: 1px solid #efe6ff;
    background: var(--surface);
    color: var(--brand);
    font-size: 16px;
    cursor: pointer;
  }
  .icon-btn:hover {
    background: var(--surface-2);
  }
  .icon-btn.danger {
    color: var(--danger);
  }
</style>
