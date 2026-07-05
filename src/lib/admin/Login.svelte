<!-- Admin sign-in — port of AdminApp._view_login(). -->
<script>
  import { api, assetUrl } from "../api.js";
  import Button from "../components/Button.svelte";
  import GlowCard from "../components/GlowCard.svelte";
  import { hoverGrow } from "../actions.js";

  export let restaurant;
  export let onLoggedIn = () => {};

  let username = "";
  let password = "";
  let showPassword = false;
  let error = "";
  let busy = false;

  let logoSrc = "";
  $: (async () => {
    logoSrc = restaurant?.logoPath ? await assetUrl(restaurant.logoPath) : "";
  })();

  async function doLogin() {
    error = "";
    busy = true;
    try {
      await api.login(username, password);
      onLoggedIn();
    } catch (err) {
      error = typeof err === "string" ? err : (err?.message || "Login failed.");
    } finally {
      busy = false;
    }
  }

  function onKeydown(e) {
    if (e.key === "Enter") doLogin();
  }
</script>

<div class="wrap">
  <GlowCard padding={28} radius={18}>
    <div class="card-body">
      <div class="header-row">
        {#if logoSrc}
          <img class="logo" src={logoSrc} alt="logo" />
        {/if}
        <span class="title">{restaurant?.name ?? "Your Restaurant"}</span>
      </div>
      <span class="subtitle">Admin sign in</span>

      <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
        <label for="username">Username</label>
        <input id="username" type="text" bind:value={username} on:keydown={onKeydown} autofocus />
      </div>

      <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
        <label for="password">Password</label>
        <div class="password-row">
          {#if showPassword}
            <input id="password" type="text" bind:value={password} on:keydown={onKeydown} />
          {:else}
            <input id="password" type="password" bind:value={password} on:keydown={onKeydown} />
          {/if}
          <button class="toggle" type="button" on:click={() => (showPassword = !showPassword)}>
            {showPassword ? "Hide" : "Show"}
          </button>
        </div>
      </div>

      {#if error}
        <span class="error">{error}</span>
      {/if}

      <Button label="Sign in" gradient="var(--grad-brand)" height={46} expand
        disabled={busy} onClick={doLogin} />
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
    width: 340px;
  }
  .header-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    object-fit: cover;
  }
  .title {
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
  }
  .subtitle {
    font-size: 15px;
    color: var(--text-muted);
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
    width: 100%;
  }
  .field input:focus {
    outline: 2px solid var(--brand);
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
  .error {
    color: var(--danger);
    font-size: 12px;
  }
</style>
