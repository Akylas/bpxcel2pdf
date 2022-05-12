<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { readBinaryFile } from '@tauri-apps/api/fs';
  import { basename } from '@tauri-apps/api/path';
  import { open as openURl } from '@tauri-apps/api/shell';
  import {
    Header,
    HeaderGlobalAction,
    HeaderUtilities,
    SkipToContent,
  } from 'carbon-components-svelte';
  import Printer16 from 'carbon-icons-svelte/lib/Printer16';
  import { _ } from 'svelte-i18n';
  import FileDrop from 'svelte-tauri-filedrop';
  import * as xlsx from 'xlsx';

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
    json = xlsx.utils
      .sheet_to_json(sheet)
    json = json.filter(
        (c) =>
          ignoredNames.indexOf(c[nameField]) === -1 && ignoredNames.indexOf(c[phoneField]) === -1
      ) as any;

      total = json.length

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
</script>

<div class="container">
  <Header company="Bonne Pioche" platformName="Excel 2 PDF">
    <svelte:fragment slot="skip-to-content">
      <SkipToContent />
    </svelte:fragment>
    <HeaderUtilities>
      <HeaderGlobalAction aria-label={$_('print')} icon={Printer16} on:click={printPDF} />
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
