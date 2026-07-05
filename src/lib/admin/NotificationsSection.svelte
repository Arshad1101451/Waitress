<!-- Notifications section — port of AdminApp._section_notifications(). -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { showToast } from "../stores.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";

  export let onChanged = () => {};

  let notifications = [];

  onMount(load);

  async function load() {
    notifications = await api.getNotifications(false, 100);
  }

  async function markRead(n) {
    // No toast — matches the original, this one action is silent.
    await api.markNotificationRead(n.id);
    await load();
    onChanged();
  }

  async function markAllRead() {
    await api.markAllNotificationsRead();
    showToast("All notifications marked read.");
    await load();
    onChanged();
  }

  async function refresh() {
    await load();
  }
</script>

<div class="title-row">
  <Heading text="Notifications" size={26} />
  <div class="spacer" />
  <Button label="Mark all read" icon="&#10003;" gradient="var(--grad-brand)" width={170} onClick={markAllRead} />
  <Button label="Refresh" icon="&#8635;" bgcolor="var(--surface-2)" textColor="var(--text)" width={130} onClick={refresh} />
</div>

{#if notifications.length === 0}
  <span class="empty">No notifications yet. Payments will appear here.</span>
{:else}
  {#each notifications as n (n.id)}
    <GlowCard padding={12}>
      <div class="row" class:unread={!n.read} use:hoverGrow={{ color: "var(--pink)" }}>
        <span class="icon">{n.kind === "payment" ? "💳" : "🔔"}</span>
        <div class="info">
          <span class="title" class:bold={!n.read}>{n.title}</span>
          <span class="body">{n.body}</span>
          <span class="time">{n.createdAt}</span>
        </div>
        {#if !n.read}
          <button class="text-btn" on:click={() => markRead(n)}>Mark read</button>
        {/if}
      </div>
    </GlowCard>
  {/each}
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
    border-radius: 12px;
    padding: 4px;
  }
  .row.unread {
    background: var(--surface-2);
  }
  .icon {
    font-size: 18px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 14px;
    color: var(--text);
  }
  .title.bold {
    font-weight: 700;
  }
  .body {
    font-size: 12px;
    color: var(--text-muted);
  }
  .time {
    font-size: 10px;
    color: var(--text-muted);
  }
  .text-btn {
    border: none;
    background: transparent;
    color: var(--brand);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
  }
</style>
