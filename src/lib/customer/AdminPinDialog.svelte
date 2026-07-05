<!-- Padlock-icon-triggered admin access, present on every customer screen.
     Port of CustomerApp._prompt_admin_pin / _open_admin_panel. -->
<script>
  import { deviceConfig } from "../stores.js";
  import { api } from "../api.js";
  import { showToast } from "../stores.js";
  import Button from "../components/Button.svelte";
  import { hoverGrow } from "../actions.js";

  export let onReload = () => {}; // reload settings + go home
  export let onExitKiosk = () => {};

  const MAX_ATTEMPTS = 3;
  const LOCK_MS = 30 * 60 * 1000; // 30 minutes

  let stage = null; // null | "confirm" | "pin" | "locked" | "panel"
  let pin = "";
  let error = "";
  let lockUntil = null; // ms timestamp, for the "locked" screen's message

  function lockMinutesLeft() {
    if (!lockUntil) return 0;
    return Math.max(1, Math.ceil((lockUntil - Date.now()) / 60000));
  }

  export async function open() {
    // Re-fetch fresh — this throttle needs to hold even if the device was
    // restarted since the last failed attempt, so it's persisted server-side
    // in DeviceConfig rather than trusted from local component state alone.
    const cfg = await api.getDeviceConfig();
    deviceConfig.set(cfg);

    const now = Date.now();
    if (cfg.adminLockUntil && now < cfg.adminLockUntil) {
      lockUntil = cfg.adminLockUntil;
      stage = "locked";
      return;
    }
    if (cfg.adminLockUntil && now >= cfg.adminLockUntil) {
      // Lock has expired — clear it so the next failed run gets a fresh
      // count of 3, rather than instantly re-locking on attempt 1.
      const cleared = await api.updateDeviceConfig({ adminLoginAttempts: 0, adminLockUntil: null });
      deviceConfig.set(cleared);
    }
    pin = "";
    error = "";
    stage = "confirm";
  }

  function confirmYes() {
    pin = "";
    error = "";
    stage = "pin";
  }

  async function submitPin() {
    if (pin === $deviceConfig?.adminPin) {
      const cleared = await api.updateDeviceConfig({ adminLoginAttempts: 0, adminLockUntil: null });
      deviceConfig.set(cleared);
      error = "";
      stage = "panel";
      return;
    }

    const attempts = ($deviceConfig?.adminLoginAttempts ?? 0) + 1;
    if (attempts >= MAX_ATTEMPTS) {
      const until = Date.now() + LOCK_MS;
      const locked = await api.updateDeviceConfig({ adminLoginAttempts: attempts, adminLockUntil: until });
      deviceConfig.set(locked);
      lockUntil = until;
      stage = "locked";
    } else {
      const updated = await api.updateDeviceConfig({ adminLoginAttempts: attempts });
      deviceConfig.set(updated);
      error = `Incorrect PIN — attempt ${attempts} of ${MAX_ATTEMPTS}`;
    }
  }

  function close() {
    stage = null;
  }

  async function reload() {
    close();
    await onReload();
    showToast("Reloaded settings", "var(--brand)");
  }

  async function exitKiosk() {
    close();
    await onExitKiosk();
    showToast("Kiosk mode released", "var(--brand)");
  }
</script>

<button class="lock-btn" use:hoverGrow={{ color: "white", scale: 1.15 }} on:click={open} title="Admin">🔒</button>

{#if stage === "confirm"}
  <div class="backdrop" on:click|self={close}>
    <div class="modal">
      <h3>Admin login?</h3>
      <p class="muted">Do you want to log in as admin?</p>
      <div class="actions">
        <button class="ghost" use:hoverGrow={{ color: "var(--brand)" }} on:click={close}>No</button>
        <Button label="Yes" gradient="var(--grad-brand)" onClick={confirmYes} />
      </div>
    </div>
  </div>
{:else if stage === "pin"}
  <div class="backdrop" on:click|self={close}>
    <div class="modal">
      <h3>Admin access</h3>
      <input
        type="password"
        placeholder="Admin PIN"
        bind:value={pin}
        autofocus
        on:keydown={(e) => e.key === "Enter" && submitPin()}
      />
      {#if error}<p class="error">{error}</p>{/if}
      <div class="actions">
        <button class="ghost" use:hoverGrow={{ color: "var(--brand)" }} on:click={close}>Cancel</button>
        <Button label="Unlock" gradient="var(--grad-brand)" onClick={submitPin} />
      </div>
    </div>
  </div>
{:else if stage === "locked"}
  <div class="backdrop" on:click|self={close}>
    <div class="modal">
      <h3>Admin login locked</h3>
      <p class="muted">
        Too many incorrect PIN attempts. Admin login is locked — please try again in
        {lockMinutesLeft()} minute{lockMinutesLeft() === 1 ? "" : "s"}.
      </p>
      <div class="actions">
        <button class="ghost" use:hoverGrow={{ color: "var(--brand)" }} on:click={close}>Close</button>
      </div>
    </div>
  </div>
{:else if stage === "panel"}
  <div class="backdrop" on:click|self={close}>
    <div class="modal">
      <h3>Admin panel</h3>
      <p class="muted">Device: {$deviceConfig?.deviceId ?? ""}</p>
      <p class="muted">Table: {$deviceConfig?.tableId || "—"}</p>
      <hr />
      <div class="stack">
        <Button label="Reload settings" gradient="var(--grad-brand)" expand onClick={reload} icon="&#8635;" />
        <Button label="Exit kiosk mode" gradient="var(--grad-sunny)" expand onClick={exitKiosk} icon="&#128275;" />
      </div>
      <div class="actions">
        <button class="ghost" use:hoverGrow={{ color: "var(--brand)" }} on:click={close}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .lock-btn {
    background: rgba(255, 255, 255, 0.2);
    border: none;
    border-radius: 50%;
    width: 32px;
    height: 32px;
    color: rgba(255, 255, 255, 0.87);
    cursor: pointer;
    font-size: 14px;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(51, 35, 91, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10001;
  }
  .modal {
    background: var(--surface);
    border-radius: 16px;
    padding: 24px;
    width: min(360px, 90vw);
    box-shadow: var(--shadow-strong);
  }
  h3 {
    margin: 0 0 14px;
    font-family: var(--font-display);
    font-style: italic;
    color: var(--purple);
  }
  input {
    width: 100%;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--surface-2);
    font-size: 14px;
    margin-bottom: 8px;
  }
  .error {
    color: var(--danger);
    font-size: 12px;
    margin: 0 0 8px;
  }
  .muted {
    color: var(--text-muted);
    font-size: 12px;
    margin: 2px 0;
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 12px 0;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 8px;
  }
  .ghost {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 8px 14px;
  }
</style>
