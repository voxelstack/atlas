import init, { initThreadPool, crunch, greet } from '$atlas/server/atlas_server.js';

await init();
await initThreadPool(navigator.hardwareConcurrency);
console.log(`server: greet().id = ${greet().id}`);
console.log(`server: crunch() = ${crunch()}`);
