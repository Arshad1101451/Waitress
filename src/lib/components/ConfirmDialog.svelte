<!-- Single generic confirmation modal — every destructive/committing action
     in Admin & Kitchen routes through confirmAction() (stores.js) to open
     this, mirroring the Python app's one shared `_confirm()` helper. -->
<script>
  import { confirmState, resolveConfirm } from "../stores.js";
</script>

{#if $confirmState}
  <div class="backdrop" on:click|self={() => resolveConfirm(false)}>
    <div class="modal">
      <h3>{$confirmState.title}</h3>
      <p>{$confirmState.body}</p>
      <div class="actions">
        <button class="ghost" on:click={() => resolveConfirm(false)}>Cancel</button>
        <button
          class="solid"
          style="background: {$confirmState.danger ? 'var(--danger)' : 'var(--brand)'};"
          on:click={() => resolveConfirm(true)}
        >
          {$confirmState.yesLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

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
    width: min(420px, 90vw);
    box-shadow: var(--shadow-strong);
  }
  h3 {
    margin: 0 0 10px;
    font-family: var(--font-display);
    font-style: italic;
    color: var(--purple);
  }
  p {
    margin: 0 0 20px;
    color: var(--text);
    font-size: 14px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
  button {
    border: none;
    border-radius: 10px;
    padding: 9px 18px;
    font-weight: 600;
    cursor: pointer;
  }
  .ghost {
    background: transparent;
    color: var(--text-muted);
  }
  .solid {
    color: white;
  }
</style>
