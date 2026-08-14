# Miracle Ren'Py Launcher

Miracle Ren'Py Launcher (MRL) is planned as a portable Windows desktop launcher for Ren'Py games, with local-first save management and optional Google Drive synchronization.

## Current status

The project is currently in the specification phase. The product and architecture requirements are maintained in [`Cdc.md`](Cdc.md). The planned stack is Rust + Tauri for the desktop/backend layer and Svelte + TypeScript + Vite for the frontend.

## Contributing

Read [`AGENTS.md`](AGENTS.md) before making changes. Never commit credentials or local `.env` files. Once the application scaffold exists, development and test commands will be documented here.

## Development commands

```powershell
npm install                 # install frontend dependencies
npm run dev                 # start the Vite frontend
npm run tauri dev           # run the desktop application
npm run check               # validate Svelte and TypeScript
npm run tauri build         # create Windows executable, MSI and NSIS bundles
```

The GitHub Actions workflow runs the frontend/Rust checks and stores Windows build artifacts for pushes to `main`.
