# .me

**Still heavily WIP.**

## Dependencies

- [node](https://nodejs.org/en)
- [rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Entry points

- Main thread: `src/routes/+page.svelte`
- Worker: `src/worker.ts`

The Rust code lives in `src/atlas/`.

## Running

> Rust changes require a manual build and may require a dev server restart.

> Remember to set the log level to verbose on the browser console.

```
pnpm build:atlas
pnpm dev
```

[wgpu](https://wgpu.rs/) only supports [WebGPU](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API) on Chrome. Tested on version `114.0.5735.199`.
