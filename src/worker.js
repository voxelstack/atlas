import init, { get } from '../dotatlas/dotatlas.js';

onmessage = async () => {
	await init();
	postMessage(get());
};
