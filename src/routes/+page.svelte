<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';
	import init, { initOutput, AtlasClient } from '$atlas/client';
	import spawn from '$lib/spawner';

	let surface: HTMLCanvasElement;

	onMount(async () => {
		await init();
		initOutput();

		const server = await spawn(Worker);
		const atlas = new AtlasClient(server);
		await atlas.ping();
		await atlas.attach(surface.transferControlToOffscreen());
	});
</script>

<canvas bind:this={surface} />

<style>
	canvas {
		position: fixed;

		top: 0;
		left: 0;

		width: 100vw;
		height: 100vh;
	}
</style>
