<script>
  import { invoke } from '@tauri-apps/api/tauri'
  import { createEventDispatcher } from 'svelte'
  import './_ProfileForm.css';

  const dispatch = createEventDispatcher()

  let username = ''
  let password = ''
  let confirmPassword = ''
  let apiKey = ''

  async function submitForm() {
    if (password !== confirmPassword) {
      alert('Passwords do not match')
      return
    }

    try {
      await invoke('submit_profile', { username, password, apiKey })
      dispatch('profileCreated')
    } catch (error) {
      alert('Error creating profile: ' + error)
    }
  }
</script>

<div class="profile-form">
  <h2>Create Profile</h2>
  <form on:submit|preventDefault="{submitForm}">
    <div class="form-group">
      <label for="username">Username</label>
      <input type="text" id="username" bind:value="{username}" required />
    </div>
    <div class="form-group">
      <label for="password">Password</label>
      <input type="password" id="password" bind:value="{password}" required />
    </div>
    <div class="form-group">
      <label for="confirmPassword">Confirm Password</label>
      <input type="password" id="confirmPassword" bind:value="{confirmPassword}" required />
    </div>
    <div class="form-group">
      <label for="apiKey">API Key</label>
      <input type="text" id="apiKey" bind:value="{apiKey}" required />
    </div>
    <button type="submit">Create Profile</button>
  </form>
</div>