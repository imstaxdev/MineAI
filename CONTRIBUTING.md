# Contributing to MineIA

Thanks for helping improve MineIA.

## Requirements

- Windows 10 or newer
- Rust stable
- Node.js 20 or newer
- npm
- Java 17 or newer for launching modern Minecraft versions

## Setup

```powershell
git clone https://github.com/your-user/mineia.git
cd mineia
npm.cmd install --prefix apps/mineia-launcher
```

## Development

```powershell
npm.cmd run dev --prefix apps/mineia-launcher
```

## Checks Before Pull Request

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

## Pull Request Flow

1. Fork the repository.
2. Create a branch:

```powershell
git checkout -b feature/short-description
```

3. Make focused changes.
4. Run checks.
5. Commit with a clear message.
6. Open a Pull Request with:
   - what changed
   - why it changed
   - screenshots for UI changes
   - test commands run

## Rules

- Keep changes scoped.
- Do not commit generated builds, downloaded Minecraft files, local databases or logs.
- Do not add telemetry, account tokens or secrets.
- Validate user-provided paths and archive extraction paths.
- Prefer small modules over large mixed files.
- Keep UI text clear and non-technical.
