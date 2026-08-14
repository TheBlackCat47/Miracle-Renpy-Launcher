<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  type SystemStatus = {
    app_name: string;
    version: string;
    platform: string;
    data_directory: string;
  };

  let status: SystemStatus | null = null;
  let error = '';

  async function loadStatus() {
    error = '';
    try {
      status = await invoke<SystemStatus>('get_system_status');
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  function addGame() {
    error = 'La détection des jeux sera disponible dans le prochain incrément.';
  }
</script>

<svelte:head>
  <title>MRL — Bibliothèque</title>
</svelte:head>

<main class="shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">M</div>
      <div>
        <strong>Miracle</strong>
        <span>Ren'Py Launcher</span>
      </div>
    </div>

    <nav aria-label="Navigation principale">
      <a class="nav-item active" href="/" aria-current="page">Bibliothèque</a>
      <a class="nav-item" href="/sync" on:click|preventDefault={() => (error = 'La synchronisation sera activée avec le moteur Cloud.')}>Synchronisation</a>
      <a class="nav-item" href="/settings" on:click|preventDefault={() => (error = 'Les paramètres seront ajoutés avec la configuration locale.')}>Paramètres</a>
    </nav>

    <div class="sidebar-footer">
      <button class="status-button" on:click={loadStatus}>Vérifier le système</button>
      {#if status}
        <small>{status.platform} · MRL {status.version}</small>
      {/if}
    </div>
  </aside>

  <section class="content">
    <header class="topbar">
      <div>
        <p class="eyebrow">Bibliothèque locale</p>
        <h1>Vos jeux</h1>
      </div>
      <button class="primary" on:click={addGame}>+ Ajouter un jeu</button>
    </header>

    {#if error}
      <div class="notice" role="status">{error}</div>
    {/if}

    <div class="empty-state">
      <div class="empty-icon">✦</div>
      <h2>Votre bibliothèque est vide</h2>
      <p>Ajoutez un jeu Ren'Py pour commencer à gérer vos sauvegardes localement.</p>
      <button class="secondary" on:click={addGame}>Ajouter mon premier jeu</button>
    </div>

    <footer class="content-footer">
      <span>Local First</span>
      <span>·</span>
      <span>Prêt pour le prochain jeu</span>
    </footer>
  </section>
</main>
