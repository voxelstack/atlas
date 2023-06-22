<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';

	let canvas: HTMLCanvasElement;

	onMount(() => {
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
