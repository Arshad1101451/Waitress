<!-- Payments section — port of AdminApp._section_payments()/_gw_*() helpers. -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { showToast, confirmAction, restaurant as restaurantStore } from "../stores.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";

  export let restaurant;
  export let onRestaurantChanged = () => {};

  const REALTIME_GATEWAYS = new Set(["credit", "debit", "paypal"]);

  let master = []; // ordered master list
  let enabled = new Set();
  let gwConfig = {}; // { [nameLower]: { apiKey, secretKey, merchantId } }
  let newGatewayName = "";

  let kitchenDisplayMode = "tablet";
  let notifyOnPayment = false;
  let voiceGreetingEnabled = false;

  onMount(load);

  function parseMaster(r) {
    const seen = new Set();
    const list = [];
    for (const raw of (r?.paymentGatewaysAll ?? "").split(",")) {
      const g = raw.trim();
      if (g && !seen.has(g.toLowerCase())) {
        seen.add(g.toLowerCase());
        list.push(g);
      }
    }
    return list;
  }

  function parseEnabled(r) {
    return new Set(
      (r?.paymentGateways ?? "")
        .split(",")
        .map((x) => x.trim())
        .filter(Boolean)
    );
  }

  function parseConfig(r) {
    try {
      const data = JSON.parse(r?.paymentGatewayConfig || "{}");
      return data && typeof data === "object" ? data : {};
    } catch {
      return {};
    }
  }

  async function load() {
    restaurant = await api.getRestaurant();
    master = parseMaster(restaurant);
    enabled = parseEnabled(restaurant);
    gwConfig = parseConfig(restaurant);
    kitchenDisplayMode = restaurant.kitchenDisplayMode || "tablet";
    notifyOnPayment = !!restaurant.notifyOnPayment;
    voiceGreetingEnabled = !!restaurant.voiceGreetingEnabled;
  }

  async function persistGateways(newMaster, newEnabled, msg) {
    const latest = await api.getRestaurant();
    const updated = {
      ...latest,
      paymentGatewaysAll: newMaster.join(","),
      paymentGateways: newMaster.filter((g) => newEnabled.has(g)).join(","),
    };
    await api.saveRestaurant(updated);
    await refreshAndNotify();
    if (msg) showToast(msg);
  }

  async function persistConfig(newConfig) {
    const latest = await api.getRestaurant();
    const updated = { ...latest, paymentGatewayConfig: JSON.stringify(newConfig) };
    await api.saveRestaurant(updated);
    await refreshAndNotify();
  }

  async function refreshAndNotify() {
    const saved = await api.getRestaurant();
    restaurant = saved;
    restaurantStore.set(saved);
    master = parseMaster(saved);
    enabled = parseEnabled(saved);
    gwConfig = parseConfig(saved);
    onRestaurantChanged();
  }

  // ------------------------------------------------------------- toggle (instant)
  async function toggleGateway(name, value) {
    const newEnabled = new Set(enabled);
    if (value) newEnabled.add(name);
    else newEnabled.delete(name);
    enabled = newEnabled;
    await persistGateways(master, newEnabled, `${name} ${value ? "shown to customers" : "hidden"}`);
  }

  // ------------------------------------------------------------- add gateway
  async function addGateway() {
    const name = newGatewayName.trim();
    if (!name) {
      showToast("Enter a gateway name.", "var(--danger)");
      return;
    }
    if (master.some((g) => g.toLowerCase() === name.toLowerCase())) {
      showToast("That gateway already exists.", "var(--danger)");
      return;
    }
    const newMaster = [...master, name];
    const newEnabled = new Set(enabled);
    newEnabled.add(name);
    newGatewayName = "";
    await persistGateways(newMaster, newEnabled, `Added ${name}.`);
  }

  // ------------------------------------------------------------- remove gateway
  function removeGateway(name) {
    confirmAction({
      title: "Remove gateway?",
      body: `Remove "${name}" as a payment option?`,
      yesLabel: "Remove",
      danger: true,
      onYes: async () => {
        const newMaster = master.filter((g) => g !== name);
        const newEnabled = new Set(enabled);
        newEnabled.delete(name);
        await persistGateways(newMaster, newEnabled, `Removed ${name}.`);
      },
    });
  }

  // ------------------------------------------------------------- configure/rename dialog
  let gwDialogOpen = false;
  let gwDialogName = "";
  let gwDialogOriginalName = "";
  let gwDialogNeedsConfig = false;
  let gwApiKey = "";
  let gwSecretKey = "";
  let gwMerchantId = "";
  let gwShowApiKey = false;
  let gwShowSecretKey = false;

  function openGwDialog(name) {
    gwDialogOriginalName = name;
    gwDialogName = name;
    gwDialogNeedsConfig = REALTIME_GATEWAYS.has(name.toLowerCase());
    const cfg = gwConfig[name.toLowerCase()] || {};
    gwApiKey = cfg.apiKey || "";
    gwSecretKey = cfg.secretKey || "";
    gwMerchantId = cfg.merchantId || "";
    gwShowApiKey = false;
    gwShowSecretKey = false;
    gwDialogOpen = true;
  }

  function closeGwDialog() {
    gwDialogOpen = false;
  }

  async function saveGwDialog() {
    const newName = gwDialogName.trim();
    if (!newName) {
      closeGwDialog();
      return;
    }
    const oldName = gwDialogOriginalName;
    const renamed = newName !== oldName;

    let newMaster = master;
    let newEnabled = new Set(enabled);
    if (renamed) {
      newMaster = master.map((g) => (g === oldName ? newName : g));
      if (enabled.has(oldName)) {
        newEnabled.delete(oldName);
        newEnabled.add(newName);
      }
    }

    const newConfig = { ...gwConfig };
    if (gwDialogNeedsConfig) {
      newConfig[newName.toLowerCase()] = {
        apiKey: gwApiKey.trim(),
        secretKey: gwSecretKey.trim(),
        merchantId: gwMerchantId.trim(),
      };
      if (renamed) delete newConfig[oldName.toLowerCase()];
    } else if (renamed && newConfig[oldName.toLowerCase()]) {
      newConfig[newName.toLowerCase()] = newConfig[oldName.toLowerCase()];
      delete newConfig[oldName.toLowerCase()];
    }

    gwDialogOpen = false;
    const latest = await api.getRestaurant();
    const updated = {
      ...latest,
      paymentGatewaysAll: newMaster.join(","),
      paymentGateways: newMaster.filter((g) => newEnabled.has(g)).join(","),
      paymentGatewayConfig: JSON.stringify(newConfig),
    };
    await api.saveRestaurant(updated);
    await refreshAndNotify();
    showToast(gwDialogNeedsConfig ? "Gateway configuration saved." : "Gateway renamed.");
  }

  function isConfigured(name) {
    const cfg = gwConfig[name.toLowerCase()] || {};
    return !!cfg.apiKey;
  }

  // ------------------------------------------------------------- device settings save
  async function saveDeviceSettings() {
    const latest = await api.getRestaurant();
    const updated = {
      ...latest,
      kitchenDisplayMode,
      printingEnabled: kitchenDisplayMode !== "tablet",
      notifyOnPayment,
      voiceGreetingEnabled,
    };
    await api.saveRestaurant(updated);
    await refreshAndNotify();
    showToast("Device settings saved.");
  }
</script>

<Heading text="Payments &amp; Devices" size={26} />

<GlowCard padding={16}>
  <span class="card-title">Payment gateways</span>
  <span class="card-subtitle">Ticked gateways appear to customers at checkout. Changes save instantly.</span>

  {#if master.length === 0}
    <span class="empty">No gateways yet — add one below.</span>
  {:else}
    {#each master as g (g)}
      <div class="gw-row" use:hoverGrow={{ color: "var(--pink)" }}>
        <label class="checkbox-row">
          <input
            type="checkbox"
            checked={enabled.has(g)}
            on:change={(e) => toggleGateway(g, e.target.checked)}
          />
          {g.charAt(0).toUpperCase() + g.slice(1)}
        </label>
        {#if REALTIME_GATEWAYS.has(g.toLowerCase())}
          <span class="status" class:ok={isConfigured(g)} class:bad={!isConfigured(g)}>
            {isConfigured(g) ? "● Configured" : "● Needs setup"}
          </span>
        {/if}
        <div class="spacer" />
        <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.18 }}
          title={REALTIME_GATEWAYS.has(g.toLowerCase()) ? "Configure" : "Rename"}
          on:click={() => openGwDialog(g)}>
          {REALTIME_GATEWAYS.has(g.toLowerCase()) ? "⚙️" : "✏️"}
        </button>
        <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.18 }} title="Remove" on:click={() => removeGateway(g)}>🗑</button>
      </div>
    {/each}
  {/if}

  <div class="divider" />
  <div class="add-row">
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="new-gw">New gateway name</label>
      <input id="new-gw" type="text" bind:value={newGatewayName}
        on:keydown={(e) => e.key === "Enter" && addGateway()} />
    </div>
    <Button label="Add gateway" icon="&#43;" gradient="var(--grad-brand)" width={170} onClick={addGateway} />
  </div>
</GlowCard>

<GlowCard padding={16}>
  <div class="row-header">
    <span class="card-title">Device settings</span>
    <div class="spacer" />
    <Button label="Save" icon="&#128190;" gradient="var(--grad-sunny)" width={140} onClick={saveDeviceSettings} />
  </div>
  <span class="card-subtitle">Kitchen order display</span>
  <div class="radio-group">
    <label class="radio-row" use:hoverGrow={{ color: "var(--brand)" }}>
      <input type="radio" name="kdm" value="tablet" bind:group={kitchenDisplayMode} />
      Tablet only
    </label>
    <label class="radio-row" use:hoverGrow={{ color: "var(--brand)" }}>
      <input type="radio" name="kdm" value="printer" bind:group={kitchenDisplayMode} />
      Printer only
    </label>
    <label class="radio-row" use:hoverGrow={{ color: "var(--brand)" }}>
      <input type="radio" name="kdm" value="both" bind:group={kitchenDisplayMode} />
      Tablet + printer
    </label>
  </div>
  <div class="divider" />
  <label class="switch-row" use:hoverGrow={{ color: "var(--pink)" }}>
    <label class="switch">
      <input type="checkbox" bind:checked={notifyOnPayment} />
      <span class="slider" />
    </label>
    Notify me when a customer pays
  </label>
  <label class="switch-row" use:hoverGrow={{ color: "var(--pink)" }}>
    <label class="switch">
      <input type="checkbox" bind:checked={voiceGreetingEnabled} />
      <span class="slider" />
    </label>
    Waitress voice greeting on customer tables
  </label>
</GlowCard>

{#if gwDialogOpen}
  <div class="modal-backdrop" on:click|self={closeGwDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading
          text={gwDialogNeedsConfig ? `Configure ${gwDialogOriginalName.charAt(0).toUpperCase() + gwDialogOriginalName.slice(1)}` : "Rename gateway"}
          size={18}
        />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="gw-name">Gateway name</label>
          <input id="gw-name" type="text" bind:value={gwDialogName} autofocus />
        </div>
        {#if gwDialogNeedsConfig}
          <span class="realtime-caption">
            Realtime processor configuration — used to charge customers live through this gateway.
          </span>
          <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
            <label for="gw-apikey">API Key</label>
            <div class="password-row">
              {#if gwShowApiKey}
                <input id="gw-apikey" type="text" bind:value={gwApiKey} />
              {:else}
                <input id="gw-apikey" type="password" bind:value={gwApiKey} />
              {/if}
              <button class="toggle" type="button" on:click={() => (gwShowApiKey = !gwShowApiKey)}>
                {gwShowApiKey ? "Hide" : "Show"}
              </button>
            </div>
          </div>
          <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
            <label for="gw-secret">Secret Key</label>
            <div class="password-row">
              {#if gwShowSecretKey}
                <input id="gw-secret" type="text" bind:value={gwSecretKey} />
              {:else}
                <input id="gw-secret" type="password" bind:value={gwSecretKey} />
              {/if}
              <button class="toggle" type="button" on:click={() => (gwShowSecretKey = !gwShowSecretKey)}>
                {gwShowSecretKey ? "Hide" : "Show"}
              </button>
            </div>
          </div>
          <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
            <label for="gw-merchant">Merchant ID</label>
            <input id="gw-merchant" type="text" bind:value={gwMerchantId} />
          </div>
        {/if}
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeGwDialog} />
          <Button label="Save" gradient="var(--grad-brand)" width={100} onClick={saveGwDialog} />
        </div>
      </GlowCard>
    </div>
  </div>
{/if}

<style>
  .spacer {
    flex: 1;
  }
  .card-title {
    font-weight: 700;
    color: var(--text);
    display: block;
    margin-bottom: 4px;
  }
  .card-subtitle {
    color: var(--text-muted);
    font-size: 12px;
    display: block;
    margin-bottom: 10px;
  }
  .empty {
    color: var(--text-muted);
    font-style: italic;
  }
  .gw-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
  }
  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--text);
  }
  .status {
    font-size: 11px;
    font-style: italic;
  }
  .status.ok {
    color: var(--success);
  }
  .status.bad {
    color: var(--danger);
  }
  .icon-btn {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 19px;
    cursor: pointer;
    padding: 5px 9px;
    border-radius: 8px;
  }
  .icon-btn:hover {
    background: var(--surface-2);
    color: var(--brand);
  }
  .icon-btn.danger:hover {
    color: var(--danger);
  }
  .divider {
    height: 1px;
    background: var(--surface-2);
    margin: 12px 0;
  }
  .add-row {
    display: flex;
    align-items: flex-end;
    gap: 12px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
  }
  .field label {
    font-size: 12px;
    color: var(--text-muted);
  }
  .field input {
    border-radius: 10px;
    border: 1px solid #e0d4fa;
    padding: 8px 12px;
    font-size: 14px;
    font-family: var(--font-body);
    color: var(--text);
    width: 100%;
  }
  .field input:focus {
    outline: 2px solid var(--brand);
  }
  .row-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .radio-group {
    display: flex;
    gap: 20px;
    margin-top: 6px;
  }
  .radio-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: var(--text);
  }
  .switch-row {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: var(--text);
    margin-bottom: 10px;
  }
  .switch {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 22px;
    flex-shrink: 0;
  }
  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: #ccc;
    border-radius: 999px;
    transition: 0.15s;
  }
  .slider::before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 3px;
    bottom: 3px;
    background: white;
    border-radius: 50%;
    transition: 0.15s;
  }
  .switch input:checked + .slider {
    background: var(--success);
  }
  .switch input:checked + .slider::before {
    transform: translateX(18px);
  }
  .realtime-caption {
    font-size: 11px;
    font-style: italic;
    color: var(--text-muted);
    display: block;
    margin-bottom: 10px;
  }
  .password-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .password-row input {
    flex: 1;
  }
  .toggle {
    border: none;
    background: transparent;
    color: var(--brand);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(51, 35, 91, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9000;
  }
  .modal {
    width: min(420px, 92vw);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 10px;
  }
</style>
