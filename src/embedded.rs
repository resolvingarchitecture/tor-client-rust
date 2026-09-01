//! Embedded Tor via [Arti](https://gitlab.torproject.org/tpo/core/arti), the Tor
//! Project's pure-Rust Tor client. Enabled by the `embedded` feature.
//!
//! [`EmbeddedTor::start`] boots a Tokio runtime on dedicated worker threads,
//! builds a bootstrapped `arti_client::TorClient` (persisting directory + guard
//! state under the given directory), and keeps it alive for the lifetime of the
//! value. Arti drives its own background tasks on the runtime, so — unlike the
//! embedded I2P router in `i2p-rust` — no separate driver thread is needed.
//!
//! `1m5-core-rust` never sees this: it always talks to [`crate::TorClient`],
//! which routes to the embedded backend or the local SOCKS daemon per
//! `ra.tor.mode`.

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient as ArtiClient;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use tor_rtcompat::PreferredRuntime;

use crate::http;

/// A running embedded Tor client. Dropping it stops the runtime and all of
/// Arti's background tasks.
pub struct EmbeddedTor {
    rt: Runtime,
    client: std::sync::Arc<ArtiClient<PreferredRuntime>>,
}

impl EmbeddedTor {
    /// Bootstrap Arti, storing directory/guard state under `state_dir` (e.g.
    /// `~/.1m5/core/data/tor` or `/var/lib/1m5/tor`). Blocks until bootstrapped
    /// — cold start fetches a consensus and can take 10–30s; warm starts reuse
    /// the persisted state and are fast.
    pub fn start(state_dir: PathBuf) -> io::Result<EmbeddedTor> {
        // rustls 0.23 needs a process-wide CryptoProvider; pin it to ring so
        // Arti's TLS setup is unambiguous. Idempotent; ignore "already set".
        let _ = rustls::crypto::ring::default_provider().install_default();

        let cache_dir = state_dir.join("cache");
        std::fs::create_dir_all(&state_dir)?;
        std::fs::create_dir_all(&cache_dir)?;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("tor-arti")
            .build()
            .map_err(|e| io::Error::other(format!("tokio runtime: {e}")))?;

        let config = TorClientConfigBuilder::from_directories(&state_dir, &cache_dir)
            .build()
            .map_err(|e| io::Error::other(format!("arti config: {e}")))?;

        let client = rt
            .block_on(ArtiClient::create_bootstrapped(config))
            .map_err(|e| io::Error::other(format!("arti bootstrap: {e}")))?;

        info!(
            "embedded Tor (Arti) bootstrapped; state={}",
            state_dir.display()
        );
        Ok(EmbeddedTor { rt, client })
    }

    /// Fetch an `http://` URL (`.onion` or clearnet) over a fresh Tor stream and
    /// return the response body.
    pub fn fetch(&self, url: &str, timeout: Duration) -> io::Result<Vec<u8>> {
        let (host, port, path) = http::parse_url(url)?;
        self.rt.block_on(async {
            let stream = tokio::time::timeout(timeout, self.client.connect((host.as_str(), port)))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "arti connect timed out"))?
                .map_err(|e| io::Error::other(format!("arti connect: {e}")))?;

            let (mut rd, mut wr) = tokio::io::split(stream);
            wr.write_all(http::format_get(&host, &path).as_bytes())
                .await?;
            wr.flush().await?;

            let mut raw = Vec::new();
            tokio::time::timeout(timeout, rd.read_to_end(&mut raw))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "arti read timed out"))??;
            Ok(http::split_body(raw))
        })
    }
}
