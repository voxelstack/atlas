<script lang="ts">
	import { onMount } from 'svelte';
	import Worker from '../worker?worker';

	onMount(() => {
		const handle = new Worker();
		handle.onmessage = (m) => console.log(`worker: ${m.data}`);
		handle.onerror = (e) => console.error(e);

		const message = 'ping';
		console.log(`main: ${message}`);
		handle.postMessage(message);
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
