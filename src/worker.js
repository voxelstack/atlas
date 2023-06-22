import init, { initThreadPool, crunch } from '../dotatlas/dotatlas.js';

await init();
await initThreadPool(navigator.hardwareConcurrency);

console.log(crunch());
