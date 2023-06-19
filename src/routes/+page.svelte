<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';

	let canvas: HTMLCanvasElement;

	onMount(() => {
		const handle = new Worker();
		handle.onmessage = (m) => console.log(`worker: ${m.data}`);
		handle.onerror = (e) => console.error(e);

		const offscreen = canvas.transferControlToOffscreen();
		handle.postMessage(['attach', { canvas: offscreen }], [offscreen]);
	});
</script>

<canvas bind:this={canvas} />

<style>
	canvas {
		position: fixed;

		top: 0;
		left: 0;

		width: 100vw;
		height: 100vh;
	}
</style>
