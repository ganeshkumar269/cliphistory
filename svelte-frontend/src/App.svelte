<!-- frontend/src/App.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  interface ClipItem {
    value: string;
    source: string | null;
    hash: string;
    timestamp: number;
  }

  const clips = writable<ClipItem[]>([]);
  const uniqueSourceApps = writable<string[]>([]);
  const searchTerm = writable('');
  const selectedSourceApp = writable(''); // Stores the value of the selected source app filter

  let unlistenClipsUpdated: UnlistenFn | null = null;

  async function fetchAndRenderFilteredClips() {
    const termValue = $searchTerm.trim() || null;
    const sourceAppValue = $selectedSourceApp || null;

    try {
      console.log(`Svelte: Fetching with term: "${termValue}", source_app: "${sourceAppValue}"`);
      const results = await invoke<ClipItem[]>('on_search', {
        term: termValue === null ? "" : termValue,
        sourceAppFilter: sourceAppValue === null ? "" : sourceAppValue, // Ensure your Rust command expects this param name
      });
      clips.set(results);
    } catch (error) {
      console.error("Error during search/filter:", error);
      clips.set([]);
    }
  }

  async function populateSourceAppDropdown() {
    try {
      const apps = await invoke<string[]>('get_unique_source_apps');
      uniqueSourceApps.set(apps.filter(app => app && app.trim() !== ''));
    } catch (error) {
      console.error("Error fetching unique source apps:", error);
    }
  }

  async function handleClipClick(clickedClip: ClipItem) {
    try {
      await invoke('update_clipboard_on_click', { clipText: clickedClip.value });
      console.log('Called update_clipboard_on_click for:', clickedClip.value);
      clips.update(el => {
        let filteredEl = el.filter(e => e.hash != clickedClip.hash)
        filteredEl.unshift(clickedClip)
        return filteredEl
      })
    } catch (error) {
      console.error('Failed to handle clip click:', error);
    }
  }

  // --- Event Handlers for UI ---
  function handleSearchInput(event) {
    console.log(`HandleSearchInput: ${event}`)
    const inputElement = event.target as HTMLInputElement;
    searchTerm.set(inputElement.value);
    fetchAndRenderFilteredClips()
  }

  function handleSourceAppChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedSourceApp.set(target.value);
    console.log(`selected source: ${event.target.value}`)
    fetchAndRenderFilteredClips();
  }


  onMount(async () => {
    await populateSourceAppDropdown();
    await fetchAndRenderFilteredClips();

    unlistenClipsUpdated = await listen<string>('clips_updated', async (event) => {
      console.log('Svelte: Received clips_updated event.', event);
      const newClip: ClipItem = event.payload as unknown as ClipItem;

      if ($searchTerm.trim() === "" && $selectedSourceApp === "") {
        clips.update(currentClips => [newClip, ...currentClips]);
      } else {
        // If filtering is active, a full re-fetch is better to ensure consistency
        await fetchAndRenderFilteredClips();
      }
    });
  });

  onDestroy(() => {
    if (unlistenClipsUpdated) {
      unlistenClipsUpdated();
    }
  });

</script>
    <div class="search-container">
      <input type="text"  id="search-input-svelte" placeholder="Search clips..."  onkeydown={(e) => e.key === 'Enter' && handleSearchInput(e)}/>
    </div>
    <div id="source-app-filter-container">
      <select id="source-app-filter-svelte" onchange={handleSourceAppChange}>
        <option value="">All Apps</option>
        {#each $uniqueSourceApps as app (app)}
          <option value={app}>{app}</option>
        {/each}
      </select>
    </div>
  <div id="clip-list-container">
    <ul id="clip-list">
      {#if $clips.length === 0}
        <li class="no-clips-message">{$searchTerm || $selectedSourceApp ? 'No matching clips found.' : 'No clips yet.'}</li>
      {:else}
        {#each $clips as item (item.value + (item.source || '') + Math.random())} <!-- Key needs to be unique -->
          <li class="clip-content" onclick={() => handleClipClick(item)} title={item.value + (item.source ? `\nFrom: ${item.source}` : '')}>
              {item.value.length > 100 ? item.value.substring(0, 97) + '...' : item.value}
          </li>
        {/each}
      {/if}
    </ul>
  </div>
<style>
  .search-container {
    flex-grow: 1; /* Makes search container take available horizontal space */
    display: flex;
    align-items: center;
    background-color: #2d2d2d;
    border-radius: 5px;
    border: 1px solid #444;
  }

  #search-input-svelte { /* ID for Svelte component's search input */
    flex-grow: 1; /* Allows input to fill the search-container */
    padding: 6px 8px;
    border: 1px solid #555;
    background-color: #3a3a3a;
    color: #e0e0e0;
    border-radius: 3px; /* Keep consistent with dropdown */
    margin-right: 8px;
    outline: none;
  }
  #search-input-svelte:focus {
    border-color: #007bff;
  }

  #source-app-filter-container {
    margin-left: 10px; /* Space between search and dropdown */
  }

  #source-app-filter-svelte { /* ID for Svelte component's dropdown */
    padding: 6px 8px;
    border: 1px solid #555;
    background-color: #3a3a3a;
    color: #e0e0e0;
    border-radius: 3px;
    outline: none;
    min-width: 150px; /* Give it some base width */
  }
  #source-app-filter-svelte:focus {
    border-color: #007bff;
  }

  #clip-list-container {
    background-color: #2d2d2d;
    border: 1px solid #444;
    border-radius: 5px;
    padding: 10px; /* Padding inside the list container */
    flex-grow: 1; /* Allows the list to take remaining vertical space */
    overflow-y: auto; /* Enables vertical scroll for long lists */
  }

  #clip-list {
    list-style: none;
    padding: 0;
    margin: 0; /* Remove default ul margin */
  }

  #clip-list li {
    padding: 8px 10px; /* Padding within each list item */
    border-bottom: 1px solid #404040;
    cursor: pointer;
    transition: background-color 0.2s;
    color: #e0e0e0;
    display: flex;
    flex-direction: column; /* Stack content and source app vertically */
    overflow: hidden; /* Prevent content from breaking out of li boundaries */
  }
  #clip-list li:last-child {
    border-bottom: none;
  }
  #clip-list li:hover {
    background-color: #3a3a3a;
  }

  .clip-content {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0; /* Crucial for flex children to allow shrinking and ellipsis */
    text-align: left; /* Ensure text is left-aligned */
  }

  .no-clips-message {
    color: #777;
    text-align: center;
    padding: 20px; /* Give it some space */
    font-style: italic;
  }

</style>