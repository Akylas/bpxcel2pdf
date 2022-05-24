<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { readBinaryFile } from '@tauri-apps/api/fs';
  import { basename } from '@tauri-apps/api/path';
  import { open as openURl } from '@tauri-apps/api/shell';
  import {
    Button,
    DataTable,
    Header,
    HeaderGlobalAction,
    HeaderUtilities,
    SkipToContent,
    Modal,
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

  let settingsOpened = false;
  let addIgnoredOpened = false;
  let addIgnoredFieldOpened = false;
  let firstnameField = localStorage.getItem('firstnameField') || 'Prénom';
  let nameField = localStorage.getItem('nameField') || 'Nom';
  let phoneField = localStorage.getItem('phoneField') || 'Téléphone';
  let ignoredFields = JSON.parse(
    localStorage.getItem('ignoredFields') ||
      JSON.stringify(['Horodateur', 'Adresse email', 'Téléphone', 'Commentaires libres'])
  );
  let ignoredNames = JSON.parse(
    localStorage.getItem('ignoredNames') ||
      JSON.stringify([
        'Reliquat magasin',
        'Total commandés aux fournisseurs',
        'TOTAL CLIENT',
        'Total commandé au producteurs',
        'En plus pour le magasin',
        'Total précommandes',
      ])
  );

  let json: { [k: string]: string }[] = null;
  let currentFilePath: string = null;
  let total = 0;
  async function handleDroppedFile(paths: string[]) {
    currentFilePath = paths[0];
    const source = await readBinaryFile(currentFilePath);
    const xlsxData = xlsx.read(source, { type: 'array' });
    const sheet = xlsxData.Sheets[xlsxData.SheetNames[0]];
    json = xlsx.utils.sheet_to_json(sheet);
    json = json.filter(
      (c) => ignoredNames.indexOf(c[nameField]) === -1 && ignoredNames.indexOf(c[phoneField]) === -1
    ) as any;

    total = json.length;

    json.forEach((d) => {
      ignoredFields.forEach((k) => delete d[k]);
      // Object.keys(d).forEach((k) => {
      //   // let index = k.indexOf('[');
      //   // if (index !== -1) {
      //   //   let index2 = k.indexOf(']');
      //   //   d[k.substring(index + 1, index2)] = d[k];
      //   //   delete d[k];
      //   // }
      // });
    });
    document.title = await basename(paths[0]);
  }
  async function printPDF() {
    window.print();
  }

  listen<string>('tauri://menu', async ({ payload }) => {
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

  async function deleteIgnoredItem(row) {
    if (row.id) {
      const index = ignoredNames.indexOf(row.id);
      if (index >= 0) {
        ignoredNames.splice(index, 1);

        localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
        ignoredNames = ignoredNames;
      }
    }
  }
  async function addIgnoredItem() {
    if (addIgnoredNew) {
      ignoredNames.push(addIgnoredNew);
      localStorage.setItem('ignoredNames', JSON.stringify(ignoredNames));
      ignoredNames = ignoredNames;
      addIgnoredNew = null;
      addIgnoredOpened = false;
    }
  }

  let addIgnoredNew;
  async function onAddingIgnoredChange(event) {
    addIgnoredNew = event.detail;
  }
  async function deleteIgnoredField(row) {
    if (row.id) {
      const index = ignoredFields.indexOf(row.id);
      if (index >= 0) {
        ignoredFields.splice(index, 1);

        localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
        ignoredFields = ignoredFields;
      }
    }
  }
  async function addIgnoredField() {
    if (addIgnoredFieldNew) {
      ignoredFields.push(addIgnoredFieldNew);
      localStorage.setItem('ignoredFields', JSON.stringify(ignoredFields));
      ignoredFields = ignoredFields;
      addIgnoredFieldNew = null;
      addIgnoredFieldOpened = false;
    }
  }

  let addIgnoredFieldNew;
  async function onAddingIgnoredFieldChange(event) {
    addIgnoredFieldNew = event.detail;
  }
</script>

<div class="container">
  <Header company="Bonne Pioche" platformName="Excel 2 PDF">
    <svelte:fragment slot="skip-to-content">
      <SkipToContent />
    </svelte:fragment>
    <HeaderUtilities>
      <HeaderGlobalAction aria-label={$_('print')} icon={Printer16} on:click={printPDF} />
      <HeaderGlobalAction
        aria-label={$_('settings')}
        icon={SettingsAdjust16}
        on:click={() => (settingsOpened = !settingsOpened)}
      />
    </HeaderUtilities>
  </Header>

  <div style="padding-top:3rem;flex:auto;display:flex;height:100%;">
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
                {object[nameField] || ''}
                {object[firstnameField] || ''}
              </div>
              {#each Object.keys(object) as k}
                {#if k !== nameField && k !== firstnameField && ignoredNames.indexOf(object[nameField]) === -1}
                  <span style="font-size:26px">☐</span>&nbsp;<span
                    style="break-inside: avoid;margin-top:3px;display:inline-block;"
                    >{k}:&nbsp;&nbsp;&nbsp;</span
                  >
                  <span style="font-weight: bold;">{object[k]}</span><br />
                {/if}
              {/each}
            </div>
          {/each}
          <!-- <div class="card"> -->
          <!-- </div> -->
        </div>
      </div>
    {/if}
    <FileDrop extensions={['xlsx', 'csv']} handleFiles={handleDroppedFile} let:files>
      <div class="dropzone" class:droppable={files.length > 0}>
        {#if files.length > 0}
          <h1 style:text-align="center" style:word-break="break-word">
            Import Excel: <br />
            {files[0]}
          </h1>
        {/if}
      </div>
    </FileDrop>
  </div>

  <Modal
    size="sm"
    bind:open={settingsOpened}
    modalHeading={$_('settings')}
    primaryButtonText={$_('save')}
    secondaryButtonText={$_('cancel')}
    on:click:button--secondary={() => (settingsOpened = false)}
    on:open
    on:close
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
