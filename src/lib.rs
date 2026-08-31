//! # tor-client (Rust)
//!
//! A client for a **local Tor daemon** - it does not run Tor. Requests go out
//! through Tor's SOCKS proxy (`127.0.0.1:9050`); the control port
//! (`127.0.0.1:9051`) is probed for readiness. A Rust port of the design in
//! [`tor-client-java`](https://github.com/resolvingarchitecture/tor-client-java).
//!
//! Used as the Tor **protocol service** for `1m5-core-rust` (via
//! `onemfive_core::protocol::TorProtocolService`).
//!
//! ```no_run
//! use tor_client::TorClient;
//! use std::collections::HashMap;
//!
//! let client = TorClient::from_config(&HashMap::new());
//! if client.start() {
//!     let mut env = seda_bus_envelope();  // env.headers["url"] = "http://<onion>/"
//!     client.send(&mut env);
//! }
//! # fn seda_bus_envelope() -> tor_client::Envelope { unimplemented!() }
//! ```

mod detector;
mod http;
mod socks;

pub use detector::{LocalTorDetector, DEFAULT_CONTROL_PORT, DEFAULT_HOST, DEFAULT_SOCKS_PORT};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use log::{info, warn};

/// Re-exported so callers need one crate. `1m5-core-rust` provides its own.
pub use seda_bus::Envelope;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

fn status_from_u8(v: u8) -> Status {
    match v {
        1 => Status::Connecting,
        2 => Status::Connected,
        3 => Status::Error,
        _ => Status::Disconnected,
    }
}
fn status_to_u8(s: Status) -> u8 {
    match s {
        Status::Disconnected => 0,
        Status::Connecting => 1,
        Status::Connected => 2,
        Status::Error => 3,
    }
}

/// A client for a local Tor daemon.
pub struct TorClient {
    detector: LocalTorDetector,
    request_timeout: Duration,
    status: AtomicU8,
}

impl TorClient {
    pub fn new() -> TorClient {
        TorClient {
            detector: LocalTorDetector::default(),
            request_timeout: Duration::from_secs(60),
            status: AtomicU8::new(status_to_u8(Status::Disconnected)),
        }
    }

    /// Config keys: `ra.tor.host`, `ra.tor.socksPort`, `ra.tor.controlPort`,
    /// `ra.tor.requestTimeoutSecs`.
    pub fn from_config(cfg: &HashMap<String, String>) -> TorClient {
        let mut c = TorClient::new();
        if let Some(h) = cfg.get("ra.tor.host") {
            c.detector.host = h.clone();
        }
        if let Some(p) = cfg.get("ra.tor.socksPort").and_then(|s| s.parse().ok()) {
            c.detector.socks_port = p;
        }
        if let Some(p) = cfg.get("ra.tor.controlPort").and_then(|s| s.parse().ok()) {
            c.detector.control_port = p;
        }
        if let Some(t) = cfg
            .get("ra.tor.requestTimeoutSecs")
            .and_then(|s| s.parse().ok())
        {
            c.request_timeout = Duration::from_secs(t);
        }
        c
    }

    pub fn detector(&self) -> &LocalTorDetector {
        &self.detector
    }

    pub fn status(&self) -> Status {
        status_from_u8(self.status.load(Ordering::Acquire))
    }

    fn set_status(&self, s: Status) {
        self.status.store(status_to_u8(s), Ordering::Release);
    }

    /// Verify a local Tor daemon is reachable. Returns false (cleanly) if not.
    pub fn start(&self) -> bool {
        self.set_status(Status::Connecting);
        if !self.detector.is_local_tor_running() {
            warn!(
                "No local Tor daemon on {} (SOCKS {} reachable={}, control {} reachable={}). \
                 Install and run Tor with 'ControlPort 9051' - see README.md.",
                self.detector.host,
                self.detector.socks_port,
                self.detector.is_socks_reachable(),
                self.detector.control_port,
                self.detector.is_control_reachable(),
            );
            self.set_status(Status::Disconnected);
            return false;
        }
        info!(
            "Local Tor daemon reachable (SOCKS {}, control {}).",
            self.detector.socks_port, self.detector.control_port
        );
        self.set_status(Status::Connected);
        true
    }

    pub fn stop(&self) -> bool {
        self.set_status(Status::Disconnected);
        true
    }

    /// Fetch `envelope.headers["url"]` through Tor and put the response body in
    /// `envelope.payload`. HTTP only for now (HTTPS needs a TLS crate - see
    /// TODO.md). On error, records `envelope.headers["error"]`.
    pub fn send(&self, envelope: &mut Envelope) -> bool {
        let Some(url) = envelope.headers.get("url").cloned() else {
            envelope
                .headers
                .insert("error".into(), "no url header".into());
            return false;
        };
        match http::fetch_via_socks(
            &self.detector.host,
            self.detector.socks_port,
            &url,
            self.request_timeout,
        ) {
            Ok(body) => {
                envelope.payload = body;
                true
            }
            Err(e) => {
                warn!("Tor request to {url} failed: {e}");
                envelope.headers.insert("error".into(), e.to_string());
                false
            }
        }
    }
}

impl Default for TorClient {
    fn default() -> Self {
        Self::new()
    }
}
