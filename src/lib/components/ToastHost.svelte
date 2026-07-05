<script>
  import { toasts } from "../stores.js";
</script>

<div class="toast-host">
  {#each $toasts as t (t.id)}
    <div class="toast" style="background: {t.color};">{t.message}</div>
  {/each}
</div>

<style>
  .toast-host {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 9999;
    pointer-events: none;
  }
  .toast {
    color: white;
    padding: 10px 20px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 500;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.2);
    animation: rise 180ms ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
