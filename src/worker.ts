import init, { initThreadPool, initOutput, AtlasServer } from '$atlas/server';

await init();
initOutput();
await initThreadPool(navigator.hardwareConcurrency);

postMessage('ready');

const atlas = new AtlasServer(self);
await atlas.listen();
