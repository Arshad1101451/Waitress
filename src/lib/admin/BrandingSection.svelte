<!-- Branding section — port of AdminApp._section_branding()/_gallery_pick().
     NOTE: the AI-gallery single-click-to-confirm-to-save flow was a
     recurring bug in the Python original (confirm opened but never
     committed) — the onYes handler below re-fetches the LATEST restaurant
     via api.getRestaurant() (never trusts possibly-stale local state),
     saves immediately, then re-reads the value back and pushes the new
     restaurant up via onRestaurantChanged so Background.svelte reflects it
     everywhere without needing a manual reload. -->
<script>
  import { onMount } from "svelte";
  import { api, assetUrl } from "../api.js";
  import { showToast, confirmAction, restaurant as restaurantStore } from "../stores.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import Button from "../components/Button.svelte";

  export let restaurant;
  export let onRestaurantChanged = () => {};

  // ---------------------------------------------------------------- draft/local state
  let name = "";
  let tagline = "";
  let address = "";
  let phone = "";
  let email = "";
  let currencySymbol = "";
  let currencyCode = "";
  let taxRate = "0";

  let stagedLogoPath = "";
  let logoPreview = "";

  let stagedBackground = "";
  let ownImages = [];
  let ownImagePreviews = {};

  let galleryImages = [];
  let galleryPreviews = {};

  onMount(load);

  function hydrateFromRestaurant(r) {
    name = r?.name ?? "";
    tagline = r?.tagline ?? "";
    address = r?.address ?? "";
    phone = r?.phone ?? "";
    email = r?.email ?? "";
    currencySymbol = r?.currencySymbol ?? "$";
    currencyCode = r?.currency ?? "USD";
    taxRate = String(r?.taxRate ?? "0");
    stagedLogoPath = r?.logoPath ?? "";
    stagedBackground = r?.activeBackground ?? "";
  }

  async function load() {
    hydrateFromRestaurant(restaurant);
    logoPreview = stagedLogoPath ? await assetUrl(stagedLogoPath) : "";

    ownImages = await api.listBackgroundImages();
    const ownMap = {};
    for (const p of ownImages) ownMap[p] = await assetUrl(p);
    ownImagePreviews = ownMap;

    galleryImages = await api.listGalleryImages();
    const galMap = {};
    for (const p of galleryImages) galMap[p] = await assetUrl(p);
    galleryPreviews = galMap;
  }

  function prettyName(path) {
    const base = path.split(/[\\/]/).pop() || path;
    const stem = base.replace(/\.[^.]+$/, "");
    const parts = stem.split("_");
    const words = parts.length > 2 ? parts.slice(2) : parts;
    return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
  }

  // ---------------------------------------------------------------- logo
  async function chooseLogo() {
    const path = await api.pickImageFile();
    if (!path) return;
    const newPath = await api.copyLogo(path);
    stagedLogoPath = newPath;
    logoPreview = await assetUrl(newPath);
  }

  // ---------------------------------------------------------------- own backgrounds
  function selectOwnBackground(path) {
    stagedBackground = path;
  }

  async function addBackground() {
    const path = await api.pickImageFile();
    if (!path) return;
    const newPath = await api.copyBackground(path);
    ownImages = [...ownImages, newPath];
    ownImagePreviews = { ...ownImagePreviews, [newPath]: await assetUrl(newPath) };
    stagedBackground = newPath;
  }

  // ---------------------------------------------------------------- save (identity+logo+background)
  async function save() {
    const taxNum = Number(taxRate);
    if (Number.isNaN(taxNum)) {
      showToast("Tax rate must be a number.", "var(--danger)");
      return;
    }
    const latest = await api.getRestaurant();
    const updated = {
      ...latest,
      name: name.trim() || latest.name,
      tagline,
      address: address.trim(),
      phone: phone.trim(),
      email: email.trim(),
      currencySymbol: currencySymbol || "$",
      currency: currencyCode || "USD",
      taxRate: taxNum,
      logoPath: stagedLogoPath,
      activeBackground: stagedBackground,
    };
    await api.saveRestaurant(updated);
    const saved = await api.getRestaurant();
    restaurant = saved;
    restaurantStore.set(saved);
    onRestaurantChanged();
    showToast("Branding saved — customer tables update on next sync.");
  }

  // ---------------------------------------------------------------- AI gallery (instant commit)
  function pickGalleryImage(path) {
    confirmAction({
      title: "Use this background?",
      body: "Set this dining-hall image as the background on every customer screen?",
      yesLabel: "Use it",
      onYes: async () => {
        // Re-fetch the LATEST settings — never trust possibly-stale local
        // state here, to avoid clobbering concurrent edits — then save
        // immediately, independent of the main Save button above.
        const latest = await api.getRestaurant();
        const updated = { ...latest, activeBackground: path };
        await api.saveRestaurant(updated);

        // Verify the write actually committed by re-reading it back.
        const confirmed = await api.getRestaurant();
        restaurant = confirmed;
        restaurantStore.set(confirmed);
        stagedBackground = confirmed.activeBackground;
        onRestaurantChanged();

        if (confirmed.activeBackground !== path) {
          showToast("Background update did not persist — please retry.", "var(--danger)");
          return;
        }
        showToast("Background updated for all customer screens.");
      },
    });
  }
</script>

<div class="title-row">
  <Heading text="Branding" size={26} />
  <div class="spacer" />
  <Button label="Save" icon="&#128190;" gradient="var(--grad-sunny)" width={140} onClick={save} />
</div>

<GlowCard padding={16}>
  <span class="card-title">Business identity (shown on every screen)</span>
  <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
    <label for="b-name">Business name</label>
    <input id="b-name" type="text" bind:value={name} />
  </div>
  <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
    <label for="b-tagline">Tagline</label>
    <input id="b-tagline" type="text" bind:value={tagline} />
  </div>
  <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
    <label for="b-address">Address</label>
    <input id="b-address" type="text" bind:value={address} />
  </div>
  <div class="field-row">
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="b-phone">Phone</label>
      <input id="b-phone" type="text" bind:value={phone} />
    </div>
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="b-email">Email</label>
      <input id="b-email" type="text" bind:value={email} />
    </div>
  </div>
  <div class="field-row">
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="b-symbol">Currency symbol</label>
      <input id="b-symbol" type="text" bind:value={currencySymbol} />
    </div>
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="b-currency">Currency code</label>
      <input id="b-currency" type="text" bind:value={currencyCode} />
    </div>
    <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
      <label for="b-tax">Tax rate (e.g. 0.08)</label>
      <input id="b-tax" type="text" bind:value={taxRate} />
    </div>
  </div>
  <div class="logo-row">
    <Button label="Choose logo…" icon="&#128247;" gradient="var(--grad-brand)" width={180} onClick={chooseLogo} />
    {#if logoPreview}
      <img class="logo-preview" src={logoPreview} alt="logo" />
    {:else}
      <div class="logo-preview placeholder">🍽</div>
    {/if}
  </div>
</GlowCard>

<GlowCard padding={16}>
  <span class="card-title">Background — luxury dining-hall gallery</span>
  <span class="card-subtitle">
    Tap a decorated dining hall to use it (you'll confirm before it goes live on every customer screen).
  </span>
  {#if galleryImages.length === 0}
    <span class="empty">No gallery images found.</span>
  {:else}
    <div class="gallery">
      {#each galleryImages as path (path)}
        <button
          class="tile"
          class:selected={path === restaurant?.activeBackground}
          on:click={() => pickGalleryImage(path)}
        >
          {#if galleryPreviews[path]}
            <img src={galleryPreviews[path]} alt={prettyName(path)} />
          {/if}
          <span class="tile-label">{prettyName(path)}</span>
        </button>
      {/each}
    </div>
  {/if}
</GlowCard>

<GlowCard padding={16}>
  <div class="row-header">
    <span class="card-title">Background — your own images</span>
    <div class="spacer" />
    <Button label="Add background…" icon="&#43;" gradient="var(--grad-brand)" width={200} onClick={addBackground} />
  </div>
  <span class="card-subtitle">Upload a photo from your computer, then tap it to select.</span>
  {#if ownImages.length === 0}
    <span class="empty">No backgrounds yet.</span>
  {:else}
    <div class="gallery">
      {#each ownImages as path (path)}
        <button
          class="tile own"
          class:selected={path === stagedBackground}
          on:click={() => selectOwnBackground(path)}
        >
          {#if ownImagePreviews[path]}
            <img src={ownImagePreviews[path]} alt="" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</GlowCard>

<style>
  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .spacer {
    flex: 1;
  }
  .card-title {
    font-weight: 700;
    color: var(--text);
    display: block;
    margin-bottom: 6px;
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
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
    flex: 1;
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
  .field-row {
    display: flex;
    gap: 12px;
  }
  .logo-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 6px;
  }
  .logo-preview {
    width: 64px;
    height: 64px;
    border-radius: 12px;
    object-fit: cover;
  }
  .logo-preview.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-2);
    font-size: 22px;
  }
  .row-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .gallery {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 8px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px;
    border-radius: 12px;
    border: 3px solid #eadcff;
    background: transparent;
    cursor: pointer;
    transition: transform 130ms ease-out, border-color 130ms ease-out, box-shadow 130ms ease-out;
  }
  .tile:hover {
    transform: scale(1.05);
    border-color: var(--light-purple);
    box-shadow: var(--shadow-soft);
  }
  .tile img {
    width: 150px;
    height: 92px;
    object-fit: cover;
    border-radius: 8px;
  }
  .tile.own img {
    width: 120px;
    height: 80px;
  }
  .tile-label {
    font-size: 11px;
    font-style: italic;
    color: var(--text-muted);
  }
  .tile.selected {
    border-color: var(--brand);
    background: var(--surface-2);
  }
  .tile.selected .tile-label {
    color: var(--text);
    font-weight: 700;
  }
</style>
