<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { readFile } from '@tauri-apps/plugin-fs';
  import { basename } from '@tauri-apps/api/path';
  import { open as openURl } from '@tauri-apps/plugin-shell';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    Button,
    DataTable,
    Header,
    HeaderGlobalAction,
    HeaderUtilities,
    SkipToContent,
    Modal,
    Tabs,
    Tab,
    TabContent,
    TextInput,
    Toolbar,
    ToolbarContent,
  } from 'carbon-components-svelte';
  import Printer16 from 'carbon-icons-svelte/lib/Printer.svelte';
  import SettingsAdjust16 from 'carbon-icons-svelte/lib/SettingsAdjust.svelte';
  import TrashCan16 from 'carbon-icons-svelte/lib/TrashCan.svelte';
  import { _ } from 'svelte-i18n';
  import FileDrop from 'svelte-tauri-filedrop';
  import * as xlsx from 'xlsx';

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
  let selected = $state(0);
  let addIgnoredNew = $state<string | undefined>(undefined);
  let addIgnoredFieldNew = $state<string | undefined>(undefined);

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
    const unlistenPromise = listen<string>('menu', async ({ payload }) => {
      console.log('payload', payload);
      switch (payload) {
        case 'learn_more':
          openURl(REPO_URL);
          break;
        case 'print':
          printPDF();
          break;
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  async function deleteIgnoredItem(row: { id?: string }) {
    if (row.id) {
      const index = ignoredNames.indexOf(row.id);
      if (index >= 0) {
        ignoredNames.splice(index, 1);
        localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
      }
    }
  }

  async function addIgnoredItem() {
    if (addIgnoredNew) {
      ignoredNames.push(addIgnoredNew);
      localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
      addIgnoredNew = undefined;
      addIgnoredOpened = false;
    }
  }

  async function onAddingIgnoredChange(event: CustomEvent<string>) {
    addIgnoredNew = event.detail;
  }

  async function deleteIgnoredField(row: { id?: string }) {
    if (row.id) {
      const index = ignoredFields.indexOf(row.id);
      if (index >= 0) {
        ignoredFields.splice(index, 1);
        localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
      }
    }
  }

  async function addIgnoredField() {
    if (addIgnoredFieldNew) {
      ignoredFields.push(addIgnoredFieldNew);
      localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
      addIgnoredFieldNew = undefined;
      addIgnoredFieldOpened = false;
    }
  }

  async function onAddingIgnoredFieldChange(event: CustomEvent<string>) {
    addIgnoredFieldNew = event.detail;
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
  <Header company="Bonne Pioche" platformName="Excel 2 PDF">
    <HeaderUtilities>
      <HeaderGlobalAction aria-label={$_('print')} icon={Printer16} on:click={printPDF} />
      <HeaderGlobalAction
        aria-label={$_('settings')}
        icon={SettingsAdjust16}
        on:click={() => (settingsOpened = !settingsOpened)}
      />
    </HeaderUtilities>
  </Header>
  <Tabs bind:selected style="padding-top:3rem;width:100%;">
    <Tab label="Précommandes" />
    <!-- <Tab label="Résumé Forme" /> -->
    <svelte:fragment slot="content">
      <TabContent style="height:100%;padding-top:3rem;">
        <div style="padding-top:10px; flex:auto;display:flex;height:100% !important;justify-content:center;">
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
                <!-- <div class="card"> -->
                <!-- </div> -->
              </div>
            </div>
          {:else}
            <Button style="align-self:center;text-align:center;" on:click={openFile}
              >{$_('drag_xslx_file')}</Button
            >
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
      </TabContent>
      <!-- <TabContent /> -->
    </svelte:fragment>
  </Tabs>

  <Modal
    size="sm"
    bind:open={settingsOpened}
    modalHeading={$_('settings')}
    primaryButtonText={$_('close')}
    on:click:button--primary={() => (settingsOpened = false)}
    on:open
    on:close={reload}
    on:submit
  >
    <div style="padding:10px">
      <DataTable
        title={$_('ignored')}
        description={$_('ignored_items')}
        size="short"
        headers={[
          { key: 'name', value: $_('name') },
          { key: 'overflow', empty: true },
        ]}
        rows={ignoredNames.map((s) => ({ id: s, name: s }))}
      >
        <svelte:fragment slot="cell" let:cell let:row>
          {#if cell.key === 'overflow'}
            <Button
              kind="danger-ghost"
              iconDescription={$_('delete')}
              size="small"
              icon={TrashCan16}
              on:click={() => deleteIgnoredItem(row)}
            />
            <!-- <OverflowMenu flipped>
              <OverflowMenuItem danger text={$_('delete')} on:click={deleteIgnoredItem} />
            </OverflowMenu> -->
          {:else}{cell.value}{/if}
        </svelte:fragment>
        <Toolbar size="sm">
          <ToolbarContent>
            <Button on:click={() => (addIgnoredOpened = true)}>{$_('add')}</Button>
          </ToolbarContent>
        </Toolbar>
      </DataTable>

      <DataTable
        title={$_('ignored_fields')}
        description={$_('ignored_fields_items')}
        size="short"
        headers={[
          { key: 'name', value: $_('name') },
          { key: 'overflow', empty: true },
        ]}
        rows={ignoredFields.map((s) => ({ id: s, name: s }))}
      >
        <svelte:fragment slot="cell" let:cell let:row>
          {#if cell.key === 'overflow'}
            <Button
              kind="danger-ghost"
              iconDescription={$_('delete')}
              size="small"
              icon={TrashCan16}
              on:click={() => deleteIgnoredField(row)}
            />
            <!-- <OverflowMenu flipped>
              <OverflowMenuItem danger text={$_('delete')} on:click={deleteIgnoredItem} />
            </OverflowMenu> -->
          {:else}{cell.value}{/if}
        </svelte:fragment>
        <Toolbar size="sm">
          <ToolbarContent>
            <Button on:click={() => (addIgnoredFieldOpened = true)}>{$_('add')}</Button>
          </ToolbarContent>
        </Toolbar>
      </DataTable>
    </div>
  </Modal>

  <Modal
    bind:open={addIgnoredOpened}
    modalHeading={$_('add_ignored')}
    primaryButtonText={$_('add')}
    secondaryButtonText={$_('cancel')}
    on:click:button--secondary={() => (addIgnoredOpened = false)}
    on:open
    on:close
    on:submit={addIgnoredItem}
  >
    <TextInput labelText={$_('text')} on:change={onAddingIgnoredChange} />
  </Modal>
  <Modal
    bind:open={addIgnoredFieldOpened}
    modalHeading={$_('add_ignored')}
    primaryButtonText={$_('add')}
    secondaryButtonText={$_('cancel')}
    on:click:button--secondary={() => (addIgnoredFieldOpened = false)}
    on:open
    on:close
    on:submit={addIgnoredField}
  >
    <TextInput labelText={$_('text')} on:change={onAddingIgnoredFieldChange} />
  </Modal>
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
    border: 1 solid #eee;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .droppable {
    background: #d6dff088;
  }
</style>
