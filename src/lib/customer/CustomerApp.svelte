<!-- Root of the Customer module — port of waitress/customer/app.py. -->
<script>
  import { onMount, onDestroy } from "svelte";
  import { api } from "../api.js";
  import { restaurant as restaurantStore, deviceConfig, confirmAction, showToast } from "../stores.js";
  import BusinessHeader from "../components/BusinessHeader.svelte";
  import NavDock from "../components/NavDock.svelte";
  import Background from "../components/Background.svelte";
  import Home from "./Home.svelte";
  import MenuTree from "./MenuTree.svelte";
  import ItemDetail from "./ItemDetail.svelte";
  import PaymentScreen from "./PaymentScreen.svelte";
  import StatusScreen from "./StatusScreen.svelte";
  import AdminPinDialog from "./AdminPinDialog.svelte";
  import TodaysSpecialBanner from "./TodaysSpecialBanner.svelte";
  import { buildGreeting, voice } from "./voice.js";
  import { emptyCart, addLine, setQuantity, clearCart, toOrder } from "./cart.js";

  export let onExit = () => {};

  let restaurant = null;
  let cfg = null;
  let categories = [];
  let itemsByCategory = {};
  let specials = [];
  let cart = emptyCart();
  let qty = 1;
  let speaking = false;
  let greeted = false;
  let activeOrderId = null;
  let order = null;
  let statusTimer = null;
  let compact = false;
  let adminDialog;

  // simple opaque-token nav history, mirrors chrome.NavStack
  let history = [{ kind: "special" }];
  let historyIdx = 0;
  $: route = history[historyIdx];
  $: canBack = historyIdx > 0;
  $: canForward = historyIdx < history.length - 1;

  function go(token, record = true) {
    if (record) {
      history = [...history.slice(0, historyIdx + 1), token];
      historyIdx = history.length - 1;
    }
    if (token.kind !== "status") stopStatusPolling();
    if (token.kind === "menu" || token.kind === "special") {
      refreshMenuData();
    }
  }

  function navBack() {
    if (canBack) {
      historyIdx -= 1;
      go(history[historyIdx], false);
    }
  }
  function navForward() {
    if (canForward) {
      historyIdx += 1;
      go(history[historyIdx], false);
    }
  }

  async function refreshMenuData() {
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);
    const dateStr = new Date().toISOString().slice(0, 10);
    specials = await api.getActiveSpecials(dateStr);
  }

  async function loadCategoriesAndItems() {
    categories = await api.getCategories();
    const map = {};
    for (const cat of categories) {
      map[cat.id] = await api.getMenuItems(cat.id, true);
    }
    itemsByCategory = map;
  }

  $: greetingText = buildGreeting(
    restaurant?.name ?? "Your Restaurant",
    specials.map((s) => s.title),
    categories.filter((c) => c.isStarterGroup).map((c) => c.name)
  );

  function onResize() {
    compact = window.innerWidth < 600;
  }

  onMount(async () => {
    window.addEventListener("resize", onResize);
    onResize();
    await api.ensureSeeded();
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);
    cfg = await api.getDeviceConfig();
    deviceConfig.set(cfg);
    await loadCategoriesAndItems();
    await refreshMenuData();
    tryEnableKiosk();
    greet();
  });

  onDestroy(() => {
    window.removeEventListener("resize", onResize);
    stopStatusPolling();
    voice.stop();
  });

  function greet() {
    if (greeted || !restaurant?.voiceGreetingEnabled) return;
    greeted = true;
    speaking = true;
    voice.speak(greetingText, {
      onDone: () => {
        speaking = false;
      },
    });
  }

  function toggleHear() {
    if (speaking) {
      voice.stop();
      speaking = false;
    } else {
      speaking = true;
      voice.speak(greetingText, {
        onDone: () => {
          speaking = false;
        },
      });
    }
  }

  // --------------------------------------------------------------- menu nav
  function selectItem(id) {
    qty = 1;
    go({ kind: "menu", itemId: id });
  }

  function bumpDetailQty(delta) {
    qty = Math.max(1, qty + delta);
  }

  function addToCart(item) {
    cart = addLine(cart, item, qty);
  }

  function bumpCart(itemId, delta) {
    const line = cart.get(itemId);
    const current = line ? line.quantity : 0;
    cart = setQuantity(cart, itemId, current + delta);
  }

  // Bug fix: this used to compare against a `selectedItemId` state variable
  // that was never assigned anywhere, so it was permanently null — which
  // meant tapping a menu item in the tree never actually opened its detail
  // page (it silently fell back to the welcome/home pane every time). The
  // route token itself already carries the selected item id, so derive
  // directly from that instead of a dead separate variable.
  $: selectedItem = (() => {
    if (route.kind !== "menu" || !route.itemId) return null;
    for (const items of Object.values(itemsByCategory)) {
      const found = items.find((i) => i.id === route.itemId);
      if (found) return found;
    }
    return null;
  })();

  // ---------------------------------------------------------------- payment
  async function placeOrder(method) {
    const built = toOrder(cart, {
      tableId: cfg?.tableId,
      deviceId: cfg?.deviceId,
      taxRate: restaurant?.taxRate ?? 0,
      paymentMethod: method,
    });
    const id = await api.createOrder(built);
    activeOrderId = id;
    cart = clearCart();
    go({ kind: "status" });
    startStatusPolling();
  }

  function startStatusPolling() {
    stopStatusPolling();
    renderStatus();
    statusTimer = setInterval(renderStatus, 4000);
  }
  function stopStatusPolling() {
    if (statusTimer) {
      clearInterval(statusTimer);
      statusTimer = null;
    }
  }
  async function renderStatus() {
    if (!activeOrderId) return;
    try {
      order = await api.getOrder(activeOrderId);
    } catch {
      /* keep last known state on transient error */
    }
  }
  async function replyToKitchen(message) {
    if (!activeOrderId) return;
    await api.setKitchenMessage(activeOrderId, message);
    showToast("Sent to the kitchen");
  }

  // ------------------------------------------------------------- kiosk/exit
  async function tryEnableKiosk() {
    if (!cfg?.kioskEnabled) return;
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      await win.setFullscreen(true);
      await win.setDecorations(false);
      await win.setResizable(false);
    } catch {
      /* not supported on this platform/build target — degrade silently */
    }
  }

  async function disableKiosk() {
    cfg = await api.updateDeviceConfig({ kioskEnabled: false });
    deviceConfig.set(cfg);
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      await win.setFullscreen(false);
      await win.setDecorations(true);
      await win.setResizable(true);
    } catch {
      /* ignore */
    }
  }

  async function reloadSettings() {
    await refreshMenuData();
    go({ kind: "special" });
  }

  function exit() {
    voice.stop();
    confirmAction({
      title: "Exit the app?",
      body: "Close Project Waitress on this device?",
      yesLabel: "Exit",
      danger: true,
      onYes: doExit,
    });
  }

  async function doExit() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().close();
    } catch {
      /* no window handle (mobile/web) — fall back to the module switcher */
      onExit();
    }
  }
</script>

{#if restaurant}
  <Background {restaurant}>
    <div class="header-stack">
      <BusinessHeader {restaurant} {compact} subtitle="Welcome — tap a category to explore the menu">
        <svelte:fragment slot="corner">
          <div class="corner">
            <AdminPinDialog bind:this={adminDialog} onReload={reloadSettings} onExitKiosk={disableKiosk} />
          </div>
        </svelte:fragment>
      </BusinessHeader>
      <TodaysSpecialBanner {specials} onClick={() => go({ kind: "special" })} />
    </div>

    <div class="body">
      {#if route.kind === "special" || route.kind === "menu"}
        {#if compact}
          <!-- Compact devices can't show tree + detail side by side, so the
               single body pane switches between: item detail (item picked),
               the browsable tree (menu route, nothing picked yet), or the
               home/welcome pane with a "Browse Menu" button that's the only
               way to reach the tree in this layout. -->
          {#if selectedItem}
            <ItemDetail
              item={selectedItem}
              {qty}
              onBumpQty={bumpDetailQty}
              onAdd={() => addToCart(selectedItem)}
              currencySymbol={restaurant.currencySymbol}
              {cart}
              taxRate={restaurant.taxRate}
              onBumpCart={bumpCart}
              onPlaceOrder={() => go({ kind: "payment" })}
            />
          {:else if route.kind === "menu"}
            <MenuTree
              {categories}
              {itemsByCategory}
              selectedItemId={route.itemId}
              currencySymbol={restaurant.currencySymbol}
              onSelect={selectItem}
            />
          {:else}
            <Home
              {greetingText}
              {speaking}
              onToggleHear={toggleHear}
              {specials}
              currencySymbol={restaurant.currencySymbol}
              {cart}
              taxRate={restaurant.taxRate}
              onBump={bumpCart}
              onPlaceOrder={() => go({ kind: "payment" })}
              onBrowseMenu={() => go({ kind: "menu" })}
            />
          {/if}
        {:else}
          <!-- Wide layout: the category tree is always visible on the left,
               on both the home ("special") and menu routes — previously the
               tree only rendered once already on the menu route, with no
               button anywhere that could get you there in the first place. -->
          <div class="split">
            <div class="tree-pane">
              <MenuTree
                {categories}
                {itemsByCategory}
                selectedItemId={route.itemId}
                currencySymbol={restaurant.currencySymbol}
                onSelect={selectItem}
              />
            </div>
            <div class="divider" />
            <div class="detail-pane">
              {#if selectedItem}
                <ItemDetail
                  item={selectedItem}
                  {qty}
                  onBumpQty={bumpDetailQty}
                  onAdd={() => addToCart(selectedItem)}
                  currencySymbol={restaurant.currencySymbol}
                  {cart}
                  taxRate={restaurant.taxRate}
                  onBumpCart={bumpCart}
                  onPlaceOrder={() => go({ kind: "payment" })}
                />
              {:else}
                <Home
                  {greetingText}
                  {speaking}
                  onToggleHear={toggleHear}
                  {specials}
                  currencySymbol={restaurant.currencySymbol}
                  {cart}
                  taxRate={restaurant.taxRate}
                  onBump={bumpCart}
                  onPlaceOrder={() => go({ kind: "payment" })}
                />
              {/if}
            </div>
          </div>
        {/if}
      {:else if route.kind === "payment"}
        <PaymentScreen
          {cart}
          taxRate={restaurant.taxRate}
          currencySymbol={restaurant.currencySymbol}
          paymentGateways={restaurant.paymentGateways}
          onConfirm={placeOrder}
        />
      {:else if route.kind === "status"}
        <StatusScreen
          {order}
          currencySymbol={restaurant.currencySymbol}
          onReply={replyToKitchen}
          onStartNew={() => go({ kind: "special" })}
        />
      {/if}
    </div>

    <NavDock
      {canBack}
      {canForward}
      onBack={navBack}
      onForward={navForward}
      onHome={() => go({ kind: "special" })}
      onExit={exit}
    />
  </Background>
{/if}

<style>
  .header-stack {
    position: relative;
  }
  .corner {
    position: absolute;
    right: 6px;
    top: 6px;
  }
  .body {
    flex: 1;
    overflow: hidden;
    padding: 10px 14px;
  }
  .split {
    display: flex;
    height: 100%;
    gap: 0;
  }
  .tree-pane {
    width: 300px;
    flex-shrink: 0;
    overflow-y: auto;
  }
  .divider {
    width: 1px;
    background: #eadcff;
    margin: 0 14px;
  }
  .detail-pane {
    flex: 1;
    overflow-y: auto;
  }
</style>
