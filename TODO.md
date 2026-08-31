# tor-client (Rust) — TODO

## P1 — request path
- [ ] HTTPS through the SOCKS tunnel (`rustls` behind a `tls` feature).
- [ ] Follow redirects; surface status code + headers on the `Envelope`.
- [ ] Reuse the SOCKS connection / a small pool instead of one per request.
- [ ] Configurable `User-Agent`; strip identifying headers by default.

## P2 — Tor control protocol
- [ ] Port `TORControlConnection` / `TORControlCommands` from `tor-client-java`
      (authenticate with `CookieAuthentication 0` or a control password).
- [ ] Async event stream (`SETEVENTS`) → map `CIRC` / `STATUS_CLIENT` onto
      `Status`; live readiness instead of a one-shot probe.
- [ ] `NEWNYM` (new circuit) on demand and per ManCon escalation.

## P3 — inbound / hidden service
- [ ] Create or load an onion service key, `ADD_ONION` via the control port.
- [ ] Accept connections on the HS target port, turn requests into `Envelope`s
      and hand them to the bus (mirrors `tor-client-java`'s HS handler).

## P4 — privacy hardening
- [ ] Stream isolation: distinct SOCKS credentials per identity / destination.
- [ ] Assert the daemon's `SocksPort` has no `PreferSOCKSNoAuth` surprises.
- [ ] Optional bridge / pluggable-transport config passthrough.

## Testing / ops
- [ ] Integration test behind a `live` feature that uses a real local Tor.
- [ ] CI: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [ ] Publish to crates.io once the API settles (currently git dep only).

## Cross-repo
- [ ] Keep `Status` and config keys aligned with `tor-client-java` 1.2.x and the
      `onemfive_core::protocol::TorProtocolService` adapter.
