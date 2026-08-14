<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

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

  type GameRecord = {
    id: string;
    name: string;
    path: string;
    executable: string | null;
    confidence: string;
    save_count: number;
    identity_hint: string;
    added_at: string;
  };

  type RunningGame = {
    id: string;
    name: string;
    elapsed_seconds: number;
  };

  type SaveFile = {
    relative_path: string;
    size: number;
    modified_at: string;
    hash: string;
  };

  let status: SystemStatus | null = null;
  let error = '';
  let showAddPanel = false;
  let gamePath = '';
  let inspection: GameInspection | null = null;
  let games: GameRecord[] = [];
  let runningIds: string[] = [];
  let expandedGame = '';
  let saveFiles: Record<string, SaveFile[]> = {};
  let loadingSaves = '';

  onMount(() => {
    let timer: number | undefined;
    void (async () => {
      try {
        games = await invoke<GameRecord[]>('list_games');
        await refreshRunningGames();
        timer = window.setInterval(refreshRunningGames, 2000);
      } catch (reason) {
        error = reason instanceof Error ? reason.message : String(reason);
      }
    })();
    return () => {
      if (timer !== undefined) window.clearInterval(timer);
    };
  });

  async function refreshRunningGames() {
    try {
      const running = await invoke<RunningGame[]>('get_running_games');
      runningIds = running.map((game) => game.id);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

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

  async function registerGame() {
    if (!inspection?.is_renpy) return;
    error = '';
    try {
      const game = await invoke<GameRecord>('register_game', { path: inspection.path });
      games = [...games.filter((item) => item.id !== game.id), game];
      showAddPanel = false;
      inspection = null;
      gamePath = '';
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function launchGame(id: string) {
    error = '';
    try {
      await invoke<RunningGame>('launch_game', { id });
      await refreshRunningGames();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function toggleSaves(id: string) {
    if (expandedGame === id) {
      expandedGame = '';
      return;
    }
    expandedGame = id;
    loadingSaves = id;
    error = '';
    try {
      saveFiles[id] = await invoke<SaveFile[]>('scan_game_saves', { id });
      saveFiles = { ...saveFiles };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      loadingSaves = '';
    }
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024) return `${bytes} o`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
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
          {#if inspection.is_renpy}
            <button class="primary register-button" on:click={registerGame}>Ajouter à ma bibliothèque</button>
          {/if}
        {/if}
        <button class="text-button" on:click={() => (showAddPanel = false)}>Retour à la bibliothèque</button>
      </div>
    {:else if games.length > 0}
      <div class="game-grid">
        {#each games as game}
          <article class="game-card">
            <div class="game-cover">✦</div>
            <div class="game-card-body">
              <div class="game-card-heading">
                <h2>{game.name}</h2>
                <span class="confidence">{game.confidence}</span>
              </div>
              <p title={game.path}>{game.path}</p>
              <div class="game-meta">
                <span>{game.save_count} dossier{game.save_count === 1 ? '' : 's'} de sauvegarde</span>
                <span>{game.executable ? 'Exécutable détecté' : 'Lancement à configurer'}</span>
              </div>
              <button class:running={runningIds.includes(game.id)} class="launch-button" disabled={!game.executable || runningIds.includes(game.id)} on:click={() => launchGame(game.id)}>
                {runningIds.includes(game.id) ? 'Jeu en cours' : 'Lancer le jeu'}
              </button>
              <button class="save-button" on:click={() => toggleSaves(game.id)}>
                {loadingSaves === game.id ? 'Analyse en cours…' : expandedGame === game.id ? 'Masquer les sauvegardes' : 'Voir les sauvegardes'}
              </button>
              {#if expandedGame === game.id}
                <div class="save-list">
                  {#if saveFiles[game.id]?.length}
                    {#each saveFiles[game.id] as save}
                      <div class="save-row">
                        <span title={save.relative_path}>{save.relative_path}</span>
                        <small>{formatBytes(save.size)} · {save.hash.slice(0, 10)}</small>
                      </div>
                    {/each}
                  {:else}
                    <span class="no-saves">Aucune sauvegarde détectée.</span>
                  {/if}
                </div>
              {/if}
            </div>
          </article>
        {/each}
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
