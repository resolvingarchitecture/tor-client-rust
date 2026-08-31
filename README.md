# tor-client (Rust)

A client for a **local Tor daemon** — it does not run Tor. Outbound requests go
through Tor's SOCKS5 proxy (`127.0.0.1:9050`); the control port
(`127.0.0.1:9051`) is probed for readiness so startup fails fast with a clear
message. A Rust port of
[`tor-client-java`](https://github.com/resolvingarchitecture/tor-client-java).

Used as the Tor **protocol service** for
[`1m5-core-rust`](https://github.com/1m5/1m5-core-rust) via
`onemfive_core::protocol::TorProtocolService`.

## Why local-only

Tor is a C daemon. Unlike I2P — a pure-Rust router that
[`i2p-rust`](https://github.com/resolvingarchitecture/i2p-rust) embeds via
`emissary` — an embedded Tor binary can't be kept updated in-process, so this
client only ever uses a Tor instance already installed and running on the host.

## Tor daemon setup

Install Tor (`apt install tor`, `brew install tor`, …) and make sure
`/etc/tor/torrc` (or `~/.torrc`) has:

```
SocksPort 9050
ControlPort 9051
CookieAuthentication 0
```

Then `systemctl start tor` (or `tor -f ~/.torrc`). Check:
`curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip`.

## Use

```rust
use std::collections::HashMap;
use tor_client::{Status, TorClient};

let client = TorClient::from_config(&HashMap::new());
if client.start() {                       // false (cleanly) if no daemon
    assert_eq!(client.status(), Status::Connected);
    let mut env = /* seda_bus::Envelope */;
    env.headers.insert("url".into(), "http://example.onion/".into());
    client.send(&mut env);                // body -> env.payload, errors -> env.headers["error"]
}
```

### Config keys

| key | default | meaning |
|-----|---------|---------|
| `ra.tor.host` | `127.0.0.1` | daemon host |
| `ra.tor.socksPort` | `9050` | SOCKS5 proxy port |
| `ra.tor.controlPort` | `9051` | control port (probed only) |
| `ra.tor.requestTimeoutSecs` | `60` | per-request timeout |

## Build

```
cargo test
cargo clippy --all-targets
```

## Status

Early. HTTP over SOCKS5 works; HTTPS needs a TLS crate (see `TODO.md`). The
control port is only probed, not spoken — no event stream or hidden-service
management yet. See `DESIGN.md` and `TODO.md`.
