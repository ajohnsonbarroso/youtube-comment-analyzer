<script lang="ts">
    export let youtubeUrl: string;
    export let onSubmit: () => void;
    export let isLoading: boolean;
  </script>
  
  <form class="row" on:submit|preventDefault={onSubmit}>
    <input type="text" id="youtube-url" placeholder="Enter YouTube URL" bind:value={youtubeUrl} />
    <button type="submit" disabled={isLoading}>
      {isLoading ? 'Analyzing...' : 'Analyze Comments'}
    </button>
  </form>