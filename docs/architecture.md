# Architecture details

## Rust modules

- `src/main.rs`: process entrypoint, logger, CLI routing.
- `src/cli.rs`: CLI surface (`daemon`, `download`, `status`).
- `src/api/mod.rs`: Axum API handlers.
- `src/network/mod.rs`: HEAD probe (`HttpClient::probe`) for range + size.
- `src/scheduler/mod.rs`: adaptive parallel chunk policy.
- `src/db/mod.rs`: SQLite-backed chunk resume store.
- `src/download/mod.rs`: core engine and multi-thread chunk downloads.

### Key functions

- `DownloadEngine::enqueue`: creates a new job and spawns async task.
- `DownloadEngine::run_download`: selects chunked/single mode.
- `build_chunk_plan`: deterministic range splitting.
- `merge_chunks`: safe chunk assembly via temp file + rename.
- `ResumeStore::save_chunk_state`: idempotent upsert for resume checkpoints.

## Browser extension modules

- `extension/background.js`: command broker and daemon API integration.
- `extension/content.js`: download link interception.
- `extension/popup/*`: manual control panel and status display.
- `extension/options/*`: daemon URL configuration.
