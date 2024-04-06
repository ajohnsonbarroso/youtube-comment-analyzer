<script>
  import { invoke } from '@tauri-apps/api/tauri'
  import { createEventDispatcher } from 'svelte'
  import './_LoginForm.css';

  const dispatch = createEventDispatcher()

  let username = ''
  let password = ''

  async function login() {
    try {
      const loggedIn = await invoke('login', { username, password })
      dispatch('loggedIn', { username, loggedIn })
      if (!loggedIn) {
        alert('Invalid username or password')
      }
    } catch (error) {
      alert('Error logging in: ' + error)
    }
  }
</script>

<div class="login-form">
  <h2>Login</h2>
  <form on:submit|preventDefault="{login}">
    <div class="form-group">
      <label for="username">Username</label>
      <input type="text" id="username" bind:value="{username}" required />
    </div>
    <div class="form-group">
      <label for="password">Password</label>
      <input type="password" id="password" bind:value="{password}" required />
    </div>
    <button type="submit">Login</button>
  </form>
</div>
