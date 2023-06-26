import init, { initThreadPool, setPanicHook, crunch } from '$atlas/server';

// Pretend we have a server doing this through a ClientProxy.
onmessage = (m) => {
	console.debug('worker received:', m);
	m.ports[0].postMessage(['Ok', ['0']]);
};

await init();
setPanicHook();
await initThreadPool(navigator.hardwareConcurrency);
console.log(`worker::crunch() = ${crunch()}`);
