<script lang="ts">
  import './_CommentItem.css';
  export let comment: {
    id: string;
    text: string;
    sentiment: string;
    score: number;
  };

  export let index: number;

  let showId = false;

  function toggleId() {
    showId = !showId;
  }
</script>

<div class="comment-item">
  <button
    type="button"
    class="comment-header"
    on:click={toggleId}
    aria-expanded={showId}
    aria-controls="comment-id-{index}"
  >
    {showId ? 'Hide ID' : 'Show ID'}
  </button>

  {#if showId}
    <div class="comment-id" id="comment-id-{index}">ID: {comment.id}</div>
  {/if}

  <div class="comment-text">
    <p>{comment.text}</p>
    <p class="sentiment {comment.sentiment.toLowerCase()}">
      Sentiment: {comment.sentiment}
    </p>
    <p class="score">Score: {comment.score}</p>
  </div>
</div>
