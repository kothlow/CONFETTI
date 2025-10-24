<script lang="ts">
  import { appWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount, onDestroy, tick } from 'svelte';
  import Confetti from '../lib/Confetti.svelte';
  import { register, unregisterAll } from '@tauri-apps/api/globalShortcut';

  let pressed = false; // represents fullscreen state
  const pressed_keys = new Set<string>();
  let send_confetti = false;

  // toggle fullscreen reliably and update UI state
  async function toggleFullscreen() {
    let isFs = false;
    try {
      isFs = await appWindow.isFullscreen();
    } catch (e) {
      console.error('isFullscreen failed (allowlist/config?):', e);
    }
    const target = !isFs;
    try {
      await appWindow.setFullscreen(target);
      // update pressed to reflect actual target state
      pressed = target;
    } catch (err) {
      console.error('setFullscreen failed (allowlist/config?):', err);
      // keep UI consistent with actual state
      try {
        pressed = await appWindow.isFullscreen();
      } catch {
        pressed = false;
      }
      // bail out early if fullscreen couldn't be set
      return;
    }
    // set body background based on new state
    document.body.style.background = target ? 'transparent' : 'white';
    if (target) {
      // request native transparent / click-through when entering fullscreen
      try {
        await invoke('set_transparent_clickthrough');
      } catch (e) {
        console.error('backend call failed:', e);
      }
    } else {
      // leaving fullscreen: stop confetti and (optionally) restore native flags
      send_confetti = false;
      try {
        await invoke('clear_transparent_clickthrough');
      } catch {
        
        /* no-op if not implemented */
      }
    }
  }

  async function unToggleFullscreen() {
    // convenience: ensure we leave fullscreen and update UI
    const isFs = await appWindow.isFullscreen();
    if (isFs) {
      await appWindow.setFullscreen(false);
      pressed = false;
      document.body.style.background = 'white';
      send_confetti = false;
      try {
        await invoke('clear_transparent_clickthrough');
      } catch {
        /* no-op */
      }
    }
  }

  // kept same semantics but robust key checks
  async function handleKeyPress(event: KeyboardEvent) {
    if (typeof event.key === 'string' && event.key.toLowerCase() === 'enter') {
      console.log("Enter key pressed");
      await toggleFullscreen();
    }
  }

  async function handleEscKeyPress(event: KeyboardEvent) {
    if (typeof event.key === 'string' && event.key.toLowerCase() === 'escape') {
      await unToggleFullscreen();
    }
  }

  async function handleConfettiKeyPress(event: KeyboardEvent) {
    if (
      pressed_keys.has('Control') &&
      pressed_keys.has('Shift') &&
      typeof event.key === 'string' &&
      event.key.toLowerCase() === 'c' &&
      pressed === true
    ) {
      console.log("Ctrl+Shift+C pressed - starting confetti");
      // If already running, remount to restart; otherwise mount normally
      if (send_confetti) {
        send_confetti = false;
        await tick();
      }
      send_confetti = true;
       // optional: notify backend (you had start_confetti invoke, keep if needed)
       try {
         await invoke('start_confetti');
       } catch (e) {
         console.error('backend call failed:', e);
       }
       // automatically stop confetti after 3s in the Confetti component (prop)
     }
   }

  // single window-level listeners; add/remove on mount/unmount
  function onKeyDown(e: KeyboardEvent) {
    // keep pressed_keys set for modifier tracking
    pressed_keys.add(e.key);
    handleKeyPress(e);
    handleConfettiKeyPress(e);
    handleEscKeyPress(e);
  }
  function onKeyUp(e: KeyboardEvent) {
    pressed_keys.delete(e.key);
  }

  onMount(() => {
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    // ensure initial background is white
    document.body.style.background = 'white';
    // register global hotkey (CTRL+SHIFT+C). format is platform-dependent but this works cross-platform.
    register('CTRL+SHIFT+C', () => {
      // only trigger confetti while fullscreen
      if (pressed) {
        // restart confetti run
        if (send_confetti) {
          send_confetti = false;
          // small tick delay not shown here — your existing logic already handles remount via tick()
        }
        send_confetti = true;
      }
    }).catch((e) => console.error('globalShortcut register failed', e));

    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
      unregisterAll().catch(() => {});
    };
  });
</script>

<main class="container" class:fullscreen={pressed}>
  <div class="message" role="region" aria-live="polite">
    {#if !pressed}
      <p>Press Enter to start confetti generator.</p>
    {/if}
  </div>
  {#if send_confetti}
    <!-- allow parent to be notified when a run finishes so it can be retriggered later -->
    <Confetti durationSeconds={3} on:finished={() => (send_confetti = false)} />
  {/if}
</main>

<style>
:global(html, body) {
  margin: 0;
  height: 100%;
  overflow: hidden; /* remove scrollbar */
}

/* keep the app white by default; when fullscreen (pressed) the container becomes transparent */
.container {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  color: black;
  background: white;
}
.container.fullscreen {
  background: transparent;
}
.message p {
  font-size: 1.1rem;
  user-select: none;
}
</style>