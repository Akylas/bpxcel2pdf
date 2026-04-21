<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { check as checkUpdate, Update } from '@tauri-apps/plugin-updater';
  import { InlineNotification, Modal, ProgressBar } from 'carbon-components-svelte';
  import { _ } from 'svelte-i18n';
  type UpdateStatus = 'idle' | 'checking' | 'available' | 'uptodate' | 'downloading' | 'downloaded' | 'error';

  export let open = false;

  let status: UpdateStatus = 'idle';
  let errorMessage = '';
  let updateVersion = '';
  let changelog = '';
  let downloadProgress = 0;
  let currentVersion = '';
  let update: Update;

  async function startCheck() {
    status = 'checking';
    errorMessage = '';
    try {
      currentVersion = await getVersion();
      update = await checkUpdate();
      if (update) {
        updateVersion = update.version;
        status = 'available';
        changelog = update.body
      } else {
        status = 'uptodate';
      }
    } catch (err) {
      status = 'error';
      errorMessage = err instanceof Error ? err.message : String(err);
    }
  }

  async function downloadAndInstall() {
    status = 'downloading';
    // Use milestone values to show coarse progress (pending → downloading → done).
    // The tauri updater event does not expose byte-level progress in v1.
    downloadProgress = 0;
    let contentLength;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
           contentLength = event.data.contentLength;
        }
         else if (event.event === 'Progress') {
          downloadProgress =  Math.round(event.data.chunkLength / contentLength * 100)
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

  $: primaryButtonText = getPrimaryButtonText(status);
  $: primaryButtonDisabled = ['checking', 'downloading', 'uptodate', 'error'].includes(status);

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

<Modal
  size="sm"
  bind:open
  modalHeading={$_('update_check_for_updates')}
  primaryButtonText={primaryButtonText}
  primaryButtonDisabled={primaryButtonDisabled}
  secondaryButtonText={status !== 'downloaded' ? $_('close') : undefined}
  on:click:button--primary={handlePrimary}
  on:click:button--secondary={() => (open = false)}
  on:close={() => (open = false)}
>
  <div style="padding: 0 1rem 1rem;">
    {#if status === 'checking'}
      <ProgressBar helperText={$_('update_checking')} />
    {:else if status === 'uptodate'}
      <InlineNotification
        lowContrast
        kind="success"
        title={$_('update_up_to_date')}
        subtitle={$_('update_up_to_date_desc')}
      />
    {:else if status === 'available'}
      <InlineNotification
        lowContrast
        kind="info"
        title={$_('update_available')}
        subtitle="{currentVersion} → {updateVersion}"
      />
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
              background: var(--cds-field-01);
              padding: 0.75rem;
              border-radius: 4px;
            "
          >{changelog}</div>
        </div>
      {/if}
    {:else if status === 'downloading'}
      <ProgressBar
        value={downloadProgress}
        helperText={$_('update_downloading')}
      />
    {:else if status === 'downloaded'}
      <InlineNotification
        lowContrast
        kind="success"
        title={$_('update_downloaded')}
        subtitle={$_('update_downloaded_desc')}
      />
    {:else if status === 'error'}
      <InlineNotification
        lowContrast
        kind="error"
        title={$_('update_error')}
        subtitle={errorMessage}
      />
    {/if}
  </div>
</Modal>
