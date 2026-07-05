<!-- Activation gate — port of AdminApp._view_activation(). -->
<script>
  import { api } from "../api.js";
  import { showToast } from "../stores.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Button from "../components/Button.svelte";
  import { hoverGrow } from "../actions.js";

  export let restaurant;
  export let onActivated = () => {};

  let code = "";
  let error = "";
  let busy = false;
  let deviceId = "";

  (async () => {
    deviceId = await api.deviceId();
  })();

  async function activate() {
    error = "";
    busy = true;
    try {
      const message = await api.activate(code);
      showToast(message);
      onActivated();
    } catch (err) {
      error = typeof err === "string" ? err : (err?.message || "Activation failed.");
    } finally {
      busy = false;
    }
  }
</script>

<div class="wrap">
  <GlowCard padding={28} radius={18}>
    <div class="card-body">
      <div class="header-row">
        <span class="lock-icon">🔒</span>
        <span class="title">Activate Project Waitress</span>
      </div>
      <p class="desc">
        Enter the activation code issued for this device. Each code activates a single device.
      </p>
      <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
        <label for="activation-code">Activation code</label>
        <input
          id="activation-code"
          type="text"
          placeholder="WTRS-XXXXX-XXXXX-XXXXX-XXXXX"
          bind:value={code}
          on:keydown={(e) => e.key === "Enter" && activate()}
        />
      </div>
      {#if error}
        <span class="error">{error}</span>
      {/if}
      <Button label="Activate" gradient="var(--grad-brand)" height={46} expand
        disabled={busy} onClick={activate} />
      <span class="device-id">Device ID: {deviceId}</span>
    </div>
  </GlowCard>
</div>

<style>
  .wrap {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
  }
  .card-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 380px;
  }
  .header-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .lock-icon {
    font-size: 20px;
    color: var(--brand);
  }
  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
  }
  .desc {
    color: var(--text-muted);
    font-size: 13px;
    margin: 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field label {
    font-size: 12px;
    color: var(--text-muted);
  }
  .field input {
    height: 42px;
    border-radius: 10px;
    border: 1px solid #e0d4fa;
    padding: 0 12px;
    font-size: 14px;
    font-family: var(--font-body);
    color: var(--text);
  }
  .field input:focus {
    outline: 2px solid var(--brand);
  }
  .error {
    color: var(--danger);
    font-size: 12px;
  }
  .device-id {
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
