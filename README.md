# NetDout

NetDout is a full open-source multi-threaded download manager scaffold with:

- **Rust daemon/core** for adaptive chunked downloading, resume state, and local API.
- **Browser extension scaffold** (Chrome + Firefox) for link interception and daemon control.

## High-level architecture

1. **Core daemon (`src/`)**
   - `network`: probes HTTP metadata and range support.
   - `download`: chunk planning, concurrent chunk fetch, resume-safe writes, merge.
   - `scheduler`: adaptive thread/chunk parallelism strategy.
   - `db`: SQLite state for resume metadata.
   - `api`: local HTTP API for extension and other clients.
   - `cli`: manual testing and automation entrypoints.

2. **Browser extension (`extension/`)**
   - `background.js`: listens for intercepted downloads and calls local daemon API.
   - `content.js`: extracts candidate file URLs from page interactions.
   - `popup/`: compact UI to submit URL + output path and poll status.
   - `options/`: daemon URL configuration.

3. **Data flow**
   - User clicks download in browser.
   - Extension sends `{url, output}` to daemon `POST /downloads`.
   - Daemon probes server, plans chunks, downloads with async workers.
   - Progress is tracked in memory + SQLite resume metadata.
   - Extension polls `GET /downloads/:id` and renders speed/ETA/progress.

## Step-by-step scaffold map

### 1) Rust daemon and library setup
- `cargo run -- daemon` starts local API server.
- `cargo run -- download <URL> <OUTPUT>` tests direct CLI integration.

### 2) Multi-thread chunking + resume
- `build_chunk_plan` splits remote file ranges.
- Each chunk writes to `.<filename>.chunks/chunk-{i}.part`.
- Chunk progress and completion persist in SQLite (`chunk_state`).
- `merge_chunks` assembles final file atomically through temp output.

### 3) API integration
- `POST /downloads` queues an async job.
- `GET /downloads/:id` returns state (`queued|running|completed|failed`).

### 4) Browser extension bridge
- Background scripts send API requests to `http://127.0.0.1:8472`.
- Popup provides manual URL submission + status checks.

## Run and test

```bash
cargo fmt
cargo test
cargo run -- daemon
```

With daemon running, test an API call:

```bash
curl -X POST http://127.0.0.1:8472/downloads \
  -H 'content-type: application/json' \
  -d '{"url":"https://example.com/file.zip","output":"./file.zip"}'
```

## Notes for production hardening

- Add checksum validation and persistent job queue table.
- Add pause/resume/cancel commands and bandwidth shaping.
- Move to websocket push updates for lower polling overhead.
- Add signed native messaging host for tighter browser integration.
