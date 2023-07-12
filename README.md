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

> Rust changes require a manual build and a dev server restart.

> Remember to set the log level to verbose on the browser console.

```
pnpm build:atlas
pnpm dev
```

Tested on Chrome `114.0.5735.199` and Firefox `115.0.2`.
