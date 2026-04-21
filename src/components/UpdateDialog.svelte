<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { check as checkUpdate, Update } from '@tauri-apps/plugin-updater';
  import { Alert, Button, Modal, Progressbar } from 'flowbite-svelte';
  import { _ } from 'svelte-i18n';

  type UpdateStatus =
    | 'idle'
    | 'checking'
    | 'available'
    | 'uptodate'
    | 'downloading'
    | 'downloaded'
    | 'error';

  let open = $state(false);
  let status: UpdateStatus = $state('idle');
  let errorMessage = $state('');
  let updateVersion = $state('');
  let changelog = $state('');
  let downloadProgress = $state(0);
  let currentVersion = $state('');
  let update: Update | null = $state(null);

  async function startCheck() {
    status = 'checking';
    errorMessage = '';
    try {
      currentVersion = await getVersion();
      update = await checkUpdate();
      if (update) {
        updateVersion = update.version;
        status = 'available';
        changelog = update.body ?? '';
      } else {
        status = 'uptodate';
      }
    } catch (err) {
      status = 'error';
      errorMessage = err instanceof Error ? err.message : String(err);
    }
  }

  async function downloadAndInstall() {
    if (!update) return;
    status = 'downloading';
    downloadProgress = 0;
    let contentLength: number | undefined;
    let downloaded = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          downloadProgress = contentLength ? Math.round((downloaded / contentLength) * 100) : 0;
        } else if (event.event === 'Finished') {
          downloadProgress = 100;
          status = 'downloaded';
        }
      });
      status = 'downloaded';
    } catch (err) {
      status = 'error';
      errorMessage = err instanceof Error ? err.message : String(err);
    }
  }

  async function restartApp() {
    await relaunch();
  }

  export async function checkForUpdates() {
    open = true;
    await startCheck();
  }

  const primaryButtonText = $derived(getPrimaryButtonText(status));
  const primaryButtonDisabled = $derived(
    ['checking', 'downloading', 'uptodate', 'error'].includes(status)
  );

  function getPrimaryButtonText(s: UpdateStatus): string {
    if (s === 'downloaded') return $_('update_restart');
    if (s === 'available') return $_('update_download_install');
    return $_('close');
  }

  function handlePrimary() {
    if (status === 'downloaded') {
      restartApp();
    } else if (status === 'available') {
      downloadAndInstall();
    } else {
      open = false;
    }
  }
</script>

<Modal bind:open title={$_('update_check_for_updates')} size="sm" onclose={() => (open = false)}>
  <div style="padding: 0 0 1rem;">
    {#if status === 'checking'}
      <Progressbar labelOutside={$_('update_checking')} animate />
    {:else if status === 'uptodate'}
      <Alert color="green">
        <span class="font-medium">{$_('update_up_to_date')}</span>
        {$_('update_up_to_date_desc')}
      </Alert>
    {:else if status === 'available'}
      <Alert color="blue">
        <span class="font-medium">{$_('update_available')}</span>
        {currentVersion} → {updateVersion}
      </Alert>
      {#if changelog}
        <div style="margin-top: 1rem;">
          <p style="font-weight: 600; margin-bottom: 0.5rem;">{$_('update_changelog')}:</p>
          <!-- Changelog is rendered as plain text via Svelte's auto-escaping (no @html), so user-supplied content is safe. -->
          <div
            style="
              white-space: pre-wrap;
              font-size: 0.875rem;
              max-height: 200px;
              overflow-y: auto;
              background: #f9fafb;
              padding: 0.75rem;
              border-radius: 4px;
            "
          >{changelog}</div>
        </div>
      {/if}
    {:else if status === 'downloading'}
      <Progressbar progress={String(downloadProgress)} labelOutside={$_('update_downloading')} />
    {:else if status === 'downloaded'}
      <Alert color="green">
        <span class="font-medium">{$_('update_downloaded')}</span>
        {$_('update_downloaded_desc')}
      </Alert>
    {:else if status === 'error'}
      <Alert color="red">
        <span class="font-medium">{$_('update_error')}</span>
        {errorMessage}
      </Alert>
    {/if}
  </div>
  {#snippet footer()}
    <Button onclick={handlePrimary} disabled={primaryButtonDisabled}>{primaryButtonText}</Button>
    {#if status !== 'downloaded'}
      <Button color="alternative" onclick={() => (open = false)}>{$_('close')}</Button>
    {/if}
  {/snippet}
</Modal>

