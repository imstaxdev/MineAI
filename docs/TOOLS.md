# Internal Tools

MineIA keeps development commands small and explicit.

## Setup

```powershell
npm.cmd install --prefix apps/mineia-launcher
```

## Development

```powershell
npm.cmd run dev --prefix apps/mineia-launcher
```

## Checks

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Runs Rust formatting, Clippy, Rust tests, TypeScript checks and the Vite build.

## Release Build

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

Release artifacts are generated under `target/release` and `target/release/bundle`.

## Cleanup

```powershell
node tools/clean.mjs
```

Removes generated build output and local logs. It refuses to delete paths outside the repository root.
