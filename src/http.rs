//! A tiny HTTP/1.1 GET over a SOCKS tunnel. HTTP only (no TLS).

use std::io::{self, Read, Write};
use std::time::Duration;

use crate::socks;

/// Fetch `url` (http:// only) through the SOCKS5 proxy; returns the response body.
pub fn fetch_via_socks(
    proxy_host: &str,
    proxy_port: u16,
    url: &str,
    timeout: Duration,
) -> io::Result<Vec<u8>> {
    let (host, port, path) = parse_url(url)?;

    let mut stream = socks::connect_through(proxy_host, proxy_port, &host, port, timeout)?;
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: ra-tor-client\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;

    // Split headers / body on the first CRLFCRLF.
    match find(&raw, b"\r\n\r\n") {
        Some(i) => Ok(raw[i + 4..].to_vec()),
        None => Ok(raw),
    }
}

fn parse_url(url: &str) -> io::Result<(String, u16, String)> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "only http:// URLs are supported (HTTPS needs a TLS crate - see TODO.md)",
        )
    })?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (
            h.to_string(),
            p.parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "bad port"))?,
        ),
        None => (authority.to_string(), 80u16),
    };
    Ok((host, port, path.to_string()))
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}
