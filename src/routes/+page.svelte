<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import LoginForm from '../lib/LoginForm.svelte';
  import ProfileForm from '../lib/ProfileForm.svelte';
  import CommentForm from '../lib/CommentForm.svelte';
  import CommentItem from '../lib/CommentItem.svelte';
  import './_page.css';

  interface CommentData {
    id: string;
    text: string;
    sentiment: string;
    score: number;
  }

  let loggedIn = false;
  let showProfileForm = false;
  let youtubeUrl = '';
  let commentData: CommentData[] = [];
  let isLoading = false;
  let apiKey = '';

  async function handleLogin(event: CustomEvent<{ username: string; loggedIn: boolean }>) {
    if (event.detail.loggedIn) {
      loggedIn = true;
      await fetchApiKey(event.detail.username);
    }
  }

  async function fetchApiKey(username: string) {
    try {
      const storedApiKey = await invoke<string>('get_api_key', { username });
      console.log('Stored API key:', storedApiKey);
      if (storedApiKey) {
        apiKey = storedApiKey;
      }
    } catch (error) {
      console.error('Error fetching API key:', error);
    }
  }

  function handleProfileCreated() {
    showProfileForm = false;
  }

  function logout() {
    loggedIn = false;
    apiKey = '';
  }

  async function analyzeComments() {
    isLoading = true;
    try {
      commentData = await invoke<CommentData[]>('analyze_youtube_comments', { url: youtubeUrl, apiKey });
      console.log('Comments analyzed successfully');
    } catch (error) {
      console.error('Error analyzing comments:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<main>
  {#if !loggedIn}
    {#if !showProfileForm}
      <LoginForm on:loggedIn={handleLogin} />
      <button on:click="{() => showProfileForm = true}">Create Profile</button>
    {:else}
      <ProfileForm on:profileCreated="{handleProfileCreated}" />
    {/if}
  {:else}
    <button on:click="{logout}">Logout</button>
    <CommentForm bind:youtubeUrl={youtubeUrl} onSubmit={analyzeComments} bind:isLoading={isLoading} />
    <div class="results-container">
      {#if commentData.length > 0}
        {#each commentData as comment, index}
          <CommentItem {comment} {index} />
        {/each}
      {:else if isLoading}
        <p>Analyzing comments, please wait...</p>
      {:else}
        <p>Enter a YouTube URL and click "Analyze Comments" to see the results.</p>
      {/if}
    </div>
  {/if}
</main>
