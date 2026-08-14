<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  type SystemStatus = {
    app_name: string;
    version: string;
    platform: string;
    data_directory: string;
  };

  type GameInspection = {
    path: string;
    folder_name: string;
    is_renpy: boolean;
    confidence: 'high' | 'medium' | 'none';
    executable: string | null;
    identity_hint: string;
    save_directories: string[];
    markers: string[];
  };

  let status: SystemStatus | null = null;
  let error = '';
  let showAddPanel = false;
  let gamePath = '';
  let inspection: GameInspection | null = null;

  async function loadStatus() {
    error = '';
    try {
      status = await invoke<SystemStatus>('get_system_status');
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  function addGame() {
    showAddPanel = true;
    error = '';
    inspection = null;
  }

  async function inspectGame() {
    error = '';
    inspection = null;
    try {
      inspection = await invoke<GameInspection>('inspect_game_directory', { path: gamePath });
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
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

    {#if showAddPanel}
      <div class="add-panel">
        <div>
          <p class="eyebrow">Nouveau jeu</p>
          <h2>Inspecter un dossier Ren'Py</h2>
          <p>Indiquez le chemin du dossier qui contient le jeu et son dossier <code>game/</code>.</p>
        </div>
        <form on:submit|preventDefault={inspectGame}>
          <label for="game-path">Chemin du jeu</label>
          <div class="path-row">
            <input id="game-path" bind:value={gamePath} placeholder="C:\\Jeux\\MonJeu" autocomplete="off" />
            <button class="primary" type="submit">Analyser</button>
          </div>
        </form>
        {#if inspection}
          <div class:valid={inspection.is_renpy} class:invalid={!inspection.is_renpy} class="inspection-result">
            <strong>{inspection.is_renpy ? 'Jeu Ren\'Py détecté' : 'Structure Ren\'Py non confirmée'}</strong>
            <span>Confiance : {inspection.confidence}</span>
            <span>Marqueurs : {inspection.markers.join(', ') || 'aucun'}</span>
            <span>Sauvegardes : {inspection.save_directories.length || 'aucune détectée'}</span>
          </div>
        {/if}
        <button class="text-button" on:click={() => (showAddPanel = false)}>Retour à la bibliothèque</button>
      </div>
    {:else}
      <div class="empty-state">
      <div class="empty-icon">✦</div>
      <h2>Votre bibliothèque est vide</h2>
      <p>Ajoutez un jeu Ren'Py pour commencer à gérer vos sauvegardes localement.</p>
      <button class="secondary" on:click={addGame}>Ajouter mon premier jeu</button>
      </div>
    {/if}

    <footer class="content-footer">
      <span>Local First</span>
      <span>·</span>
      <span>Prêt pour le prochain jeu</span>
    </footer>
  </section>
</main>
