<!-- Message-the-table modal: quick-send canned messages plus a custom
     multiline field. All kitchen-authored messages are prefixed
     "Kitchen: " so the UI can later tell kitchen vs. guest messages apart
     purely from the string prefix. -->
<script>
  import { api } from "../api.js";
  import { showToast } from "../stores.js";
  import { hoverGrow } from "../actions.js";

  export let order;
  export let onClose = () => {};
  export let onSent = () => {};

  const QUICK_MESSAGES = [
    "Your order is being prepared now.",
    "Your order will be ready shortly.",
    "Sorry, we're running about 10 minutes behind.",
    "One item is unavailable — may we suggest a substitute?",
    "Your order is ready and on its way!",
  ];

  let customText = "";
  let sending = false;

  $: isGuestMessage = (order.kitchenMessage ?? "").startsWith("Guest:");
  $: guestMessageBody = isGuestMessage ? order.kitchenMessage.slice("Guest:".length).trim() : "";

  async function send(text) {
    if (sending) return;
    sending = true;
    try {
      await api.setKitchenMessage(order.id, "Kitchen: " + text);
      showToast(`Message sent to Table ${order.tableId || "?"}`, "var(--brand)");
      onSent();
    } finally {
      sending = false;
    }
  }

  function sendQuick(text) {
    send(text);
  }

  function sendCustom() {
    const trimmed = customText.trim();
    if (!trimmed) return;
    send(trimmed);
  }
</script>

<div class="backdrop" on:click|self={onClose}>
  <div class="modal">
    <h3>Message Table {order.tableId || "?"}&nbsp;&nbsp;(Order #{order.id})</h3>

    {#if isGuestMessage}
      <div class="guest-banner">Guest replied: {guestMessageBody}</div>
    {/if}

    <div class="quick-list">
      {#each QUICK_MESSAGES as msg}
        <button class="quick-btn" use:hoverGrow={{ color: "var(--brand-dark)", scale: 1.02 }} disabled={sending} on:click={() => sendQuick(msg)}>
          {msg}
        </button>
      {/each}
    </div>

    <div class="custom-block">
      <textarea
        class="custom-input"
        rows="3"
        placeholder="Write a custom message…"
        bind:value={customText}
      />
      <button
        class="send-custom-btn"
        use:hoverGrow={{ color: "white", scale: 1.05 }}
        disabled={sending || !customText.trim()}
        on:click={sendCustom}
      >
        Send custom
      </button>
    </div>

    <div class="footer">
      <button class="close-btn" use:hoverGrow={{ color: "var(--brand)" }} on:click={onClose}>Close</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(51, 35, 91, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }
  .modal {
    background: var(--surface);
    border-radius: 16px;
    padding: 24px;
    width: min(460px, 92vw);
    max-height: 86vh;
    overflow-y: auto;
    box-shadow: var(--shadow-strong);
  }
  h3 {
    margin: 0 0 14px;
    font-family: var(--font-display);
    font-style: italic;
    color: var(--purple);
    font-size: 19px;
  }
  .guest-banner {
    background: var(--surface-2);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 13px;
    color: var(--text);
    margin-bottom: 14px;
  }
  .quick-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  .quick-btn {
    width: 100%;
    text-align: left;
    border: 1px solid var(--brand);
    background: transparent;
    color: var(--brand-dark);
    border-radius: 10px;
    padding: 10px 14px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .quick-btn:hover:not(:disabled) {
    background: var(--surface-2);
  }
  .quick-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .custom-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  .custom-input {
    width: 100%;
    border: 1px solid #efe6ff;
    border-radius: 10px;
    padding: 10px 12px;
    font-family: inherit;
    font-size: 13px;
    color: var(--text);
    resize: vertical;
  }
  .send-custom-btn {
    align-self: flex-end;
    border: none;
    border-radius: 10px;
    padding: 9px 18px;
    background: var(--grad-brand);
    color: white;
    font-weight: 700;
    cursor: pointer;
  }
  .send-custom-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .footer {
    display: flex;
    justify-content: flex-end;
  }
  .close-btn {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-weight: 600;
    padding: 9px 14px;
    cursor: pointer;
  }
</style>
