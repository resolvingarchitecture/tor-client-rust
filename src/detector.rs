//! Detects a local Tor daemon.

use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use log::debug;

/// Default loopback ports a Tor daemon listens on.
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_SOCKS_PORT: u16 = 9050;
pub const DEFAULT_CONTROL_PORT: u16 = 9051;

/// Probes SOCKS + control ports so [`crate::TorClient::start`] can fail fast with
/// a clear message instead of a confusing SOCKS/control error later.
///
/// Tor is a C daemon, so - like `tor-client-java` and unlike I2P (a pure-Rust
/// router via emissary in `i2p-rust`) - this client only ever uses a Tor
/// instance already installed and running on the host.
#[derive(Clone, Debug)]
pub struct LocalTorDetector {
    pub host: String,
    pub socks_port: u16,
    pub control_port: u16,
    pub timeout: Duration,
}

impl Default for LocalTorDetector {
    fn default() -> Self {
        LocalTorDetector {
            host: DEFAULT_HOST.to_string(),
            socks_port: DEFAULT_SOCKS_PORT,
            control_port: DEFAULT_CONTROL_PORT,
            timeout: Duration::from_millis(750),
        }
    }
}

impl LocalTorDetector {
    pub fn is_socks_reachable(&self) -> bool {
        self.reachable(self.socks_port)
    }

    pub fn is_control_reachable(&self) -> bool {
        self.reachable(self.control_port)
    }

    /// True only if both the SOCKS proxy and the control port answer.
    pub fn is_local_tor_running(&self) -> bool {
        self.is_socks_reachable() && self.is_control_reachable()
    }

    fn reachable(&self, port: u16) -> bool {
        match connect(&self.host, port, self.timeout) {
            Ok(_) => true,
            Err(e) => {
                debug!("nothing on {}:{} ({e})", self.host, port);
                false
            }
        }
    }
}

pub(crate) fn connect(host: &str, port: u16, timeout: Duration) -> io::Result<TcpStream> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::other("no address"))?;
    TcpStream::connect_timeout(&addr, timeout)
}
