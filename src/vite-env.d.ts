/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly ID_client?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
