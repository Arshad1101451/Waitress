<!-- Live order board. Wide layout (>=600px): 4 independently-scrolling
     columns side by side. Compact layout (<600px): a horizontally-scrolling
     row of status chips above a single full-width column showing only the
     active status. -->
<script>
  import { restaurant as restaurantStore } from "../stores.js";
  import OrderCard from "./OrderCard.svelte";
  import { hoverGrow } from "../actions.js";

  export let columns; // { NEW: Order[], ACCEPTED: Order[], PREPARING: Order[], READY: Order[] }
  export let activeStatus = "NEW";
  export let onChanged = () => {};

  const BOARD_STATUSES = ["NEW", "ACCEPTED", "PREPARING", "READY"];
  const TITLES = {
    NEW: "Incoming",
    ACCEPTED: "Accepted",
    PREPARING: "Preparing",
    READY: "Ready",
  };
  const ACCENTS = {
    NEW: "var(--accent)",
    ACCEPTED: "#5B8DEF",
    PREPARING: "var(--brand)",
    READY: "var(--success)",
  };

  let compact = false;

  function checkCompact() {
    compact = window.innerWidth < 600;
  }

  checkCompact();
  if (typeof window !== "undefined") {
    window.addEventListener("resize", checkCompact);
  }

  $: currencySymbol = $restaurantStore?.currencySymbol ?? "$";
</script>

{#if compact}
  <div class="compact-wrap">
    <div class="chip-row">
      {#each BOARD_STATUSES as status}
        <button
          class="chip"
          class:active={activeStatus === status}
          use:hoverGrow={{ color: "var(--brand-dark)", scale: 1.06 }}
          on:click={() => (activeStatus = status)}
        >
          <span class="dot" style="background: {ACCENTS[status]};" />
          {TITLES[status]} ({(columns[status] ?? []).length})
        </button>
      {/each}
    </div>
    <div class="compact-column">
      {#if (columns[activeStatus] ?? []).length === 0}
        <div class="empty">No orders here.</div>
      {:else}
        {#each columns[activeStatus] as order (order.id)}
          <OrderCard {order} {currencySymbol} onChanged={onChanged} />
        {/each}
      {/if}
    </div>
  </div>
{:else}
  <div class="wide-wrap">
    {#each BOARD_STATUSES as status}
      <div class="column" style="border-top-color: {ACCENTS[status]};">
        <div class="column-header" use:hoverGrow={{ color: "var(--brand)", scale: 1.02 }}>
          <span class="dot" style="background: {ACCENTS[status]};" />
          <span class="title">{TITLES[status]}</span>
          <span class="count">{(columns[status] ?? []).length}</span>
        </div>
        <div class="column-body">
          {#if (columns[status] ?? []).length === 0}
            <div class="empty">&mdash;</div>
          {:else}
            {#each columns[status] as order (order.id)}
              <OrderCard {order} {currencySymbol} onChanged={onChanged} />
            {/each}
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .wide-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
    padding: 14px;
  }
  .column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
    border-radius: var(--radius);
    border-top: 5px solid;
    overflow: hidden;
  }
  .column-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 14px;
    flex-shrink: 0;
  }
  .title {
    font-weight: 700;
    color: var(--text);
    font-size: 15px;
  }
  .count {
    margin-left: auto;
    background: var(--surface);
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 700;
    border-radius: 999px;
    padding: 2px 9px;
  }
  .column-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 10px 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: 24px 0;
    font-size: 14px;
  }

  /* compact layout */
  .compact-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .chip-row {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 10px 12px;
    flex-shrink: 0;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    border: 1px solid #efe6ff;
    background: var(--surface);
    color: var(--text);
    border-radius: 999px;
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .chip.active {
    background: var(--surface-2);
    border-color: var(--brand);
    color: var(--brand-dark);
  }
  .compact-column {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>
