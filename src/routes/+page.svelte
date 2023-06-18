<script lang="ts">
	import { greet } from 'dotatlas';
	import { onMount } from 'svelte';

	onMount(() => {
		greet();
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
