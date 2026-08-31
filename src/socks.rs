//! Minimal SOCKS5 CONNECT client (no auth) - enough to tunnel an HTTP request
//! through Tor's SOCKS proxy.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Open a TCP stream to `dest_host:dest_port` *through* the SOCKS5 proxy at
/// `proxy_host:proxy_port`.
pub fn connect_through(
    proxy_host: &str,
    proxy_port: u16,
    dest_host: &str,
    dest_port: u16,
    timeout: Duration,
) -> io::Result<TcpStream> {
    let mut s = crate::detector::connect(proxy_host, proxy_port, timeout)?;
    s.set_read_timeout(Some(timeout.max(Duration::from_secs(30))))?;
    s.set_write_timeout(Some(timeout))?;

    // greeting: VER=5, NMETHODS=1, METHOD=0 (no auth)
    s.write_all(&[0x05, 0x01, 0x00])?;
    let mut method = [0u8; 2];
    s.read_exact(&mut method)?;
    if method != [0x05, 0x00] {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("SOCKS5 proxy refused no-auth (got {method:?})"),
        ));
    }

    // request: VER=5, CMD=1 (connect), RSV=0, ATYP=3 (domain), len, name, port
    if dest_host.len() > 255 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "host too long"));
    }
    let mut req = vec![0x05, 0x01, 0x00, 0x03, dest_host.len() as u8];
    req.extend_from_slice(dest_host.as_bytes());
    req.extend_from_slice(&dest_port.to_be_bytes());
    s.write_all(&req)?;

    // reply: VER, REP, RSV, ATYP, BND.ADDR, BND.PORT
    let mut head = [0u8; 4];
    s.read_exact(&mut head)?;
    if head[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("SOCKS5 connect failed, REP={}", head[1]),
        ));
    }
    let bnd_len = match head[3] {
        0x01 => 4,
        0x04 => 16,
        0x03 => {
            let mut l = [0u8; 1];
            s.read_exact(&mut l)?;
            l[0] as usize
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("SOCKS5 bad ATYP {other}"),
            ))
        }
    };
    let mut skip = vec![0u8; bnd_len + 2];
    s.read_exact(&mut skip)?;
    Ok(s)
}
