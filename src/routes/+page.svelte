<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';
	import initWasm, { setPanicHook, AtlasClient, ServerMessage } from '$atlas/client';

	let surface: HTMLCanvasElement;

	onMount(async () => {
		await initWasm();
		setPanicHook();

		const atlas = new AtlasClient(new Worker());
		try {
			console.log('main->worker:', 'PING');
			const res = (await atlas.ping()) as ServerMessage;
			switch (res) {
				case ServerMessage.Pong:
					console.log('main<-worker:', 'PONG');
					break;
			}
		} catch (e) {
			console.error(e);
		}
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
