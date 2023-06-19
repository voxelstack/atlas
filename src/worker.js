import init, { attach } from '../dotatlas/dotatlas.js';

onmessage = async (message) => {
	await init();

	// TODO Move all glue to Rust.
	const [action, payload] = message.data;
	switch (action) {
		case 'attach':
			attach(payload.canvas);
	}
};
