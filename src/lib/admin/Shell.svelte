<!-- Sidebar + main content wrapper — port of AdminApp._sidebar()/select(). -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { restaurant as restaurantStore } from "../stores.js";
  import Background from "../components/Background.svelte";
  import BusinessHeader from "../components/BusinessHeader.svelte";
  import NavDock from "../components/NavDock.svelte";
  import { hoverGrow } from "../actions.js";

  import Dashboard from "./Dashboard.svelte";
  import MenuSection from "./MenuSection.svelte";
  import SpecialsSection from "./SpecialsSection.svelte";
  import BrandingSection from "./BrandingSection.svelte";
  import PaymentsSection from "./PaymentsSection.svelte";
  import AdminsSection from "./AdminsSection.svelte";
  import NotificationsSection from "./NotificationsSection.svelte";

  export let restaurant;
  export let admin;
  export let onSignOut = () => {};
  export let onExit = () => {};
  export let onRestaurantChanged = () => {};

  const NAV = [
    { key: "dashboard", label: "Dashboard" },
    { key: "menu", label: "Menu" },
    { key: "specials", label: "Specials" },
    { key: "branding", label: "Branding" },
    { key: "payments", label: "Payments" },
    { key: "admins", label: "Admins" },
    { key: "notifications", label: "Notifications" },
  ];

  let section = "dashboard";
  let history = ["dashboard"];
  let historyIndex = 0;
  let unreadCount = 0;

  $: isSuperadmin = admin?.role === "superadmin";
  $: visibleNav = NAV.filter((n) => n.key !== "admins" || isSuperadmin);
  $: canBack = historyIndex > 0;
  $: canForward = historyIndex < history.length - 1;

  onMount(refreshUnread);

  async function refreshUnread() {
    try {
      unreadCount = await api.unreadNotificationCount();
    } catch {
      unreadCount = 0;
    }
  }

  function select(key, record = true) {
    section = key;
    if (record) {
      history = [...history.slice(0, historyIndex + 1), key];
      historyIndex = history.length - 1;
    }
    refreshUnread();
  }

  function navBack() {
    if (!canBack) return;
    historyIndex -= 1;
    section = history[historyIndex];
    refreshUnread();
  }

  function navForward() {
    if (!canForward) return;
    historyIndex += 1;
    section = history[historyIndex];
    refreshUnread();
  }

  async function handleSignOut() {
    await api.logout();
    onSignOut();
  }

  async function handleRestaurantChanged() {
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);
    onRestaurantChanged();
  }
</script>

<Background restaurant={restaurant}>
  <BusinessHeader {restaurant} compact />

  <div class="body">
    <aside class="sidebar">
      <div class="nav-items">
        {#each visibleNav as item (item.key)}
          <button
            class="nav-item"
            class:selected={section === item.key}
            use:hoverGrow={{ color: "var(--pink)" }}
            on:click={() => select(item.key)}
          >
            <span class="nav-label">{item.label}</span>
            {#if item.key === "notifications" && unreadCount > 0}
              <span class="badge">{unreadCount}</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="sidebar-footer">
        <div class="divider" />
        <span class="username" use:hoverGrow={{ color: "var(--pink)" }}>{admin?.username ?? ""}</span>
        <span class="role">{admin?.role ?? ""}</span>
        <button class="sign-out" use:hoverGrow={{ color: "var(--pink)" }} on:click={handleSignOut}>
          <span class="icon">⏻</span>
          <span>Sign out</span>
        </button>
      </div>
    </aside>

    <main class="content">
      {#if section === "dashboard"}
        <Dashboard {restaurant} onRestaurantChanged={handleRestaurantChanged} />
      {:else if section === "menu"}
        <MenuSection {restaurant} />
      {:else if section === "specials"}
        <SpecialsSection {restaurant} />
      {:else if section === "branding"}
        <BrandingSection {restaurant} onRestaurantChanged={handleRestaurantChanged} />
      {:else if section === "payments"}
        <PaymentsSection {restaurant} onRestaurantChanged={handleRestaurantChanged} />
      {:else if section === "admins"}
        <AdminsSection {admin} />
      {:else if section === "notifications"}
        <NotificationsSection onChanged={refreshUnread} />
      {/if}
    </main>
  </div>

  <NavDock
    {canBack}
    {canForward}
    onBack={navBack}
    onForward={navForward}
    onHome={() => select("dashboard")}
    onExit={onExit}
  />
</Background>

<style>
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .sidebar {
    width: 230px;
    flex-shrink: 0;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .nav-items {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 16px 12px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border: none;
    background: transparent;
    border-radius: 10px;
    padding: 12px 16px;
    font-size: 15px;
    font-style: italic;
    font-weight: 600;
    font-family: var(--font-body);
    color: var(--text-muted);
    cursor: pointer;
    text-align: left;
  }
  .nav-item.selected {
    background: var(--surface-2);
    color: var(--brand);
    font-weight: 700;
  }
  .nav-label {
    flex: 1;
  }
  .badge {
    background: var(--accent);
    color: white;
    font-size: 11px;
    font-weight: 700;
    border-radius: 999px;
    padding: 1px 7px;
  }
  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 16px 16px;
  }
  .divider {
    height: 1px;
    background: var(--surface-2);
    margin-bottom: 6px;
  }
  .username {
    font-weight: 700;
    font-size: 14px;
    color: var(--text);
  }
  .role {
    font-size: 11px;
    color: var(--text-muted);
  }
  .sign-out {
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 2px;
    margin-top: 4px;
    align-self: flex-start;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
</style>
