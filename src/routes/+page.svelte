<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';
	import init, { greet } from '$atlas/client';

	let canvas: HTMLCanvasElement;

	onMount(async () => {
		await init();
		console.log(`client: greet().id = ${greet().id}`);

		new Worker();
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
