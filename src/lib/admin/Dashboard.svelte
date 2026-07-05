<!-- Dashboard section — port of AdminApp._section_dashboard(). -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { showToast } from "../stores.js";
  import { money } from "../format.js";
  import { hoverGrow } from "../actions.js";
  import GlowCard from "../components/GlowCard.svelte";
  import Heading from "../components/Heading.svelte";
  import ShinyText from "../components/ShinyText.svelte";
  import Button from "../components/Button.svelte";

  export let restaurant;
  export let onRestaurantChanged = () => {};

  let summary = null;
  let days = [];
  let byTable = [];
  let loading = true;

  // Report modal state.
  let reportOpen = false;
  let reportPeriod = "week";
  let reportRows = [];

  $: sym = restaurant?.currencySymbol ?? "$";

  onMount(load);

  async function load() {
    loading = true;
    try {
      summary = await api.dashboardSummary();
      days = await api.salesByDay(7);
      byTable = await api.salesByTable();
    } finally {
      loading = false;
    }
  }

  // Rust HashMap<String,f64> keys come through verbatim as "orders" /
  // "revenue" / "avg_order" (snake_case, NOT auto-camelCased since these
  // are map keys, not struct fields) — read defensively either way.
  function block(obj) {
    if (!obj) return { orders: 0, revenue: 0, avgOrder: 0 };
    return {
      orders: obj.orders ?? 0,
      revenue: obj.revenue ?? 0,
      avgOrder: obj.avg_order ?? obj.avgOrder ?? 0,
    };
  }

  function rowLabel(r) {
    return r?.label ?? r?.name ?? "";
  }
  function rowOrders(r) {
    return r?.orders ?? 0;
  }
  function rowRevenue(r) {
    return r?.revenue ?? 0;
  }

  async function refresh() {
    await load();
    restaurant = await api.getRestaurant();
    onRestaurantChanged();
    showToast("Dashboard refreshed.", "var(--brand)");
  }

  function maxRevenue(rows) {
    return Math.max(1, ...rows.map((r) => Number(rowRevenue(r)) || 0));
  }

  async function openReport(period) {
    reportPeriod = period;
    reportRows = await api.salesByPeriod(period, 12);
    reportOpen = true;
  }

  function closeReport() {
    reportOpen = false;
  }

  $: reportTotalOrders = reportRows.reduce((s, r) => s + (rowOrders(r) || 0), 0);
  $: reportTotalRevenue = reportRows.reduce((s, r) => s + (Number(rowRevenue(r)) || 0), 0);
</script>

<div class="title-row">
  <Heading text="Dashboard" size={26} />
  <div class="spacer" />
  <Button label="Refresh" icon="&#8635;" gradient="var(--grad-brand)" width={150} onClick={refresh} />
</div>

{#if !loading}
  <div class="stats">
    <GlowCard padding={18} hover>
      <div class="stat">
        <span class="stat-label">Today</span>
        <ShinyText text={money(sym, block(summary?.today).revenue)} size={26} color="var(--purple)" />
        <span class="stat-sub">{block(summary?.today).orders} orders · avg {money(sym, block(summary?.today).avgOrder)}</span>
      </div>
    </GlowCard>
    <GlowCard padding={18} hover>
      <div class="stat">
        <span class="stat-label">This week</span>
        <ShinyText text={money(sym, block(summary?.week).revenue)} size={26} color="var(--purple)" />
        <span class="stat-sub">{block(summary?.week).orders} orders · avg {money(sym, block(summary?.week).avgOrder)}</span>
      </div>
    </GlowCard>
    <GlowCard padding={18} hover>
      <div class="stat">
        <span class="stat-label">This month</span>
        <ShinyText text={money(sym, block(summary?.month).revenue)} size={26} color="var(--purple)" />
        <span class="stat-sub">{block(summary?.month).orders} orders · avg {money(sym, block(summary?.month).avgOrder)}</span>
      </div>
    </GlowCard>
    <GlowCard padding={18} hover>
      <div class="stat">
        <span class="stat-label">All time</span>
        <ShinyText text={money(sym, block(summary?.all).revenue)} size={26} color="var(--purple)" />
        <span class="stat-sub">{block(summary?.all).orders} orders · avg {money(sym, block(summary?.all).avgOrder)}</span>
      </div>
    </GlowCard>
  </div>

  <GlowCard padding={16}>
    <span class="card-title">Revenue — last 7 days</span>
    {#if days.length === 0}
      <span class="empty">No data yet.</span>
    {:else}
      <div class="bar-chart">
        {#each days as d}
          <div class="bar-col" use:hoverGrow={{ color: "var(--brand)", scale: 1.08 }}>
            <div class="bar" style="height: {(rowRevenue(d) / maxRevenue(days)) * 100}px;" />
            <span class="bar-caption">{money(sym, rowRevenue(d))}</span>
            <span class="bar-label">{rowLabel(d)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </GlowCard>

  <GlowCard padding={16}>
    <span class="card-title">Sales by table</span>
    {#if byTable.length === 0}
      <span class="empty">No data yet.</span>
    {:else}
      <div class="bar-chart">
        {#each byTable as t}
          <div class="bar-col" use:hoverGrow={{ color: "var(--brand)", scale: 1.08 }}>
            <div class="bar" style="height: {(rowRevenue(t) / maxRevenue(byTable)) * 100}px;" />
            <span class="bar-caption">{money(sym, rowRevenue(t))} · {rowOrders(t)} ord</span>
            <span class="bar-label">{rowLabel(t)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </GlowCard>

  <GlowCard padding={16} hover>
    <div class="reports-header" use:hoverGrow={{ color: "var(--brand)" }}>
      <span class="card-title">Detailed reports</span>
      <div class="spacer" />
      <Button label="Weekly report" icon="&#128202;" gradient="var(--grad-cool)" width={180}
        onClick={() => openReport("week")} />
      <Button label="Monthly report" icon="&#128202;" gradient="var(--grad-sunny)" width={190}
        onClick={() => openReport("month")} />
    </div>
    <span class="empty">Open a period-by-period breakdown as a table.</span>
  </GlowCard>
{/if}

{#if reportOpen}
  <div class="modal-backdrop" on:click|self={closeReport}>
    <div class="modal">
      <GlowCard padding={24}>
        <Heading text={reportPeriod === "week" ? "Weekly report" : "Monthly report"} size={20} />
        {#if reportRows.length === 0}
          <p class="empty" style="text-align:center;padding:24px;">No data yet.</p>
        {:else}
          <table class="report-table">
            <thead>
              <tr>
                <th>Period</th>
                <th class="num">Orders</th>
                <th class="num">Revenue</th>
              </tr>
            </thead>
            <tbody>
              {#each reportRows as r}
                <tr>
                  <td>{rowLabel(r)}</td>
                  <td class="num">{rowOrders(r)}</td>
                  <td class="num">{money(sym, rowRevenue(r))}</td>
                </tr>
              {/each}
            </tbody>
          </table>
          <div class="report-total">
            Total: {reportTotalOrders} orders · {money(sym, Math.round(reportTotalRevenue * 100) / 100)}
          </div>
        {/if}
        <div class="modal-actions">
          <Button label="Close" gradient="var(--grad-brand)" width={120} onClick={closeReport} />
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
  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 190px;
  }
  .stat-label {
    font-style: italic;
    font-size: 12px;
    color: var(--text-muted);
  }
  .stat-sub {
    font-style: italic;
    font-size: 11px;
    color: var(--text-muted);
  }
  .card-title {
    font-weight: 700;
    color: var(--text);
  }
  .empty {
    color: var(--text-muted);
    font-style: italic;
  }
  .bar-chart {
    display: flex;
    align-items: flex-end;
    gap: 14px;
    padding-top: 14px;
    min-height: 140px;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 74px;
  }
  .bar {
    width: 32px;
    min-height: 4px;
    background: var(--grad-brand);
    border-radius: 6px 6px 0 0;
  }
  .bar-caption {
    font-size: 13px;
    color: var(--text);
    font-weight: 600;
  }
  .bar-label {
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
  }
  .reports-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 4px;
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
    width: min(500px, 92vw);
  }
  .report-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 10px;
  }
  .report-table th {
    text-align: left;
    font-size: 12px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--surface-2);
    padding: 6px 8px;
  }
  .report-table td {
    padding: 6px 8px;
    font-size: 13px;
    color: var(--text);
    border-bottom: 1px solid var(--surface-2);
  }
  .report-table tbody tr {
    transition: background-color 120ms ease;
  }
  .report-table tbody tr:hover {
    background-color: var(--surface-2);
  }
  .num {
    text-align: right;
  }
  .report-total {
    margin-top: 10px;
    font-weight: 700;
    color: var(--purple);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
  }
</style>
