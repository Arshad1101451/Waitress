import { writable } from "svelte/store";

// ------------------------------------------------------------------ toasts
// Replacement for Flet's page.show_dialog(SnackBar(...)) shim used
// throughout the Python app (_snack()).
export const toasts = writable([]);
let toastSeq = 0;

export function showToast(message, color = "var(--success)") {
  const id = ++toastSeq;
  toasts.update((list) => [...list, { id, message, color }]);
  setTimeout(() => {
    toasts.update((list) => list.filter((t) => t.id !== id));
  }, 3200);
}

// -------------------------------------------------------- confirm dialogs
// Replacement for the Python app's generic `_confirm(...)` AlertDialog
// helper — every destructive/committing action routes through this so
// there's exactly one modal implementation.
export const confirmState = writable(null); // { title, body, yesLabel, danger, onYes }

export function confirmAction({ title, body, yesLabel = "Confirm", danger = false, onYes }) {
  confirmState.set({ title, body, yesLabel, danger, onYes });
}

export function resolveConfirm(accepted) {
  confirmState.update((current) => {
    if (current && accepted) current.onYes?.();
    return null;
  });
}

// ------------------------------------------------------------- app-wide data
export const restaurant = writable(null);
export const deviceConfig = writable(null);
export const currentAdmin = writable(null);
