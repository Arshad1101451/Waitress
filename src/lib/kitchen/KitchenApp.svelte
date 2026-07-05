<!-- Kitchen module root — live order board for the back-of-house display.
     Polls the restaurant settings + all 4 board columns every 5s so
     branding/display-mode changes and new orders show up automatically
     without any user action, plus a manual refresh affordance in the
     header and NavDock's Home button. -->
<script>
  import { onMount, onDestroy } from "svelte";
  import { api } from "../api.js";
  import { restaurant as restaurantStore } from "../stores.js";
  import Background from "../components/Background.svelte";
  import BusinessHeader from "../components/BusinessHeader.svelte";
  import NavDock from "../components/NavDock.svelte";
  import Board from "./Board.svelte";
  import { hoverGrow } from "../actions.js";

  export let onExit = () => {};

  const BOARD_STATUSES = ["NEW", "ACCEPTED", "PREPARING", "READY"];

  let restaurant = null;
  let columns = { NEW: [], ACCEPTED: [], PREPARING: [], READY: [] };
  let activeStatus = "NEW";
  let loaded = false;
  let pollHandle = null;

  const DISPLAY_MODE_LABELS = {
    tablet: "Tablet",
    printer: "Printer only",
    both: "Tablet + Printer",
  };

  onMount(async () => {
    await refresh();
    loaded = true;
    pollHandle = setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });

  async function refresh() {
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);

    const fetched = await Promise.all(
      BOARD_STATUSES.map((status) => api.getOrders(status, null))
    );
    const next = {};
    BOARD_STATUSES.forEach((status, i) => {
      // Backend returns newest-first; board should read oldest-first (FIFO).
      next[status] = [...(fetched[i] ?? [])].reverse();
    });
    columns = next;
  }

  function displayModeLabel(mode) {
    return DISPLAY_MODE_LABELS[mode] ?? mode ?? "";
  }
</script>

<Background {restaurant}>
  <BusinessHeader {restaurant} compact subtitle="Kitchen display">
    <div slot="corner" class="corner">
      <div class="mode-badge" use:hoverGrow={{ color: "white", scale: 1.05 }} title="Kitchen display mode">
        <span class="printer-icon">🖨</span>
        <span>{displayModeLabel(restaurant?.kitchenDisplayMode)}</span>
      </div>
      <button class="refresh-btn" use:hoverGrow={{ color: "white", scale: 1.15 }} title="Refresh board" on:click={refresh}>
        🔄
      </button>
    </div>
  </BusinessHeader>

  <div class="body">
    {#if loaded}
      <Board {columns} bind:activeStatus onChanged={refresh} />
    {/if}
  </div>

  <NavDock
    canBack={false}
    canForward={false}
    onHome={refresh}
    onExit={onExit}
  />
</Background>

<style>
  .corner {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }
  .mode-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.22);
    color: white;
    border-radius: 999px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
  }
  .printer-icon {
    font-size: 14px;
  }
  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.22);
    color: white;
    font-size: 18px;
    cursor: pointer;
  }
  .refresh-btn:hover {
    background: rgba(255, 255, 255, 0.35);
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
</style>
