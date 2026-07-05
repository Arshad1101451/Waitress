<!-- Menu section — port of AdminApp._section_menu() and its dialogs. -->
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

  let categories = [];
  let itemsByCategory = {}; // categoryId -> MenuItem[]
  let imgCache = {}; // path -> assetUrl result

  onMount(load);

  async function load() {
    categories = await api.getCategories();
    const map = {};
    for (const cat of categories) {
      map[cat.id] = await api.getMenuItems(cat.id, false);
    }
    itemsByCategory = map;
  }

  async function imgSrc(path) {
    if (!path) return "";
    if (imgCache[path]) return imgCache[path];
    const url = await assetUrl(path);
    imgCache[path] = url;
    return url;
  }

  // ---------------------------------------------------------------- category dialog
  let catDialogOpen = false;
  let editingCat = null;
  let catName = "";
  let catStarter = false;
  let catError = "";

  function openAddCategory() {
    editingCat = null;
    catName = "";
    catStarter = false;
    catError = "";
    catDialogOpen = true;
  }

  function openEditCategory(cat) {
    editingCat = cat;
    catName = cat.name;
    catStarter = !!cat.isStarterGroup;
    catError = "";
    catDialogOpen = true;
  }

  function closeCatDialog() {
    catDialogOpen = false;
  }

  async function saveCategory() {
    if (!catName.trim()) {
      showToast("Name is required.", "var(--danger)");
      return;
    }
    if (editingCat) {
      await api.updateCategory({ ...editingCat, name: catName.trim(), isStarterGroup: catStarter });
    } else {
      await api.addCategory({
        name: catName.trim(),
        isStarterGroup: catStarter,
        sortOrder: categories.length,
      });
    }
    catDialogOpen = false;
    await load();
  }

  function deleteCategory(cat) {
    confirmAction({
      title: "Delete category?",
      body: `Delete "${cat.name}" and all its items?`,
      yesLabel: "Delete",
      danger: true,
      onYes: async () => {
        await api.deleteCategory(cat.id);
        showToast(`Deleted category ${cat.name}`);
        await load();
      },
    });
  }

  // ---------------------------------------------------------------- item availability
  function toggleAvailability(item, newValue) {
    const want = newValue ? "Show" : "Hide";
    confirmAction({
      title: `${want} item?`,
      body: `${want} "${item.name}" on customer menus?`,
      onYes: async () => {
        await api.setItemAvailability(item.id, newValue);
        await load();
      },
    });
    // Do NOT mutate local state here — the checkbox visually reverts
    // automatically because `item.available` (bound value) is unchanged
    // until load() re-fetches on confirm.
  }

  // ---------------------------------------------------------------- item dialog
  let itemDialogOpen = false;
  let editingItem = null;
  let editingCategoryId = null;
  let itemName = "";
  let itemDesc = "";
  let itemPrice = "0.00";
  let itemImagePath = "";
  let itemImagePreview = "";

  function openAddItem(categoryId) {
    editingItem = null;
    editingCategoryId = categoryId;
    itemName = "";
    itemDesc = "";
    itemPrice = "0.00";
    itemImagePath = "";
    itemImagePreview = "";
    itemDialogOpen = true;
  }

  async function openEditItem(item) {
    editingItem = item;
    editingCategoryId = item.categoryId;
    itemName = item.name;
    itemDesc = item.description || "";
    itemPrice = String(item.price ?? "0.00");
    itemImagePath = item.imagePath || "";
    itemImagePreview = itemImagePath ? await imgSrc(itemImagePath) : "";
    itemDialogOpen = true;
  }

  function closeItemDialog() {
    itemDialogOpen = false;
  }

  async function pickItemImage() {
    const path = await api.pickImageFile();
    if (!path) return;
    const newPath = await api.copyMenuImage(path);
    itemImagePath = newPath;
    itemImagePreview = await imgSrc(newPath);
  }

  async function saveItem() {
    if (!itemName.trim()) {
      showToast("Name is required.", "var(--danger)");
      return;
    }
    const priceNum = Number(itemPrice);
    if (Number.isNaN(priceNum)) {
      showToast("Price must be a number.", "var(--danger)");
      return;
    }
    if (editingItem) {
      await api.updateMenuItem({
        ...editingItem,
        name: itemName.trim(),
        description: itemDesc,
        price: priceNum,
        imagePath: itemImagePath,
        categoryId: editingCategoryId,
      });
    } else {
      await api.addMenuItem({
        categoryId: editingCategoryId,
        name: itemName.trim(),
        description: itemDesc,
        price: priceNum,
        imagePath: itemImagePath,
        available: true,
        sortOrder: (itemsByCategory[editingCategoryId] || []).length,
      });
    }
    itemDialogOpen = false;
    await load();
  }

  function deleteItem(item) {
    confirmAction({
      title: "Delete item?",
      body: `Permanently delete "${item.name}"?`,
      yesLabel: "Delete",
      danger: true,
      onYes: async () => {
        await api.deleteMenuItem(item.id);
        showToast(`Deleted ${item.name}`);
        await load();
      },
    });
  }
</script>

<div class="title-row">
  <Heading text="Menu" size={26} />
  <div class="spacer" />
  <Button label="Add category" icon="&#43;" gradient="var(--grad-brand)" width={170} onClick={openAddCategory} />
</div>

{#if categories.length === 0}
  <span class="empty">No categories yet — add one to get started.</span>
{/if}

{#each categories as cat (cat.id)}
  <GlowCard padding={16}>
    <div class="cat-header" use:hoverGrow={{ color: "var(--pink)" }}>
      <span class="cat-name">{cat.name}{cat.isStarterGroup ? " · starter group" : ""}</span>
      <div class="spacer" />
      <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.18 }} title="Add item" on:click={() => openAddItem(cat.id)}>➕</button>
      <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.18 }} title="Edit category" on:click={() => openEditCategory(cat)}>✏️</button>
      <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.18 }} title="Delete category" on:click={() => deleteCategory(cat)}>🗑</button>
    </div>
    <div class="divider" />
    {#if (itemsByCategory[cat.id] || []).length === 0}
      <span class="empty small">No items in this category.</span>
    {:else}
      {#each itemsByCategory[cat.id] as item (item.id)}
        <div class="item-row" use:hoverGrow={{ color: "var(--pink)" }}>
          {#await imgSrc(item.imagePath) then src}
            {#if src}
              <img class="item-img" src={src} alt={item.name} />
            {:else}
              <div class="item-img placeholder">🍽</div>
            {/if}
          {/await}
          <div class="item-info">
            <span class="item-name">{item.name}</span>
            <span class="item-desc">{item.description || "—"}</span>
          </div>
          <span class="item-price">{money(sym, item.price)}</span>
          <label class="switch">
            <input
              type="checkbox"
              checked={item.available}
              on:click|preventDefault={() => toggleAvailability(item, !item.available)}
            />
            <span class="slider" />
          </label>
          <button class="icon-btn" use:hoverGrow={{ color: "var(--brand)", scale: 1.18 }} title="Edit" on:click={() => openEditItem(item)}>✏️</button>
          <button class="icon-btn danger" use:hoverGrow={{ color: "var(--danger)", scale: 1.18 }} title="Delete" on:click={() => deleteItem(item)}>🗑</button>
        </div>
      {/each}
    {/if}
  </GlowCard>
{/each}

{#if catDialogOpen}
  <div class="modal-backdrop" on:click|self={closeCatDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text={editingCat ? "Edit category" : "New category"} size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="cat-name">Category name</label>
          <input id="cat-name" type="text" bind:value={catName} autofocus />
        </div>
        <label class="checkbox-row">
          <input type="checkbox" bind:checked={catStarter} />
          Suggest in the waitress greeting (starter group)
        </label>
        {#if catError}
          <span class="error">{catError}</span>
        {/if}
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeCatDialog} />
          <Button label="Save" gradient="var(--grad-brand)" width={100} onClick={saveCategory} />
        </div>
      </GlowCard>
    </div>
  </div>
{/if}

{#if itemDialogOpen}
  <div class="modal-backdrop" on:click|self={closeItemDialog}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text={editingItem ? "Edit item" : "New item"} size={18} />
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="item-name">Name</label>
          <input id="item-name" type="text" bind:value={itemName} autofocus />
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="item-desc">Description</label>
          <textarea id="item-desc" rows="3" bind:value={itemDesc}></textarea>
        </div>
        <div class="field" use:hoverGrow={{ color: "var(--pink)" }}>
          <label for="item-price">Price</label>
          <input id="item-price" type="text" bind:value={itemPrice} />
        </div>
        <div class="image-row">
          <Button label="Choose image" icon="&#128247;" gradient="var(--grad-brand)" width={160} onClick={pickItemImage} />
          {#if itemImagePreview}
            <img class="preview" src={itemImagePreview} alt="" />
          {:else}
            <div class="preview placeholder">🍽</div>
          {/if}
        </div>
        <div class="modal-actions">
          <Button label="Cancel" bgcolor="var(--surface-2)" textColor="var(--text)" width={100} onClick={closeItemDialog} />
          <Button label="Save" gradient="var(--grad-brand)" width={100} onClick={saveItem} />
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
  .empty.small {
    font-size: 12px;
  }
  .cat-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .cat-name {
    font-size: 17px;
    font-weight: 700;
    color: var(--text);
  }
  .divider {
    height: 1px;
    background: var(--surface-2);
    margin: 8px 0;
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
  .item-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
  }
  .item-img {
    width: 48px;
    height: 48px;
    border-radius: 10px;
    object-fit: cover;
    flex-shrink: 0;
  }
  .item-img.placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-2);
    font-size: 20px;
  }
  .item-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }
  .item-name {
    font-weight: 600;
    color: var(--text);
    font-size: 14px;
  }
  .item-desc {
    color: var(--text-muted);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .item-price {
    color: var(--brand);
    font-weight: 700;
    width: 80px;
    font-size: 14px;
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
  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text);
    margin-bottom: 10px;
  }
  .error {
    color: var(--danger);
    font-size: 12px;
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
