<!-- Unified launcher — port of waitress/launcher/app.py. Bundles Customer,
     Kitchen, and Admin behind a role picker, with a floating role-switch
     control. Admin session persistence across role switches is handled by
     the Rust backend's in-memory `admin_session` (see state.rs) — it
     survives regardless of which Svelte component is mounted, so this
     component doesn't need to keep AdminApp "alive" itself. -->
<script>
  import { onMount } from "svelte";
  import { api } from "./lib/api.js";
  import { restaurant as restaurantStore, deviceConfig } from "./lib/stores.js";
  import Heading from "./lib/components/Heading.svelte";
  import Background from "./lib/components/Background.svelte";
  import ToastHost from "./lib/components/ToastHost.svelte";
  import ConfirmDialog from "./lib/components/ConfirmDialog.svelte";
  import CustomerApp from "./lib/customer/CustomerApp.svelte";
  import AdminApp from "./lib/admin/AdminApp.svelte";
  import KitchenApp from "./lib/kitchen/KitchenApp.svelte";

  const ROLES = [
    { key: "customer", title: "Customer / Table", subtitle: "Guests browse the menu, order, and pay at the table.", icon: "🍽" },
    { key: "kitchen", title: "Kitchen", subtitle: "Live order board — accept, prepare, and message tables.", icon: "🧑‍🍳" },
    { key: "admin", title: "Admin", subtitle: "Activate, configure menus & branding, view sales reports.", icon: "⚙️" },
  ];

  let role = null; // null = picker screen
  let restaurant = null;
  let cfg = null;
  let ready = false;

  onMount(async () => {
    await api.ensureSeeded();
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);
    cfg = await api.getDeviceConfig();
    deviceConfig.set(cfg);
    ready = true;
    if (cfg.roleLocked) {
      role = cfg.role;
    }
  });

  async function launch(key) {
    cfg = await api.updateDeviceConfig({ role: key });
    deviceConfig.set(cfg);
    role = key;
  }

  function showPicker() {
    role = null;
  }
</script>

{#if ready}
  {#if role === "customer"}
    <CustomerApp onExit={showPicker} />
  {:else if role === "kitchen"}
    <KitchenApp onExit={showPicker} />
  {:else if role === "admin"}
    <AdminApp onExit={showPicker} />
  {:else}
    <Background restaurant={restaurant}>
      <div class="picker">
        <div class="picker-header">
          <Heading text="Waitress" size={36} />
          <p class="tagline">Choose how this device will be used.</p>
        </div>
        <div class="cards">
          {#each ROLES as r (r.key)}
            <button class="role-card" on:click={() => launch(r.key)}>
              <span class="icon">{r.icon}</span>
              <Heading text={r.title} size={20} color="var(--purple)" />
              <span class="subtitle">{r.subtitle}</span>
            </button>
          {/each}
        </div>
        <span class="device">Device: {cfg?.deviceId ?? ""}</span>
      </div>
    </Background>
  {/if}
{/if}

<ToastHost />
<ConfirmDialog />

<style>
  .picker {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    gap: 20px;
  }
  .picker-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .tagline {
    font-style: italic;
    color: var(--text-muted);
    margin: 0;
  }
  .cards {
    display: flex;
    flex-wrap: wrap;
    gap: 18px;
    justify-content: center;
  }
  .role-card {
    width: 300px;
    height: 180px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    background: var(--surface);
    border: none;
    border-radius: var(--radius);
    box-shadow: var(--shadow-soft);
    cursor: pointer;
    padding: 18px;
    transition: transform 130ms ease-out, box-shadow 130ms ease-out;
  }
  .role-card:hover {
    transform: scale(1.03);
    box-shadow: var(--shadow-strong);
  }
  .role-card .icon {
    font-size: 40px;
  }
  .role-card .subtitle {
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
    text-align: center;
  }
  .device {
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
