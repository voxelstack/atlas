<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';
	import init, { initOutput, AtlasClient } from '$atlas/client';
	import spawn from '$lib/spawner';

	let surface: HTMLCanvasElement;
	let atlas: AtlasClient;

	onMount(async () => {
		await init();
		initOutput();

		const server = await spawn(Worker);
		atlas = new AtlasClient(server);
		await atlas.listen();
	});
</script>

<canvas bind:this={surface} />
<button on:click={() => atlas.ping()}>inc</button>

<style>
	canvas {
		position: fixed;
		pointer-events: none;

		top: 0;
		left: 0;

		width: 100vw;
		height: 100vh;
	}
</style>
