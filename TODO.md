# tor-client (Rust) — TODO

## P0 — embedded backend (done)
- [x] `Mode { Local, Embedded, Auto }` / `ra.tor.mode` (mirrors `i2p-rust`).
- [x] Embedded Arti backend behind the `embedded` feature (`arti-client`,
      rustls + bundled sqlite); persisted state under `ra.tor.dataDir`.
- [x] Runtime fallback in `auto`: `local`↔`embedded`, inline in `send()`.
- [ ] Readiness: report `Connecting`→`Connected` from Arti's bootstrap events
      instead of blocking `create_bootstrapped` (parallels `i2p-rust` P2).
- [ ] Drop the warm embedded client after a grace period once back on `local`
      (currently kept until `stop()`), if memory matters more than re-bootstrap.
- [ ] Bridges / pluggable transports passthrough for the embedded backend
      (obfs4, Snowflake) — needed for censored networks; `arti` PT maturity TBD.

## P1 — request path
- [ ] HTTPS for both backends (`rustls` at the request layer behind a `tls`
      feature; the embedded backend already links rustls).
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
- [x] `#[ignore]` live test: embedded Arti bootstraps + fetches over Tor.
- [ ] CI matrix: default, `--features embedded`; `cargo clippy -- -D warnings`,
      `cargo fmt --check`.
- [ ] Publish to crates.io once the API settles (currently git dep only).

## Cross-repo
- [ ] Keep `Status` and config keys aligned with `tor-client-java` 1.2.x and the
      `onemfive_core::protocol::TorProtocolService` adapter.
- [ ] `1m5-core-rust`: add a `tor-embedded` feature forwarding
      `tor_client/embedded` (mirrors the existing `i2p-embedded`).
- [ ] `1m505`: confirm `arti-client` builds for `x86_64-unknown-redox`
      (tokio + rustls/ring); assess Arti bridge support.
