// Thin wrapper over every Tauri command in src-tauri/src/commands.rs.
// Every function here is a 1:1 mirror of a `#[tauri::command]` — keeping the
// naming identical makes it easy to cross-reference against the Rust side.
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

export const api = {
  // device config
  getDeviceConfig: () => invoke("get_device_config"),
  updateDeviceConfig: (patch) => invoke("update_device_config", { patch }),

  // restaurant / branding
  getRestaurant: () => invoke("get_restaurant"),
  saveRestaurant: (settings) => invoke("save_restaurant", { settings }),

  // categories
  getCategories: () => invoke("get_categories"),
  addCategory: (cat) => invoke("add_category", { cat }),
  updateCategory: (cat) => invoke("update_category", { cat }),
  deleteCategory: (id) => invoke("delete_category", { id }),

  // menu items
  getMenuItems: (categoryId = null, availableOnly = true) =>
    invoke("get_menu_items", { categoryId, availableOnly }),
  getMenuItem: (id) => invoke("get_menu_item", { id }),
  addMenuItem: (item) => invoke("add_menu_item", { item }),
  updateMenuItem: (item) => invoke("update_menu_item", { item }),
  setItemAvailability: (id, available) => invoke("set_item_availability", { id, available }),
  deleteMenuItem: (id) => invoke("delete_menu_item", { id }),

  // specials
  getActiveSpecials: (onDate = null) => invoke("get_active_specials", { onDate }),
  getAllSpecials: () => invoke("get_all_specials"),
  addSpecial: (sp) => invoke("add_special", { sp }),
  updateSpecial: (sp) => invoke("update_special", { sp }),
  deleteSpecial: (id) => invoke("delete_special", { id }),

  // activation / auth
  isActivated: () => invoke("is_activated"),
  deviceId: () => invoke("device_id"),
  activate: (code) => invoke("activate", { code }),
  login: (username, password) => invoke("login", { username, password }),
  logout: () => invoke("logout"),
  currentAdmin: () => invoke("current_admin"),
  getAdmins: () => invoke("get_admins"),
  createAdmin: (username, password, role) => invoke("create_admin", { username, password, role }),
  updateAdmin: (id, username, role) => invoke("update_admin", { id, username, role }),
  setAdminActive: (id, active) => invoke("set_admin_active", { id, active }),
  deleteAdmin: (id) => invoke("delete_admin", { id }),
  resetAdminPassword: (id, newPassword) => invoke("reset_admin_password", { id, newPassword }),

  // notifications
  getNotifications: (unreadOnly = false, limit = 100) => invoke("get_notifications", { unreadOnly, limit }),
  unreadNotificationCount: () => invoke("unread_notification_count"),
  markNotificationRead: (id) => invoke("mark_notification_read", { id }),
  markAllNotificationsRead: () => invoke("mark_all_notifications_read"),

  // orders
  createOrder: (order) => invoke("create_order", { order }),
  getOrder: (id) => invoke("get_order", { id }),
  getOrders: (status = null, tableId = null) => invoke("get_orders", { status, tableId }),
  updateOrderStatus: (id, status) => invoke("update_order_status", { id, status }),
  setPayment: (id, method, status) => invoke("set_payment", { id, method, status }),
  setKitchenMessage: (id, message) => invoke("set_kitchen_message", { id, message }),

  // reporting
  dashboardSummary: () => invoke("dashboard_summary"),
  salesByDay: (days = 7) => invoke("sales_by_day", { days }),
  salesByTable: () => invoke("sales_by_table"),
  salesByPeriod: (period, buckets = 12) => invoke("sales_by_period", { period, buckets }),

  // branding / files
  pickImageFile: () => invoke("pick_image_file"),
  copyLogo: (srcPath) => invoke("copy_logo", { srcPath }),
  copyBackground: (srcPath) => invoke("copy_background", { srcPath }),
  copyMenuImage: (srcPath) => invoke("copy_menu_image", { srcPath }),
  listGalleryImages: () => invoke("list_gallery_images"),
  listBackgroundImages: () => invoke("list_background_images"),

  // printing
  printOrderTicket: (orderId) => invoke("print_order_ticket", { orderId }),

  // seed
  ensureSeeded: () => invoke("ensure_seeded"),
};

/** Converts a local filesystem path into a URL the webview can load as an
 * <img src>. Tauri serves arbitrary local files through the special
 * `asset://` protocol via `convertFileSrc`. Kept `async` even though the
 * underlying call is now synchronous, so every call site (written as
 * `{#await assetUrl(path) then src}`) keeps working unchanged. */
export async function assetUrl(path) {
  if (!path) return "";
  return convertFileSrc(path);
}
