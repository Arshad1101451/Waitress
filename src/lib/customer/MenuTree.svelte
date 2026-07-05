<!-- Category tree, all expanded by default. Port of
     CustomerApp._menu_tree / _tree_item_row. -->
<script>
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import { money } from "../format.js";
  import { hoverGrow } from "../actions.js";

  export let categories = [];
  export let itemsByCategory = {}; // { [categoryId]: MenuItem[] }
  export let selectedItemId = null;
  export let currencySymbol = "$";
  export let onSelect = (id) => {};
</script>

<GlowCard padding={12}>
  <div class="tree">
    <Heading text="Menu" size={22} />
    <div class="categories">
      {#each categories as cat (cat.id)}
        <div class="category">
          <span class="cat-title" use:hoverGrow={{ color: "var(--pink)" }}>{cat.name}{cat.isStarterGroup ? " · starter" : ""}</span>
          <div class="items">
            {#each itemsByCategory[cat.id] || [] as item (item.id)}
              <button
                class="item-row"
                class:selected={selectedItemId === item.id}
                use:hoverGrow={{ color: "var(--pink)", scale: 1.03 }}
                on:click={() => onSelect(item.id)}
              >
                <span class="dot">{selectedItemId === item.id ? "★" : "•"}</span>
                <span class="name">{item.name}</span>
                <span class="price">{money(currencySymbol, item.price)}</span>
              </button>
            {:else}
              <span class="empty">No items</span>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
</GlowCard>

<style>
  .tree {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .categories {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    max-height: 70vh;
  }
  .cat-title {
    font-style: italic;
    font-weight: 700;
    color: var(--brand);
    font-size: 16px;
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 6px;
  }
  .item-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    border: none;
    background: var(--surface-2);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-body);
  }
  .item-row.selected {
    background: var(--grad-brand);
    color: white;
  }
  .item-row .dot {
    color: var(--pink);
  }
  .item-row.selected .dot {
    color: var(--yellow);
  }
  .item-row .name {
    flex: 1;
    font-weight: 600;
    color: var(--brand);
  }
  .item-row.selected .name {
    color: white;
  }
  .item-row .price {
    color: var(--pink);
    font-weight: 600;
  }
  .item-row.selected .price {
    color: white;
  }
  .empty {
    color: var(--text-muted);
    font-style: italic;
    font-size: 13px;
    padding: 4px 10px;
  }
</style>
