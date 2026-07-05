<!-- Specials section — port of AdminApp._section_specials() and its dialog. -->
<script>
  import { onMount } from "svelte";
  import { api, assetUrl } from "../api.js";
  import { showToast, confirmAction } from "../stores.js";
  import { money } from "../format.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";

  export let restaurant;

  $: sym = restaurant?.currencySymbol ?? "$";

  let specials = [];
  let imgCache = {};

  onMount(load);

  async function load() {
    specials = await api.getAllSpecials();
  }

  async function imgSrc(path) {
    if (!path) return "";
    if (imgCache[path]) return imgCache[path];
    const url = await assetUrl(path);
    imgCache[path] = url;
    return url;
  }

  function toggleActive(sp, newValue) {
    const want = newValue ? "Show" : "Hide";
    confirmAction({
      title: `${want} special?`,
      body: `${want} "${sp.title}" on customer home screens?`,
      onYes: async () => {
        await api.updateSpecial({ ...sp, active: newValue });
        await load();
      },
    });
  }

  function deleteSpecial(sp) {
    confirmAction({
      title: "Delete special?",
      body: `Permanently delete "${sp.title}"?`,
      yesLabel: "Delete",
      danger: true,
      onYes: async () => {
        await api.deleteSpecial(sp.id);
        showToast(`Deleted ${sp.title}`);
        await load();
      },
    });
  }

  // ---------------------------------------------------------------- dialog
  let dialogOpen = false;
  let editingSpecial = null;
  let title = "";
  let description = "";
  let price = "0.00";
  let activeDate = "";
  let imagePath = "";
  let imagePreview = "";

  function openAdd() {
    editingSpecial = null;
    title = "";
    description = "";
    price = "0.00";
    activeDate = "";
    imagePath = "";
    imagePreview = "";
    dialogOpen = true;
  }

  async function openEdit(sp) {
    editingSpecial = sp;
    title = sp.title;
    description = sp.description || "";
    price = String(sp.price ?? "0.00");
    activeDate = sp.activeDate || "";
    imagePath = sp.imagePath || "";
    imagePreview = imagePath ? await imgSrc(imagePath) : "";
    dialogOpen = true;
  }

  function closeDialog() {
    dialogOpen = false;
  }

  async function pickImage() {
    const path = await api.pickImageFile();
    if (!path) return;
    // No dedicated copy-special-image command — reuse copyMenuImage, both
    // land in the same menu_images folder.
    const newPath = await api.copyMenuImage(path);
    imagePath = newPath;
    imagePreview = await imgSrc(newPath);
  }

  async function save() {
    if (!title.trim()) {
      showToast("Title is required.", "var(--danger)");
      return;
    }
    const priceNum = Number(price);
    if (Number.isNaN(priceNum)) {
      showToast("Price must be a number.", "var(--danger)");
      return;
    }
    if (editingSpecial) {
      await api.updateSpecial({
        ...editingSpecial,
        title: title.trim(),
        description,
        price: priceNum,
        activeDate: activeDate.trim(),
        imagePath,
      });
    } else {
      await api.addSpecial({
        title: title.trim(),
        description,
        price: priceNum,
        activeDate: activeDate.trim(),
        imagePath,
        active: true,
      });
    }
    dialogOpen = false;
    await load();
  }
</script>

<div class="title-row">
  <Heading text="Daily Specials" size={26} />
  <div class="spacer" />
  <Button label="Add special" icon="&#43;" gradient="var(--grad-brand)" width={160} onClick={openAdd} />
</div>
<span class="subtitle">Specials appear on the home screen of every customer table.</span>

{#if specials.length === 0}
  <span class="empty">No specials yet.</span>
{:else}
  {#each specials as sp (sp.id)}
    <GlowCard padding={16}>
      <div class="row" use:hoverGrow={{ color: "var(--pink)" }}>
        {#await imgSrc(sp.imagePath) then src}
          {#if src}
            <img class="img" src={src} alt={sp.title} />
          {:else}
            <div class="img placeholder">🍽</div>
          {/if}
        {/await}
        <div class="info">
          <span class="title">{sp.title}</span>
          <span class="desc">{sp.description || "—"}</span>
          <span class="meta">{money(sym, sp.price)} · {sp.activeDate || "every day"}</span>
        </div>
        <label class="switch">
          <input
            type="checkbox"
            checked={sp.active}
            on:click|preventDefault={() => toggleActive(sp, !sp.active)}
          />
          <span class="slider" />
        </label>
        <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.18 }} title="Edit" on:click={() => openEdit(sp)}>✏️</button>
        <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.18 }} title="Delete" on:click={() => deleteSpecial(sp)}>🗑</button>
      </div>
    </GlowCard>
  {/each}
{/if}

{#if dialogOpen}
  <div class="modal-backdrop" on:click|self={closeDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text={editingSpecial ? "Edit special" : "New special"} size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="sp-title">Title</label>
          <input id="sp-title" type="text" bind:value={title} autofocus />
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="sp-desc">Description</label>
          <textarea id="sp-desc" rows="3" bind:value={description}></textarea>
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="sp-price">Price</label>
          <input id="sp-price" type="text" bind:value={price} />
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="sp-date">Active date (YYYY-MM-DD, blank = every day)</label>
          <input id="sp-date" type="text" bind:value={activeDate} />
        </div>
        <div class="image-row">
          <Button label="Choose image" icon="&#128247;" gradient="var(--grad-brand)" width={160} onClick={pickImage} />
          {#if imagePreview}
            <img class="preview" src={imagePreview} alt="" />
          {:else}
            <div class="preview placeholder">🍽</div>
          {/if}
        </div>
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeDialog} />
          <Button label="Save" gradient="var(--grad-brand)" width={100} onClick={save} />
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
  .subtitle {
    color: var(--text-muted);
    font-size: 12px;
  }
  .empty {
    color: var(--text-muted);
    font-style: italic;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .img {
    width: 60px;
    height: 60px;
    border-radius: 10px;
    object-fit: cover;
    flex-shrink: 0;
  }
  .img.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-2);
    font-size: 22px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .title {
    font-weight: 700;
    color: var(--text);
    font-size: 14px;
  }
  .desc {
    color: var(--text-muted);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    color: var(--brand);
    font-size: 12px;
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
  .field input,
  .field textarea {
    border-radius: 10px;
    border: 1px solid #e0d4fa;
    padding: 8px 12px;
    font-size: 14px;
    font-family: var(--font-body);
    color: var(--text);
  }
  .field input:focus,
  .field textarea:focus {
    outline: 2px solid var(--brand);
  }
  .image-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 10px;
  }
  .preview {
    width: 60px;
    height: 60px;
    border-radius: 10px;
    object-fit: cover;
  }
  .preview.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-2);
    font-size: 22px;
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
    width: min(460px, 92vw);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 10px;
  }
</style>
