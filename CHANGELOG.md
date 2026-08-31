# Changelog

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
