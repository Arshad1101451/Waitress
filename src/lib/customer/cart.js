// Cart math — port of waitress/shared/cart.py's Cart/CartLine. Kept as plain
// functions over a Map<itemId, line> so CustomerApp.svelte can drive it with
// ordinary reactive assignment (`cart = addLine(cart, item, qty)`).

function round2(n) {
  return Math.round(n * 100) / 100;
}

export function emptyCart() {
  return new Map();
}

export function addLine(cart, item, quantity = 1, notes = "") {
  const next = new Map(cart);
  const existing = next.get(item.id);
  if (existing) {
    next.set(item.id, {
      ...existing,
      quantity: existing.quantity + quantity,
      notes: notes || existing.notes,
    });
  } else {
    next.set(item.id, { item, quantity, notes });
  }
  return next;
}

export function setQuantity(cart, itemId, quantity) {
  const next = new Map(cart);
  if (quantity <= 0) {
    next.delete(itemId);
  } else {
    const line = next.get(itemId);
    if (line) next.set(itemId, { ...line, quantity });
  }
  return next;
}

export function clearCart() {
  return new Map();
}

export function lineTotal(line) {
  return round2(line.item.price * line.quantity);
}

export function subtotal(cart) {
  let sum = 0;
  for (const line of cart.values()) sum += lineTotal(line);
  return round2(sum);
}

export function taxFor(cart, rate) {
  return round2(subtotal(cart) * (rate || 0));
}

export function totalFor(cart, rate) {
  return round2(subtotal(cart) + taxFor(cart, rate));
}

export function toOrder(cart, { tableId, deviceId, taxRate, paymentMethod }) {
  const sub = subtotal(cart);
  const tax = taxFor(cart, taxRate);
  const items = Array.from(cart.values()).map((line) => ({
    id: null,
    orderId: null,
    menuItemId: line.item.id,
    name: line.item.name,
    unitPrice: line.item.price,
    quantity: line.quantity,
    notes: line.notes || "",
  }));
  return {
    id: null,
    tableId: tableId || "",
    deviceId: deviceId || "",
    status: "NEW",
    paymentStatus: paymentMethod === "cash" || paymentMethod === "card" ? "PENDING" : "PAID",
    paymentMethod: paymentMethod || "",
    subtotal: sub,
    tax,
    total: round2(sub + tax),
    kitchenMessage: "",
    createdAt: "",
    updatedAt: "",
    items,
  };
}
