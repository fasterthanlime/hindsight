# hindsight

[![MIT + Apache 2.0](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-blue)](./LICENSE-MIT)
[![CI](https://github.com/bearcove/hindsight/actions/workflows/ci.yml/badge.svg)](https://github.com/bearcove/hindsight/actions/workflows/ci.yml)
[![experimental](https://img.shields.io/badge/experimental-yes-orange)](#status)
[![do-not-use](https://img.shields.io/badge/do%20not%20use-yet-red)](#status)

> DO NOT USE (YET): hindsight is **experimental** and the api/architecture are in flux.

**Unified observability hub for Bearcove tools.** Distributed tracing + live introspection over **rapace rpc**.

## Status

Active development; expect breaking changes.

- Plan/spec: [`PLAN.md`](./PLAN.md)
- Archived drafts: [`docs/archive/PLAN_v1.md`](./docs/archive/PLAN_v1.md), [`docs/archive/PLAN_v2_picante.md`](./docs/archive/PLAN_v2_picante.md)
- Current plan uses a **single HTTP port** with **Upgrade** to select transport (rapace vs websocket) + `GET /` for the ui bootstrap page.

## what is hindsight?

hindsight will be a **trace collection server + ui** that:
- will collect W3C Trace Context spans from apps (via rapace rpc transports),
- will discover app capabilities at runtime (service introspection),
- and will adapt its ui dynamically (generic trace views + framework-specific views).

the goal is one place to debug:
- [`rapace`](https://rapace.bearcove.eu/) (rpc): topology/transport/active calls (see: [architecture](https://rapace.bearcove.eu/guide/architecture/), [cells](https://rapace.bearcove.eu/guide/cells/))
- [`picante`](https://picante.bearcove.eu/) (incremental): query graphs + cache hit/miss/validation (see: [architecture](https://picante.bearcove.eu/guide/architecture/), [guide](https://picante.bearcove.eu/guide/))
- [`dodeca`](https://dodeca.bearcove.eu/) (build): build progress/pages/template stats (see: [features](https://dodeca.bearcove.eu/guide/features/), [query reference](https://dodeca.bearcove.eu/internals/queries/), [template engine](https://dodeca.bearcove.eu/internals/templates/), [debugging templates](https://dodeca.bearcove.eu/guide/debugging-templates/), [plugins](https://dodeca.bearcove.eu/internals/plugins/))

## Philosophy

**pure rapace.** one protocol end-to-end. http exists only to serve a tiny static page that loads the browser ui; trace data flows over rapace.

**extensible by discovery.** apps expose introspection services; hindsight calls `ServiceIntrospection.list_services()` and enables views accordingly.

**ephemeral by default.** in-memory storage with ttl (persistence/export are planned).

**avoid self-tracing loops.** hindsight’s own rapace sessions are untraced; tracing in apps is explicit opt-in.

## integration with bearcove projects

hindsight aims to provide **generic tracing** plus **framework-specific views** when the app exposes introspection services.

### rapace (rpc framework)

```rust
use rapace::RpcSession;
use hindsight::Tracer;

// Create a tracer that exports spans to hindsight.
// (Transport setup omitted here for brevity.)
let tracer = /* ... */;

let session = RpcSession::new(transport)
    .with_tracer(tracer); // Automatic RPC span tracking!

// All RPC calls now appear in hindsight
session.call(method_id, payload).await?;
```

### picante (incremental computation)

```rust
use picante::Runtime;
use hindsight::Tracer;

let tracer = /* ... */;
let runtime = Runtime::new()
    .with_tracer(tracer); // Planned: emit spans with picante.* attributes

// Query execution shows up as spans
let result = db.my_query.get(&db, key).await?;
```

### dodeca (static site generator)

```rust
use hindsight::Tracer;

let tracer = /* ... */;

// See your entire build pipeline traced:
// File change → Markdown parse → Image optimization → Template render
```

## Architecture

```
NOTE: diagram is aspirational; current code does not implement the full hub/discovery/ui.

Apps (native / WASM)                     hindsight (hub)
┌─────────────────────────┐              ┌──────────────────────────┐
│ App emits spans          │──rapace RPC─▶│ hindsightservice         │
│ + exposes introspection  │              │ - ingest_spans           │
│ services (optional)      │◀─rapace RPC──│ - list/get/stream traces │
└─────────────────────────┘              │                          │
                                         │ UI adapts based on:      │
                                         │ - ServiceIntrospection   │
                                         │ - PicanteIntrospection   │
                                         │ - RapaceIntrospection    │
                                         │ - DodecaIntrospection    │
                                         └──────────────────────────┘
```

## Workspace Structure

```
crates/
├── hindsight/          # Client library (emit/export spans)
├── hindsight-server/   # Server binary (`hindsight`)
├── hindsight-tui/      # TUI client (planned; currently a stub)
└── hindsight-protocol/ # Shared protocol types + RPC service trait
```

## Features

- ✅ **W3C Trace Context** (`traceparent`/`tracestate`)
- ✅ **Pure rapace rpc ingestion** (tcp + websocket transport)
- ✅ **Ephemeral in-memory store** (TTL)
- 🚧 **Service discovery driven ui** (planned: dynamic tabs per app capabilities; not implemented yet)
- 🚧 **Framework-specific views** (planned: picante/rapace/dodeca via introspection; not implemented yet)
- 🚧 **Persistence / sampling / export** (planned; not implemented yet)

## Links

- W3C Trace Context: https://www.w3.org/TR/trace-context/
- OpenTelemetry: https://opentelemetry.io/
- HTTP Upgrade (`101 Switching Protocols`): https://developer.mozilla.org/en-US/docs/Web/HTTP/Protocol_upgrade_mechanism
- WebSocket protocol (RFC 6455): https://www.rfc-editor.org/rfc/rfc6455
- Workspace: `Cargo.toml`
- Workspace crates: `crates/`
- Protocol types: `crates/hindsight-protocol/src/trace_context.rs`, `crates/hindsight-protocol/src/span.rs`, `crates/hindsight-protocol/src/service.rs`
- Server entrypoint (router + upgrade handlers): `crates/hindsight-server/src/main.rs`
- In-memory store: `crates/hindsight-server/src/storage.rs`

## Example: Distributed Trace Across Systems

```rust
// mockup: this is the kind of cross-tool trace hindsight aims to show (not working yet)

// In your web server
let span = tracer.span("handle_request").start();

// Make an RPC call (trace context auto-propagated)
let result = rpc_client.call(method, payload).await?;

// That RPC triggers a picante query in another process
// All show up in ONE trace:
//
// handle_request (50ms)
//   ├─ RPC: calculate (40ms)
//   │   ├─ picante: load_data (5ms, cache hit)
//   │   └─ picante: compute (35ms, recomputed)
//   └─ format_response (10ms)

span.end();
```

## Development

**Build:**
```bash
cargo build --workspace
```

**Run tests:**
```bash
cargo test --workspace
```

**Run the server locally:**
```bash
cargo run -p hindsight-server -- serve
```

## Contributing

See `PLAN.md` for the detailed design doc/spec.

Contributions welcome! Please open issues and PRs.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
