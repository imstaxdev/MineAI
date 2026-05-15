# MineIA

MineIA is an open source, offline-first Minecraft launcher built with AI assistance. It focuses on local profiles, low resource usage, simple mod and shader imports, and transparent runtime files.

## Features

- Offline Minecraft profiles with local usernames.
- Deterministic offline UUID generation.
- Local version cache using Mojang version metadata.
- Verified downloads with SHA-1 and file size checks.
- Java detection through `MINEIA_JAVA_HOME`, `JAVA_HOME` or `PATH`.
- Detached Minecraft process on Windows.
- Low-consumption launch defaults for RAM, FPS, render distance and JVM options.
- Per-profile folders for mods, shaders, resource packs, logs and crash reports.
- Import support for `.jar` mods, `.zip` shaders and `.mrpack` modpacks.
- Local file hashing with SHA-256 and BLAKE3.
- Tauri desktop app with SolidJS and Tailwind CSS.

## Screenshots

Add screenshots in `docs/screenshots/` before publishing the repository.

Recommended images:

- main launcher screen
- version selector
- mods and shaders screen
- local profile folder example

## Repository Layout

```text
apps/mineia-launcher/      Desktop app, Tauri backend and SolidJS UI
crates/mineia-auth/        Offline username validation and UUID generation
crates/mineia-core/        Local paths, SQLite profiles and hashing
crates/mineia-runtime/     Java detection, downloads and Minecraft launch
crates/mineia-modpacks/    Mod, shader and modpack import logic
docs/                      Project documentation
scripts/                   Developer commands
tools/                     Internal maintenance tools
```

## Requirements

- Windows 10 or newer
- Rust stable
- Node.js 20 or newer
- npm
- Java 17 or newer for modern Minecraft versions

## Install Dependencies

```powershell
npm.cmd install --prefix apps/mineia-launcher
```

PowerShell can block `npm.ps1` on some systems. Use `npm.cmd` for consistent Windows behavior.

## Run in Development

```powershell
npm.cmd run dev --prefix apps/mineia-launcher
```

## Build the App

```powershell
npm.cmd run tauri --prefix apps/mineia-launcher -- build
```

Artifacts are generated under:

```text
target/release/
target/release/bundle/
```

## Downloading Executables

For public releases, attach generated files from `target/release/bundle/` to the GitHub Release page.

Recommended release names:

- `MineIA_1.0.0_x64-setup.exe`
- `MineIA_1.0.0_x64_en-US.msi`

## Versioning

MineIA should use semantic versioning:

- `v1.0.0`: first stable release
- `v1.1.0`: new features without breaking existing behavior
- `v1.1.1`: bug fixes only
- `v2.0.0`: breaking changes or large architecture changes

Update versions in:

- `Cargo.toml`
- `apps/mineia-launcher/package.json`
- `apps/mineia-launcher/src-tauri/tauri.conf.json`

## Development Commands

```powershell
cargo fmt --all
cargo check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
npm.cmd run build --prefix apps/mineia-launcher
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
node tools/clean.mjs
```

More details are in [docs/TOOLS.md](docs/TOOLS.md).

## Security

MineIA does not include Microsoft login, telemetry, a backend service or remote account storage. It is offline-first by design.

Security notes:

- Do not commit local databases, logs, profiles or downloaded game files.
- Keep `.env` files private.
- Validate archive paths before extracting modpacks.
- Verify Mojang downloads before reuse.
- Treat imported `.jar` mods as third-party executable code.

See [SECURITY.md](SECURITY.md).

## Contributing

1. Fork the repository on GitHub.
2. Clone your fork:

```powershell
git clone https://github.com/your-user/mineia.git
cd mineia
```

3. Create a branch:

```powershell
git checkout -b feature/short-description
```

4. Make focused changes.
5. Run checks:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

6. Commit and push:

```powershell
git add .
git commit -m "Improve launcher UI"
git push origin feature/short-description
```

7. Open a Pull Request.

Pull Requests should include a summary, screenshots for UI changes and the checks that were run.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MineIA is released under the MIT License. See [LICENSE](LICENSE).

## Credits

MineIA is an open source launcher built with AI-assisted development and maintained by its contributors.
