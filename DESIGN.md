# tor-client (Rust) — Design

A client for a **local Tor daemon**, exposed as a small `TorClient` type that
`1m5-core-rust` wraps as the Tor **protocol service**. A Rust port of the design
in [`tor-client-java`](https://github.com/resolvingarchitecture/tor-client-java),
trimmed to what the router needs today.

## Where it sits

    1m5-core-rust  ──wraps──►  tor_client::TorClient
      onemfive_core::protocol::TorProtocolService (impl Service + Transport)
                                     │
                          SOCKS5 proxy 127.0.0.1:9050   (outbound)
                          control port 127.0.0.1:9051   (readiness probe)
                                     │
                          tor daemon (installed on host)

`1m5-core-rust`'s `RoutingService` discovers the protocol service by name and
pushes a routing-slip hop carrying the destination URL; the adapter calls
`TorClient::send`.

## Components

    LocalTorDetector   probes SOCKS + control ports (TcpStream::connect_timeout)
                       so start() fails fast instead of a confusing error later
    socks              minimal SOCKS5 CONNECT client, no auth (VER=5 greeting,
                       ATYP=3 domain request, parse BND reply)
    http               HTTP/1.1 GET over the SOCKS tunnel, split on CRLFCRLF;
                       http:// only (no TLS)
    TorClient          config, status (AtomicU8), start()/stop()/send()

## Message flow

**Outbound** — `1m5-core-rust` routes an `Envelope` whose `headers["url"]` is a
`.onion` or clearnet `http://` URL. `TorClient::send` opens a SOCKS5 tunnel to
the URL's host through `127.0.0.1:9050`, issues `GET`, and puts the response body
in `envelope.payload` (or an error string in `envelope.headers["error"]`).

**Inbound** — not implemented. `tor-client-java` runs a hidden service via the
control connection; here the control port is only probed. See `TODO.md`.

## Status model

`Status { Connecting, Connected, Disconnected, Error }`, stored as an `AtomicU8`
so it can be read without locking.

- `start()` → `Connecting`, then `Connected` if both ports answer, else logs a
  setup hint and returns to `Disconnected` (returns `false`, cleanly — never
  panics or blocks).
- `stop()` → `Disconnected`.

The `1m5-core-rust` adapter maps this onto its own `NetworkStatus`.

## Rust adaptations vs. the Java client

- No inheritance / `NetworkService` base — `TorClient` is a plain struct; the
  bus lifecycle lives in `1m5-core-rust`'s adapter.
- No `TaskRunner` status poller — status is set at `start()`/`stop()`; a live
  poll would reconnect to the control port.
- Hand-rolled SOCKS5 + HTTP instead of pulling in `reqwest`/`tokio` — keeps the
  crate dependency-light and blocking (the bus stage owns the thread).
- The Tor control protocol client (`TORControlConnection` & friends in Java) is
  not ported yet.

## Not here

- HTTPS (needs `rustls`/`native-tls`).
- Tor control protocol: authentication, event stream, `NEWNYM`, circuit info.
- Hidden service (onion) hosting for inbound envelopes.
- Stream isolation per identity / per destination.
