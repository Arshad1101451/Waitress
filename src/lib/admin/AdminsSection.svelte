<!-- Admins section — port of AdminApp._section_admins() and its dialogs. -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { showToast, confirmAction } from "../stores.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";

  export let admin; // current logged-in admin

  $: isSuperadmin = admin?.role === "superadmin";

  let admins = [];

  onMount(() => {
    if (isSuperadmin) load();
  });

  async function load() {
    admins = await api.getAdmins();
  }

  function toggleActive(u, newValue) {
    const want = newValue ? "Enable" : "Disable";
    confirmAction({
      title: `${want} account?`,
      body: `${want} "${u.username}"?`,
      onYes: async () => {
        try {
          await api.setAdminActive(u.id, newValue);
          await load();
        } catch (err) {
          const msg = typeof err === "string" ? err : (err?.message || "Could not update account.");
          showToast(msg, "var(--danger)");
        }
      },
    });
  }

  function deleteAdminAccount(u) {
    confirmAction({
      title: "Delete account?",
      body: `Permanently delete "${u.username}"?`,
      yesLabel: "Delete",
      danger: true,
      onYes: async () => {
        try {
          await api.deleteAdmin(u.id);
          await load();
        } catch (err) {
          const msg = typeof err === "string" ? err : (err?.message || "Could not delete account.");
          showToast(msg, "var(--danger)");
        }
      },
    });
  }

  // ---------------------------------------------------------------- add admin dialog
  let addDialogOpen = false;
  let newUsername = "";
  let newPassword = "";
  let newShowPassword = false;
  let newRole = "admin";
  let addError = "";

  function openAddDialog() {
    newUsername = "";
    newPassword = "";
    newShowPassword = false;
    newRole = "admin";
    addError = "";
    addDialogOpen = true;
  }

  function closeAddDialog() {
    addDialogOpen = false;
  }

  async function createAdminAccount() {
    addError = "";
    try {
      await api.createAdmin(newUsername, newPassword, newRole);
      addDialogOpen = false;
      await load();
    } catch (err) {
      addError = typeof err === "string" ? err : (err?.message || "Could not create account.");
    }
  }

  // ---------------------------------------------------------------- edit admin dialog
  let editDialogOpen = false;
  let editTarget = null;
  let editUsername = "";
  let editRole = "admin";
  let editError = "";

  function openEditDialog(u) {
    editTarget = u;
    editUsername = u.username;
    editRole = u.role;
    editError = "";
    editDialogOpen = true;
  }

  function closeEditDialog() {
    editDialogOpen = false;
  }

  async function submitEditAdmin() {
    editError = "";
    try {
      await api.updateAdmin(editTarget.id, editUsername, editRole);
      editDialogOpen = false;
      await load();
      showToast("Account updated.");
    } catch (err) {
      editError = typeof err === "string" ? err : (err?.message || "Could not update account.");
    }
  }

  // ---------------------------------------------------------------- reset password dialog
  let resetDialogOpen = false;
  let resetTarget = null;
  let resetPassword = "";
  let resetShowPassword = false;
  let resetError = "";

  function openResetDialog(u) {
    resetTarget = u;
    resetPassword = "";
    resetShowPassword = false;
    resetError = "";
    resetDialogOpen = true;
  }

  function closeResetDialog() {
    resetDialogOpen = false;
  }

  async function submitResetPassword() {
    resetError = "";
    try {
      await api.resetAdminPassword(resetTarget.id, resetPassword);
      resetDialogOpen = false;
      showToast("Password updated.");
    } catch (err) {
      resetError = typeof err === "string" ? err : (err?.message || "Could not reset password.");
    }
  }
</script>

{#if !isSuperadmin}
  <Heading text="Admins" size={26} />
  <span class="empty">Only a superadmin can manage accounts.</span>
{:else}
  <div class="title-row">
    <Heading text="Admin accounts" size={26} />
    <div class="spacer" />
    <Button label="Add admin" icon="&#43;" gradient="var(--grad-brand)" width={160} onClick={openAddDialog} />
  </div>

  <GlowCard padding={16}>
    {#if admins.length === 0}
      <span class="empty">No admins.</span>
    {:else}
      {#each admins as u (u.id)}
        <div class="row" use:hoverGrow={{ color: "var(--pink)" }}>
          <span class="role-icon">{u.role === "superadmin" ? "🛡" : "👤"}</span>
          <div class="info">
            <span class="username">{u.username}</span>
            <span class="meta">{u.role}{u.lastLogin ? ` · last login ${u.lastLogin}` : ""}</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={u.active}
              on:click|preventDefault={() => toggleActive(u, !u.active)}
            />
            <span class="slider" />
          </label>
          <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.15 }} title="Edit role" on:click={() => openEditDialog(u)}>✏️</button>
          <button class="text-btn" on:click={() => openResetDialog(u)}>Reset password</button>
          <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.15 }} title="Delete" on:click={() => deleteAdminAccount(u)}>🗑</button>
        </div>
      {/each}
    {/if}
  </GlowCard>
{/if}

{#if addDialogOpen}
  <div class="modal-backdrop" on:click|self={closeAddDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text="New admin" size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="new-username">Username</label>
          <input id="new-username" type="text" bind:value={newUsername} autofocus />
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="new-password">Password</label>
          <div class="password-row">
            {#if newShowPassword}
              <input id="new-password" type="text" bind:value={newPassword} />
            {:else}
              <input id="new-password" type="password" bind:value={newPassword} />
            {/if}
            <button class="toggle" type="button" on:click={() => (newShowPassword = !newShowPassword)}>
              {newShowPassword ? "Hide" : "Show"}
            </button>
          </div>
        </div>
        <div class="radio-group">
          <label class="radio-row">
            <input type="radio" name="new-role" value="admin" bind:group={newRole} />
            Admin
          </label>
          <label class="radio-row">
            <input type="radio" name="new-role" value="superadmin" bind:group={newRole} />
            Superadmin
          </label>
        </div>
        {#if addError}
          <span class="error">{addError}</span>
        {/if}
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeAddDialog} />
          <Button label="Create" gradient="var(--grad-brand)" width={100} onClick={createAdminAccount} />
        </div>
      </GlowCard>
    </div>
  </div>
{/if}

{#if editDialogOpen}
  <div class="modal-backdrop" on:click|self={closeEditDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text="Edit admin" size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="edit-username">Username</label>
          <input id="edit-username" type="text" bind:value={editUsername} autofocus />
        </div>
        <div class="radio-group">
          <label class="radio-row">
            <input type="radio" name="edit-role" value="admin" bind:group={editRole} />
            Admin
          </label>
          <label class="radio-row">
            <input type="radio" name="edit-role" value="superadmin" bind:group={editRole} />
            Superadmin
          </label>
        </div>
        {#if editError}
          <span class="error">{editError}</span>
        {/if}
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeEditDialog} />
          <Button label="Save" gradient="var(--grad-brand)" width={100} onClick={submitEditAdmin} />
        </div>
      </GlowCard>
    </div>
  </div>
{/if}

{#if resetDialogOpen}
  <div class="modal-backdrop" on:click|self={closeResetDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text="Reset password" size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="reset-password">New password for {resetTarget?.username}</label>
          <div class="password-row">
            {#if resetShowPassword}
              <input id="reset-password" type="text" bind:value={resetPassword} autofocus />
            {:else}
              <input id="reset-password" type="password" bind:value={resetPassword} autofocus />
            {/if}
            <button class="toggle" type="button" on:click={() => (resetShowPassword = !resetShowPassword)}>
              {resetShowPassword ? "Hide" : "Show"}
            </button>
          </div>
        </div>
        {#if resetError}
          <span class="error">{resetError}</span>
        {/if}
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeResetDialog} />
          <Button label="Update" gradient="var(--grad-brand)" width={100} onClick={submitResetPassword} />
        </div>
      </GlowCard>
    </div>
  </div>
{/if}

<style>
  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .spacer {
    flex: 1;
  }
  .empty {
    color: var(--text-muted);
    font-style: italic;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
  }
  .role-icon {
    font-size: 18px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }
  .username {
    font-weight: 700;
    color: var(--text);
    font-size: 14px;
  }
  .meta {
    color: var(--text-muted);
    font-size: 11px;
  }
  .text-btn {
    border: none;
    background: transparent;
    color: var(--brand);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  .icon-btn {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 8px;
  }
  .icon-btn.danger:hover {
    color: var(--danger);
    background: var(--surface-2);
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
  .radio-group {
    display: flex;
    gap: 20px;
    margin-bottom: 10px;
  }
  .radio-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    color: var(--text);
  }
  .error {
    color: var(--danger);
    font-size: 12px;
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
    width: min(400px, 92vw);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 10px;
  }
</style>
