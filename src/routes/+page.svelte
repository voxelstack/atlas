<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker&inline';
	import init, { initOutput, AtlasClient, Observable } from '$atlas/client';
	import spawn from '$lib/spawner';

	let surface: HTMLCanvasElement;
	let atlas: AtlasClient;

	let count: Observable;
	let unsubscribe: Function | undefined;

	const logger = (value: number) => console.log(`observed: ${value}`);

	onMount(async () => {
		await init();
		initOutput();

		const server = await spawn(Worker);
		atlas = new AtlasClient(server);

		await atlas.ping();
		await atlas.listen();

		await atlas.attach(surface.transferControlToOffscreen());

		count = atlas.observe('ServerEvent :: Count');
		unsubscribe = count.subscribe(logger);
		await atlas.query();
	});
</script>

<canvas bind:this={surface} />

<div class="widget">
	<div class="counter">
		<button on:click={() => atlas.dec()}>-</button>
		<span>
			{$count !== undefined ? String($count).padStart(3, '0') : '···'}
		</span>
		<button on:click={() => atlas.inc()}>+</button>
	</div>

	<button
		on:click={() => {
			unsubscribe?.();
			unsubscribe = undefined;
		}}
		disabled={!unsubscribe}
	>
		detach logger
	</button>
	<button
		on:click={() => {
			unsubscribe = count.subscribe(logger);
		}}
		disabled={!!unsubscribe}
	>
		attach logger
	</button>
</div>

<style lang="scss">
	canvas {
		position: fixed;
		pointer-events: none;

		top: 0;
		left: 0;

		width: 100vw;
		height: 100vh;
	}

	.widget {
		display: inline-flex;
		flex-direction: column;
		row-gap: 8px;
		align-items: stretch;
		padding: 16px;

		.counter {
			display: flex;
			align-items: center;
			font-size: 32px;

			span {
				font-family: monospace;
				width: 3ch;
				padding: 0 1ch;
				text-align: right;
			}

			button {
				width: 32px;
				height: 32px;
				font-size: 16px;
			}
		}

		button {
			height: 32px;
			font-size: 16px;
		}
	}
</style>
