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
    save_directory: string | null;
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

  type BackupResult = {
    backup_directory: string;
    file_count: number;
    created_at: string;
  };

  type BackupRecord = {
    directory: string;
    created_at: string;
    file_count: number;
  };

  type CloudStatus = {
    provider: string;
    configured: boolean;
    connected: boolean;
    account_email: string | null;
  };

  type DriveStatus = {
    email: string;
    display_name: string | null;
    storage_used: string | null;
  };

  type SyncResult = {
    uploaded_files: number;
    folder_name: string;
    manifest_file_id: string;
  };

  type PullResult = {
    downloaded_files: number;
    unchanged_files: number;
    backup_directory: string | null;
    conflicts: string[];
  };

  type ConflictResolutionResult = {
    relative_path: string;
    resolution: string;
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
  let backupMessages: Record<string, string> = {};
  let backups: Record<string, BackupRecord[]> = {};
  let loadingBackups = '';
  let showCloudPanel = false;
  let googleClientId = import.meta.env.ID_client ?? '';
  let cloudStatus: CloudStatus | null = null;
  let cloudMessage = '';
  let syncMessages: Record<string, string> = {};
  let syncConflicts: Record<string, string[]> = {};

  onMount(() => {
    let timer: number | undefined;
    void (async () => {
      try {
        games = await invoke<GameRecord[]>('list_games');
        cloudStatus = await invoke<CloudStatus>('get_cloud_status');
        const configuredClientId = import.meta.env.ID_client?.trim();
        if (configuredClientId && !cloudStatus.configured) {
          cloudStatus = await invoke<CloudStatus>('save_google_client_id', {
            clientId: configuredClientId,
          });
        }
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

  async function openCloudPanel() {
    showCloudPanel = true;
    showAddPanel = false;
    error = '';
    cloudMessage = '';
    try {
      cloudStatus = await invoke<CloudStatus>('get_cloud_status');
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function saveGoogleClientId() {
    error = '';
    cloudMessage = '';
    try {
      cloudStatus = await invoke<CloudStatus>('save_google_client_id', { clientId: googleClientId });
      cloudMessage = 'Identifiant client enregistré localement.';
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function connectGoogle() {
    error = '';
    cloudMessage = 'Ouverture de Google…';
    try {
      cloudStatus = await invoke<CloudStatus>('start_google_auth');
      cloudMessage = cloudStatus.connected ? 'Compte Google connecté.' : 'Connexion non finalisée.';
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
      cloudMessage = '';
    }
  }

  async function disconnectGoogle() {
    error = '';
    try {
      cloudStatus = await invoke<CloudStatus>('disconnect_google');
      cloudMessage = 'Compte Google déconnecté.';
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function verifyGoogleDrive() {
    error = '';
    cloudMessage = 'Vérification de Google Drive…';
    try {
      const drive = await invoke<DriveStatus>('verify_google_drive');
      cloudMessage = `Google Drive opérationnel pour ${drive.email}.`;
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
      cloudMessage = '';
    }
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

  async function backupSaves(id: string) {
    error = '';
    try {
      const result = await invoke<BackupResult>('backup_game_saves', { id });
      backupMessages = {
        ...backupMessages,
        [id]: `${result.file_count} fichier${result.file_count === 1 ? '' : 's'} sauvegardé${result.file_count === 1 ? '' : 's'}`,
      };
      await loadBackups(id);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function loadBackups(id: string) {
    loadingBackups = id;
    try {
      backups[id] = await invoke<BackupRecord[]>('list_backups', { id });
      backups = { ...backups };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      loadingBackups = '';
    }
  }

  async function restoreBackup(id: string, directory: string) {
    error = '';
    try {
      const result = await invoke<BackupResult>('restore_backup', { id, directory });
      backupMessages = {
        ...backupMessages,
        [id]: `Restauration effectuée · état précédent sauvegardé (${result.file_count} fichier${result.file_count === 1 ? '' : 's'})`,
      };
      await loadBackups(id);
      saveFiles[id] = await invoke<SaveFile[]>('scan_game_saves', { id });
      saveFiles = { ...saveFiles };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function syncGame(id: string) {
    error = '';
    syncMessages = { ...syncMessages, [id]: 'Synchronisation en cours…' };
    try {
      const result = await invoke<SyncResult>('sync_game_to_drive', { id });
      syncMessages = {
        ...syncMessages,
        [id]: `${result.uploaded_files} fichier${result.uploaded_files === 1 ? '' : 's'} envoyé${result.uploaded_files === 1 ? '' : 's'} dans Drive`,
      };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
      syncMessages = { ...syncMessages, [id]: '' };
    }
  }

  async function pullGame(id: string) {
    error = '';
    syncMessages = { ...syncMessages, [id]: 'Vérification de Drive en cours…' };
    try {
      const result = await invoke<PullResult>('sync_game_from_drive', { id });
      syncMessages = {
        ...syncMessages,
        [id]: result.conflicts.length
          ? `${result.conflicts.length} conflit${result.conflicts.length === 1 ? '' : 's'} détecté${result.conflicts.length === 1 ? '' : 's'} · fichiers protégés`
          : result.downloaded_files
            ? `${result.downloaded_files} fichier${result.downloaded_files === 1 ? '' : 's'} restauré${result.downloaded_files === 1 ? '' : 's'} · backup local créé`
            : `${result.unchanged_files} fichier${result.unchanged_files === 1 ? '' : 's'} déjà à jour`,
      };
      syncConflicts = { ...syncConflicts, [id]: result.conflicts };
      saveFiles[id] = await invoke<SaveFile[]>('scan_game_saves', { id });
      saveFiles = { ...saveFiles };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
      syncMessages = { ...syncMessages, [id]: '' };
    }
  }

  async function resolveConflict(id: string, relativePath: string, resolution: 'local' | 'remote') {
    error = '';
    try {
      await invoke<ConflictResolutionResult>('resolve_sync_conflict', {
        id,
        relativePath,
        resolution,
      });
      syncConflicts = {
        ...syncConflicts,
        [id]: (syncConflicts[id] ?? []).filter((path) => path !== relativePath),
      };
      syncMessages = { ...syncMessages, [id]: `Conflit résolu : ${relativePath}` };
      saveFiles[id] = await invoke<SaveFile[]>('scan_game_saves', { id });
      saveFiles = { ...saveFiles };
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
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
      <a class="nav-item" class:active={showCloudPanel} href="/sync" on:click|preventDefault={openCloudPanel}>Synchronisation</a>
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

    {#if showCloudPanel}
      <div class="cloud-panel">
        <p class="eyebrow">Stockage Cloud</p>
        <h2>Google Drive</h2>
        <p>La configuration reste locale. Aucun secret OAuth n’est enregistré dans le dépôt.</p>
        <div class="cloud-status">
          <span class:connected={cloudStatus?.connected}>{cloudStatus?.connected ? 'Compte connecté' : 'Non connecté'}</span>
          <small>{cloudStatus?.provider ?? 'Google Drive'} · {cloudStatus?.configured ? 'Client configuré' : 'Configuration requise'}</small>
        </div>
        {#if cloudStatus && !cloudStatus.connected}
          <p class="cloud-hint">Enregistrez la configuration, puis cliquez sur « Se connecter avec Google ». Les boutons Drive resteront désactivés tant que cette étape n’est pas terminée.</p>
        {/if}
        <form on:submit|preventDefault={saveGoogleClientId}>
          <label for="google-client-id">Identifiant client OAuth Google</label>
          <input id="google-client-id" bind:value={googleClientId} placeholder="123456.apps.googleusercontent.com" autocomplete="off" />
          <button class="primary cloud-save" type="submit">Enregistrer la configuration</button>
        </form>
        <div class="cloud-actions">
          <button class="secondary" disabled={!cloudStatus?.configured || cloudStatus?.connected} on:click={connectGoogle}>Se connecter avec Google</button>
          <button class="secondary" disabled={!cloudStatus?.connected} on:click={verifyGoogleDrive}>Tester Google Drive</button>
          <button class="text-button" disabled={!cloudStatus?.connected} on:click={disconnectGoogle}>Déconnecter le compte</button>
        </div>
        {#if cloudMessage}<div class="backup-message">{cloudMessage}</div>{/if}
        <button class="text-button" on:click={() => (showCloudPanel = false)}>Retour à la bibliothèque</button>
      </div>
    {:else if showAddPanel}
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
                <span>{game.save_directory ? 'Sauvegardes AppData associées' : game.executable ? 'Exécutable détecté' : 'Lancement à configurer'}</span>
              </div>
              <button class:running={runningIds.includes(game.id)} class="launch-button" disabled={!game.executable || runningIds.includes(game.id)} on:click={() => launchGame(game.id)}>
                {runningIds.includes(game.id) ? 'Jeu en cours' : 'Lancer le jeu'}
              </button>
              <button class="save-button" on:click={() => toggleSaves(game.id)}>
                {loadingSaves === game.id ? 'Analyse en cours…' : expandedGame === game.id ? 'Masquer les sauvegardes' : 'Voir les sauvegardes'}
              </button>
              <button class="backup-button" on:click={() => backupSaves(game.id)}>Créer un backup local</button>
              <button class="backup-button" disabled={!cloudStatus?.connected} title={cloudStatus?.connected ? 'Envoyer les sauvegardes vers Google Drive' : 'Connectez d’abord un compte Google dans Synchronisation'} on:click={() => syncGame(game.id)}>Synchroniser vers Drive</button>
              <button class="backup-button" disabled={!cloudStatus?.connected} title={cloudStatus?.connected ? 'Récupérer les sauvegardes depuis Google Drive' : 'Connectez d’abord un compte Google dans Synchronisation'} on:click={() => pullGame(game.id)}>Récupérer depuis Drive</button>
              <button class="backup-button" on:click={() => loadBackups(game.id)}>
                {loadingBackups === game.id ? 'Chargement…' : 'Historique des backups'}
              </button>
              {#if backupMessages[game.id]}
                <span class="backup-message">{backupMessages[game.id]}</span>
              {/if}
              {#if syncMessages[game.id]}
                <span class="backup-message">{syncMessages[game.id]}</span>
              {/if}
              {#if syncConflicts[game.id]?.length}
                <div class="conflict-list">
                  <strong>Conflits à résoudre</strong>
                  {#each syncConflicts[game.id] as conflict}
                    <div class="conflict-row">
                      <span title={conflict}>{conflict}</span>
                      <button on:click={() => resolveConflict(game.id, conflict, 'local')}>Local</button>
                      <button on:click={() => resolveConflict(game.id, conflict, 'remote')}>Drive</button>
                    </div>
                  {/each}
                </div>
              {/if}
              {#if backups[game.id]?.length}
                <div class="backup-history">
                  {#each backups[game.id] as backup}
                    <div class="backup-row">
                      <span>{backup.created_at} · {backup.file_count} fichier{backup.file_count === 1 ? '' : 's'}</span>
                      <button on:click={() => restoreBackup(game.id, backup.directory)}>Restaurer</button>
                    </div>
                  {/each}
                </div>
              {/if}
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
