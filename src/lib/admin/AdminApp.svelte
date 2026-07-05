<!-- Admin module root — port of waitress/admin/app.py::AdminApp.
     Screens: Activation -> Login -> Shell. Session persistence across role
     switches works "for free": the Rust backend keeps the logged-in admin
     in memory (admin_session), so re-entering this module just re-asks
     currentAdmin() and skips straight to the Shell if one exists. -->
<script>
  import { onMount } from "svelte";
  import { api } from "../api.js";
  import { restaurant as restaurantStore, currentAdmin as currentAdminStore } from "../stores.js";
  import Background from "../components/Background.svelte";
  import Activation from "./Activation.svelte";
  import Login from "./Login.svelte";
  import Shell from "./Shell.svelte";

  export let onExit = () => {};

  // "activation" | "login" | "shell" | null (loading)
  let screen = null;
  let restaurant = null;
  let admin = null;

  onMount(load);

  async function load() {
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);

    const activated = await api.isActivated();
    if (!activated) {
      screen = "activation";
      return;
    }
    admin = await api.currentAdmin();
    currentAdminStore.set(admin);
    screen = admin ? "shell" : "login";
  }

  function onActivated() {
    screen = "login";
  }

  async function onLoggedIn() {
    admin = await api.currentAdmin();
    currentAdminStore.set(admin);
    screen = "shell";
  }

  function onSignedOut() {
    admin = null;
    currentAdminStore.set(null);
    screen = "login";
  }

  async function refreshRestaurant() {
    restaurant = await api.getRestaurant();
    restaurantStore.set(restaurant);
  }
</script>

{#if screen === "activation"}
  <Background restaurant={restaurant}>
    <Activation {restaurant} onActivated={onActivated} />
  </Background>
{:else if screen === "login"}
  <Background restaurant={restaurant}>
    <Login {restaurant} onLoggedIn={onLoggedIn} />
  </Background>
{:else if screen === "shell"}
  <Shell
    {restaurant}
    {admin}
    onSignOut={onSignedOut}
    onExit={onExit}
    onRestaurantChanged={refreshRestaurant}
  />
{/if}
