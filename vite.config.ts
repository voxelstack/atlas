import { sveltekit } from '@sveltejs/kit/vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [
		sveltekit(),
		wasm(),
		topLevelAwait(),
		{
			name: 'configure-response-headers',
			configureServer: (server) => {
				server.middlewares.use((_req, res, next) => {
					// SharedArrayBuffer requires a secure context.
					res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
					res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');

					next();
				});
			}
		},
		{
			name: 'reload-trigger',
			handleHotUpdate: ({ file, server }) => {
				if (file.endsWith('.reload')) {
					server.ws.send({
						type: 'full-reload',
						path: '*'
					});
				}
			}
		}
	],
	server: {
		fs: {
			allow: ['./atlas']
		},
		watch: {
			ignored: [
				// Ignore wasm build result. build:atlas touches the reload trigger.
				path.resolve(__dirname, './atlas/**'),
				// Ignore Rust sources.
				path.resolve(__dirname, './src/atlas/**'),
				// Ignore Rust build result.
				path.resolve(__dirname, './target/**')
			]
		}
	}
});
