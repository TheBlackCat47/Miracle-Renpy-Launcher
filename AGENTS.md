# Repository Guidelines

## Project Structure & Module Organization

This repository is currently in the specification phase. `Cdc.md` is the authoritative product and architecture brief for Miracle Ren'Py Launcher (MRL), a Windows-first portable Tauri application with a Rust backend and Svelte/TypeScript frontend. No source, test, or asset directories exist yet. When implementation begins, keep domain boundaries explicit: place Rust business logic in focused modules/crates (for example, synchronization, Ren'Py detection, launcher, storage, and cloud providers), keep Tauri commands as a thin bridge, and keep the frontend focused on presentation. Organize tests and static assets alongside their owning component or in clearly named top-level directories.

## Build, Test, and Development Commands

No build tooling or package manifests are present yet, so there are no repository-specific commands to run. Once scaffolding is added, document the canonical commands here and in the project README. The planned stack suggests commands such as `cargo test` for Rust tests, `npm run check` for Svelte/TypeScript validation, and `npm run tauri dev` for local desktop development; use the exact scripts declared by the repository rather than inventing alternatives.

## Coding Style & Naming Conventions

Use standard Rust formatting with `cargo fmt` and lint with `cargo clippy --all-targets --all-features`. Prefer clear, domain-oriented names and small modules. Use `snake_case` for Rust files/functions, `PascalCase` for Svelte components and Rust types, and `camelCase` for TypeScript variables/functions. Keep synchronization, credentials, filesystem access, and process management out of UI components.

## Testing Guidelines

Add unit tests for domain logic and integration tests for filesystem, SQLite, process-launch, OAuth, and synchronization behavior. Include regression coverage for atomic writes, conflict preservation, migrations, offline queues, and failure scenarios. Tests should be deterministic and must never use real user credentials or an uncontrolled Google Drive account.

## Security & Configuration Tips

Do not commit `.env` files, OAuth client secrets, refresh tokens, save data, or generated credentials. Use documented placeholders for local configuration and the least-privileged Google Drive scopes. Treat user save files as valuable data: preserve originals during reconciliation and prefer recoverable backups when ambiguity exists.

## Commit & Pull Request Guidelines

There is no Git history yet, so no existing commit convention can be inferred. Use short imperative subjects (for example, `Add atomic save writer`) and keep commits focused. Pull requests should explain the behavior change, identify relevant sections of `Cdc.md`, include test commands/results, and attach screenshots or recordings for UI changes.
