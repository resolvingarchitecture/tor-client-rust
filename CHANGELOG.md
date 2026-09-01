# Changelog

## 0.2.0 — 2026-08-31

Adds an embedded Tor backend and the `local` / `embedded` / `auto` mode model,
matching `i2p-rust`. Motivated by `1m505` (1M5 on Redox OS), where no system Tor
daemon exists.

- **`Mode { Local, Embedded, Auto }`** — config key `ra.tor.mode`, default
  `auto`. `effective_mode()` probes the local daemon at `start()`.
- **Embedded backend** (`embedded` feature) — `embedded::EmbeddedTor` runs
  [Arti](https://gitlab.torproject.org/tpo/core/arti) (`arti-client` 0.45,
  `rustls` + bundled `static-sqlite`) on a dedicated Tokio runtime; directory +
  guard state persisted under `ra.tor.dataDir`/`tor`. Off by default — heavy.
- **Runtime fallback** (auto only, inline in `send()`, no threads) — `local` →
  `embedded` when the daemon vanishes (re-checked so a bad onion doesn't
  trigger it); `embedded` → `local` when the daemon returns (re-probed at most
  every 30s). Surfaced only as a `Status` transition; the embedded client is
  kept warm across flaps.
- **`http` module** refactored into transport-agnostic helpers (`parse_url`,
  `format_get`, `split_body`) shared by both backends.
- New config key `ra.tor.dataDir`. `Status` variants unchanged, so the
  `onemfive_core::protocol::TorProtocolService` adapter needs no change.
- Tests: mode/backend round-trips, `local` mode without a daemon, and an
  `#[ignore]` live test that bootstraps Arti and fetches over Tor.

## 0.1.0 — 2026-08-31

First Rust implementation, porting the design of `tor-client-java` 1.2.x.

- `LocalTorDetector` — probes SOCKS (`9050`) + control (`9051`) ports so startup
  fails fast when no daemon is running.
- Minimal SOCKS5 CONNECT client (no auth) and HTTP/1.1 GET over the tunnel
  (`http://` only).
- `TorClient` — `from_config` (keys `ra.tor.host` / `ra.tor.socksPort` /
  `ra.tor.controlPort` / `ra.tor.requestTimeoutSecs`), `start` / `stop` / `send`,
  `Status { Connecting, Connected, Disconnected, Error }` as an `AtomicU8`.
- `send` fetches `envelope.headers["url"]` through Tor into `envelope.payload`;
  errors recorded in `envelope.headers["error"]`.
- Consumed by `1m5-core-rust` as `onemfive_core::protocol::TorProtocolService`.

Not yet: HTTPS, the Tor control protocol (probe only), hidden-service hosting.
