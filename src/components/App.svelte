<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { readFile } from '@tauri-apps/plugin-fs';
  import { basename } from '@tauri-apps/api/path';
  import { open as openURl } from '@tauri-apps/plugin-shell';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    Alert,
    Button,
    Input,
    Label,
    Modal,
    Navbar,
    NavBrand,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Tabs,
    TabItem,
  } from 'flowbite-svelte';
  import { PrinterOutline, AdjustmentsHorizontalOutline, TrashBinOutline } from 'flowbite-svelte-icons';
  import { _ } from 'svelte-i18n';
  import FileDrop from 'svelte-tauri-filedrop';
  import * as xlsx from 'xlsx';
  import UpdateDialog from './UpdateDialog.svelte';

  let updateDialogRef: UpdateDialog;
  let settingsOpened = $state(false);
  let addIgnoredOpened = $state(false);
  let addIgnoredFieldOpened = $state(false);
  let firstnameFields: string[] = $state(
    JSON.parse(localStorage.getItem('firstnameField')) || ['Prénom']
  );
  let nameFields: string[] = $state(
    JSON.parse(localStorage.getItem('nameField')) || ['Nom', 'Nom de famille']
  );
  let phoneFields: string[] = $state(
    JSON.parse(localStorage.getItem('phoneField')) || ['Téléphone']
  );
  let ignoredFields: string[] = $state(
    JSON.parse(
      localStorage.getItem('ignoredFields') ||
        JSON.stringify(['Horodateur', 'Adresse email', 'Téléphone', 'Commentaires libres'])
    )
  );
  let ignoredNames: string[] = $state(
    JSON.parse(
      localStorage.getItem('ignoredNames') ||
        JSON.stringify([
          'Reliquat magasin',
          'Total commandés aux fournisseurs',
          'TOTAL CLIENT',
          'Total commandé au producteurs',
          'En plus pour le magasin',
          'Total précommandes',
        ])
    )
  );

  let json: { [k: string]: string }[] | null = $state(null);
  let currentFilePath: string | null = $state(null);
  let total = $state(0);
  let addIgnoredNew = $state('');
  let addIgnoredFieldNew = $state('');

  async function openFile() {
    try {
      const resPath = await open({
        filters: [{ name: 'excel', extensions: ['xlsx', 'csv'] }],
        multiple: false,
        directory: false,
      });
      handleDroppedFile(Array.isArray(resPath) ? resPath : [resPath]);
    } catch (error) {
      console.error(error);
    }
  }

  async function reload() {
    if (currentFilePath) {
      handleDroppedFile([currentFilePath]);
    }
  }

  async function handleDroppedFile(paths: string[]) {
    currentFilePath = paths[0];
    const source = await readFile(currentFilePath);
    const xlsxData = xlsx.read(source, { type: 'array' });
    const sheet = xlsxData.Sheets[xlsxData.SheetNames[0]];
    json = xlsx.utils.sheet_to_json(sheet);
    json = json.filter(
      (c) =>
        nameFields.every((nameField) => ignoredNames.indexOf(c[nameField]) === -1) ||
        phoneFields.every((phoneField) => ignoredNames.indexOf(c[phoneField]) === -1)
    );

    total = json.length;

    json.forEach((d) => {
      ignoredFields.forEach((k) => delete d[k]);
    });
    document.title = await basename(paths[0]);
  }

  async function printPDF() {
    window.print();
  }

  $effect(() => {
    const win = getCurrentWindow();
    const unlistenPromise = listen<string>('menu', async ({ payload }) => {
      switch (payload) {
        case 'open':
          openFile();
          break;
        case 'learn_more':
          openURl(REPO_URL);
          break;
        case 'check_update':
          updateDialogRef?.checkForUpdates();
          break;
        case 'print':
          printPDF();
          break;
        case 'fullscreen':
          win.setFullscreen(!(await win.isFullscreen()));
          break;
        case 'minimize':
          win.minimize();
          break;
        case 'maximize':
          win.toggleMaximize();
          break;
        case 'close_window':
          win.close();
          break;
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  async function deleteIgnoredItem(name: string) {
    const index = ignoredNames.indexOf(name);
    if (index >= 0) {
      ignoredNames.splice(index, 1);
      localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
    }
  }

  async function addIgnoredItem() {
    if (addIgnoredNew) {
      ignoredNames.push(addIgnoredNew);
      localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
      addIgnoredNew = '';
      addIgnoredOpened = false;
    }
  }

  async function deleteIgnoredField(field: string) {
    const index = ignoredFields.indexOf(field);
    if (index >= 0) {
      ignoredFields.splice(index, 1);
      localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
    }
  }

  async function addIgnoredField() {
    if (addIgnoredFieldNew) {
      ignoredFields.push(addIgnoredFieldNew);
      localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
      addIgnoredFieldNew = '';
      addIgnoredFieldOpened = false;
    }
  }

  function groupBy(objectArray: any[], filter: (obj: any) => string) {
    return objectArray.reduce(
      (acc, obj) => {
        const key = filter(obj);
        if (!acc[key]) {
          acc[key] = [];
        }
        acc[key].push(obj);
        return acc;
      },
      {} as Record<string, any[]>
    );
  }

  function computeValue(k: string, value: string) {
    try {
      const parsed = JSON.parse(value);
      if (Array.isArray(parsed)) {
        if (parsed.length === 1) {
          return parsed[0];
        }
        if (!/\d+/.test(parsed[0])) {
          return parsed.join(', ');
        }
        const group = groupBy(parsed, (s) => s.replace(/\d+/, '%s'));
        const actual: string[] = [];
        Object.keys(group).forEach((k) => {
          const total = group[k].reduce((acc: number, obj: string) => {
            return (
              acc + (obj.match(/\d+/g) || []).map((n) => parseInt(n, 10)).reduce((a, b) => a + b, 0)
            );
          }, 0);
          actual.push(k.replace('%s', String(total)));
        });
        return actual.join(', ');
      } else {
        return value;
      }
    } catch (error) {
      return value;
    }
  }
</script>

<div class="container">
  <Navbar class="fixed top-0 w-full z-50 border-b border-gray-200 dark:border-gray-700 px-4">
    <NavBrand href="#">
      <span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">
        Bonne Pioche&nbsp;<span class="text-gray-500 font-normal text-base">Excel 2 PDF</span>
      </span>
    </NavBrand>
    <div class="flex items-center gap-1 ml-auto">
      <Button size="sm" color="alternative" aria-label={$_('print')} onclick={printPDF}>
        <PrinterOutline class="w-5 h-5" />
      </Button>
      <Button
        size="sm"
        color="alternative"
        aria-label={$_('settings')}
        onclick={() => (settingsOpened = true)}
      >
        <AdjustmentsHorizontalOutline class="w-5 h-5" />
      </Button>
    </div>
  </Navbar>

  <div style="padding-top: 4rem; height: 100%; display: flex; flex-direction: column;">
    <Tabs style="width:100%">
      <TabItem open title="Précommandes">
        <div
          style="padding-top:10px; flex:auto;display:flex;height:100%;justify-content:center;position:relative;"
        >
          {#if json}
            <div
              id="section-to-print"
              style="overflow-y: scroll;overflow-x: hidden;height:100%;width:100%;"
            >
              <div style="overflow-y: hidden;overflow-x: hidden;">
                <div class="cardTitle" style="margin-bottom: 50px;">
                  Total Précommandes: {total}
                </div>
                {#each json as object}
                  <div class="card">
                    <div class="cardTitle">
                      {object[nameFields.find((v) => !!object[v])] || ''}
                      {object[firstnameFields.find((v) => !!object[v])] || ''}
                    </div>
                    {#each Object.keys(object) as k}
                      {#if !nameFields.includes(k) && !firstnameFields.includes(k)}
                        <span style="font-size:26px">☐</span>&nbsp;<span
                          style="break-inside: avoid;margin-top:3px;display:inline-block;"
                          >{k}:&nbsp;&nbsp;&nbsp;</span
                        >
                        <span style="font-weight: bold;">{computeValue(k, object[k])}</span><br />
                      {/if}
                    {/each}
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <Button style="align-self:center;text-align:center;" onclick={openFile}>
              {$_('drag_xslx_file')}
            </Button>
          {/if}
          <FileDrop extensions={['xlsx', 'csv']} handleFiles={handleDroppedFile} let:files>
            <div class="dropzone" class:droppable={files.length > 0}>
              {#if files.length > 0}
                <h1 style:text-align="center" style:word-break="break-word">
                  {$_('file')}: <br />
                  {files[0]}
                </h1>
              {/if}
            </div>
          </FileDrop>
        </div>
      </TabItem>
    </Tabs>
  </div>

  <!-- Settings Modal -->
  <Modal bind:open={settingsOpened} title={$_('settings')} size="sm" onclose={reload}>
    <div class="space-y-6">
      <!-- Ignored Names Table -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <div>
            <h3 class="text-base font-semibold text-gray-900 dark:text-white">{$_('ignored')}</h3>
            <p class="text-sm text-gray-500">{$_('ignored_items')}</p>
          </div>
          <Button size="sm" onclick={() => (addIgnoredOpened = true)}>{$_('add')}</Button>
        </div>
        <Table>
          <TableHead>
            <TableHeadCell>{$_('name')}</TableHeadCell>
            <TableHeadCell></TableHeadCell>
          </TableHead>
          <TableBody>
            {#each ignoredNames as name}
              <TableBodyRow>
                <TableBodyCell>{name}</TableBodyCell>
                <TableBodyCell>
                  <Button color="red" size="xs" onclick={() => deleteIgnoredItem(name)}>
                    <TrashBinOutline class="w-4 h-4" />
                  </Button>
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </div>

      <!-- Ignored Fields Table -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <div>
            <h3 class="text-base font-semibold text-gray-900 dark:text-white">
              {$_('ignored_fields')}
            </h3>
            <p class="text-sm text-gray-500">{$_('ignored_fields_items')}</p>
          </div>
          <Button size="sm" onclick={() => (addIgnoredFieldOpened = true)}>{$_('add')}</Button>
        </div>
        <Table>
          <TableHead>
            <TableHeadCell>{$_('name')}</TableHeadCell>
            <TableHeadCell></TableHeadCell>
          </TableHead>
          <TableBody>
            {#each ignoredFields as field}
              <TableBodyRow>
                <TableBodyCell>{field}</TableBodyCell>
                <TableBodyCell>
                  <Button color="red" size="xs" onclick={() => deleteIgnoredField(field)}>
                    <TrashBinOutline class="w-4 h-4" />
                  </Button>
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </div>
    </div>
    {#snippet footer()}
      <Button onclick={() => (settingsOpened = false)}>{$_('close')}</Button>
    {/snippet}
  </Modal>

  <!-- Add Ignored Name Modal -->
  <Modal bind:open={addIgnoredOpened} title={$_('add_ignored')} size="sm">
    <Label class="block">
      {$_('text')}
      <Input bind:value={addIgnoredNew} class="mt-1" />
    </Label>
    {#snippet footer()}
      <Button onclick={addIgnoredItem}>{$_('add')}</Button>
      <Button color="alternative" onclick={() => (addIgnoredOpened = false)}>{$_('cancel')}</Button>
    {/snippet}
  </Modal>

  <!-- Add Ignored Field Modal -->
  <Modal bind:open={addIgnoredFieldOpened} title={$_('add_ignored')} size="sm">
    <Label class="block">
      {$_('text')}
      <Input bind:value={addIgnoredFieldNew} class="mt-1" />
    </Label>
    {#snippet footer()}
      <Button onclick={addIgnoredField}>{$_('add')}</Button>
      <Button color="alternative" onclick={() => (addIgnoredFieldOpened = false)}>
        {$_('cancel')}
      </Button>
    {/snippet}
  </Modal>

  <UpdateDialog bind:this={updateDialogRef} />
</div>

<style>
  .dropzone {
    position: absolute;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 100;
    padding: 20px;
    background: transparent;
    border: 1px solid #eee;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .droppable {
    background: #d6dff088;
  }
</style>

