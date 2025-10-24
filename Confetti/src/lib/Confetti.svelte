<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  let canvas: HTMLCanvasElement | null = null;
  export let durationSeconds = 3; // how long to spawn confetti (in seconds)
  const dispatch = createEventDispatcher();

  type Particle = {
    x: number;
    y: number;
    vx: number;
    vy: number;
    rot: number;
    vrot: number;
    life: number; // remaining seconds
    ttl: number; // total lifespan seconds
    shape: number;
    color: string;
    size: number;
  };

  // SNFG-like palette (simple subset)
  const SNFG = [
    '#ffd400', // yellow
    '#00a651', // green
    '#0072bc', // blue
    '#a54399', // magenta
    '#ed1c24', // red
  ];

  const MAX = 300;
  const particles: Particle[] = new Array(MAX);
  let alive = 0;

  const gravity = 900; // px/s^2
  const spawnPerSecond = 120;
  let spawnAccumulator = 0;

  let running = false;
  let ctx: CanvasRenderingContext2D | null = null;
  let last = 0;
  let rafId: number | null = null;

  // spawn control
  let spawnEnabled = true;
  let spawnElapsed = 0;

  function rand(a = 0, b = 1) {
    return a + Math.random() * (b - a);
  }

  function makeParticle(width: number): Particle {
    const p: Particle = {
      x: rand(0, width),
      y: -10,
      vx: rand(-80, 80),
      vy: rand(30, 160),
      rot: rand(0, Math.PI * 2),
      vrot: rand(-6, 6),
      life: rand(1.2, 2.6),
      ttl: 0,
      shape: Math.floor(rand(0, 4)),
      color: SNFG[Math.floor(rand(0, SNFG.length))],
      size: Math.floor(rand(6, 18)),
    };
    p.ttl = p.life;
    return p;
  }

  function spawn(width: number) {
    if (alive < MAX) {
      particles[alive] = makeParticle(width);
      alive++;
      return;
    }
    const idx = Math.floor(Math.random() * MAX);
    particles[idx] = makeParticle(width);
  }

  function drawShape(p: Particle) {
    if (!ctx) return;
    const s = p.size;
    ctx.fillStyle = p.color;
    switch (p.shape) {
      case 0:
        ctx.beginPath();
        ctx.arc(0, 0, s / 2, 0, Math.PI * 2);
        ctx.fill();
        break;
      case 1:
        ctx.fillRect(-s / 2, -s / 2, s, s);
        break;
      case 2:
        ctx.beginPath();
        ctx.moveTo(0, -s / 2);
        ctx.lineTo(s / 2, 0);
        ctx.lineTo(0, s / 2);
        ctx.lineTo(-s / 2, 0);
        ctx.closePath();
        ctx.fill();
        break;
      case 3:
        ctx.beginPath();
        ctx.moveTo(0, -s / 2);
        ctx.lineTo(s / 2, s / 2);
        ctx.lineTo(-s / 2, s / 2);
        ctx.closePath();
        ctx.fill();
        break;
    }
  }

  function step(ts: number) {
    if (!running || !ctx || !canvas) return;
    const dt = Math.min(0.05, (ts - last) / 1000);
    last = ts;

    const w = canvas.width;
    const h = canvas.height;

    // spawn timing: only while spawnEnabled
    if (spawnEnabled) {
      spawnElapsed += dt;
      if (spawnElapsed >= durationSeconds) spawnEnabled = false;
      spawnAccumulator += spawnPerSecond * dt;
    }

    const toSpawn = Math.floor(spawnAccumulator);
    spawnAccumulator -= toSpawn;
    for (let i = 0; i < toSpawn; i++) spawn(w);

    ctx.clearRect(0, 0, w, h);
    for (let i = 0; i < alive; i++) {
      const p = particles[i];
      if (!p) continue;
      p.life -= dt;
      if (p.life <= 0 || p.y - p.size > h + 50) {
        particles[i] = particles[alive - 1];
        particles[alive - 1] = undefined as unknown as Particle;
        alive--;
        i--;
        continue;
      }
      p.vy += gravity * dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.rot += p.vrot * dt;

      ctx.save();
      ctx.translate(p.x, p.y);
      ctx.rotate(p.rot);
      drawShape(p);
      ctx.restore();
    }

    // stop the loop when we're no longer spawning and everything died out
    if (!spawnEnabled && alive === 0) {
      running = false;
      if (rafId !== null) cancelAnimationFrame(rafId);
      rafId = null;
      // notify parent that this confetti run finished so it can be remounted/retriggered
      try { dispatch('finished'); } catch {}
      return;
    }

    rafId = requestAnimationFrame(step);
  }

  function resizeCanvas() {
    if (!canvas) return;
    const dpr = Math.max(1, window.devicePixelRatio || 1);
    const w = Math.round(canvas.clientWidth * dpr);
    const h = Math.round(canvas.clientHeight * dpr);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
      if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
  }

  onMount(() => {
    if (!canvas) return;
    ctx = canvas.getContext('2d', { alpha: true });
    running = true;
    last = performance.now();
    resizeCanvas();
    // start loop
    rafId = requestAnimationFrame(step);
    const onResize = () => resizeCanvas();
    window.addEventListener('resize', onResize);
    return () => {
      running = false;
      if (rafId !== null) cancelAnimationFrame(rafId);
      window.removeEventListener('resize', onResize);
    };
  });

  onDestroy(() => {
    running = false;
    if (rafId !== null) cancelAnimationFrame(rafId);
  });
  
  export let active = true;
</script>

<canvas bind:this={canvas} class="confetti-canvas" aria-hidden="true"></canvas>

<style>
.confetti-canvas {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
  z-index: 99999;
  mix-blend-mode: normal;
}
</style>