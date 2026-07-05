// Small formatting helpers used across every module (mirrors theme.money()
// and the ad-hoc string building in the Python app).

export function money(symbol, amount) {
  const n = Number(amount ?? 0);
  return `${symbol}${n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function titleCase(s) {
  if (!s) return "";
  return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

export function agoText(iso) {
  if (!iso) return "";
  const then = new Date(iso.replace(" ", "T"));
  if (Number.isNaN(then.getTime())) return "";
  const mins = Math.floor((Date.now() - then.getTime()) / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins} min ago`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return `${h}h ${m}m ago`;
}
