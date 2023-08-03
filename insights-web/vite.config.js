import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	// optimizeDeps: { exclude: ['nodejs-polars-linux-x64-musl', 'nodejs-polars-linux-x64-gnu'] },
	plugins: [sveltekit()]
});
