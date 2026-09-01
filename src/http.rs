//! A tiny HTTP/1.1 GET. HTTP only (no TLS). The request/response helpers are
//! transport-agnostic; [`fetch_via_socks`] drives them over a blocking SOCKS
//! tunnel, and `embedded` drives them over an Arti stream.

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
    stream.write_all(format_get(&host, &path).as_bytes())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    Ok(split_body(raw))
}

/// Split `host:port` and path out of an `http://` URL. Errors on any other scheme.
pub(crate) fn parse_url(url: &str) -> io::Result<(String, u16, String)> {
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

/// The GET request line + headers for `path` on `host`, `Connection: close`.
pub(crate) fn format_get(host: &str, path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: ra-tor-client\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    )
}

/// Everything after the first CRLFCRLF (the body), or the whole buffer if no
/// header/body separator is present.
pub(crate) fn split_body(raw: Vec<u8>) -> Vec<u8> {
    match raw.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(i) => raw[i + 4..].to_vec(),
        None => raw,
    }
}
