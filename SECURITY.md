# Security Policy

MineIA is an offline-first launcher. It does not include Microsoft authentication, a backend service, telemetry or remote account storage.

## Supported Scope

Security reports should focus on:

- unsafe file extraction or path traversal
- unsafe process execution
- insecure downloads or missing integrity checks
- sensitive local files committed to the repository
- dependency or build-chain issues

## Reporting

Open a private security advisory on GitHub when available. If advisories are not enabled, open an issue with a minimal reproduction and avoid posting secrets, tokens or private paths.

## Local Data

Runtime data is stored in the local MineIA data directory. Do not commit local profiles, databases, downloaded game files, logs or build artifacts.

## Download Integrity

Minecraft runtime downloads are resolved from Mojang metadata and reused only after size and SHA-1 checks pass. Imported mods and shaders are hashed with SHA-256 and BLAKE3 for local verification.

## Execution Model

MineIA launches Java as a detached process with a local profile directory. User-provided paths are only accepted for supported file import operations and Java path configuration.
