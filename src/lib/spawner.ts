export default function spawn(Worker: new () => Worker): Promise<Worker> {
	return new Promise((resolve, _reject) => {
		const worker = new Worker();

		worker.onmessage = (message) => {
			if (message.data === 'ready') {
				worker.onmessage = null;
				resolve(worker);
			} else {
				console.warn('Worker message posted before initialization.', message);
			}
		};
	});
}
