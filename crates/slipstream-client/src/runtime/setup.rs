use crate::error::ClientError;
use slipstream_core::net::{
    bind_first_resolved_with_ipv4_fallback, bind_tcp_listener_addr, bind_udp_socket_addr,
};
use tokio::net::{TcpListener as TokioTcpListener, UdpSocket as TokioUdpSocket};

/// Compute the MTU based on domain length and EDNS(0) support.
/// 
/// When EDNS(0) is enabled (default), uses 1232 bytes UDP payload.
/// When EDNS(0) is disabled (for 512-byte DNS limit), uses traditional 512 bytes.
pub(crate) fn compute_mtu(domain_len: usize) -> Result<u32, ClientError> {
    // Account for DNS headers and overhead
    // Without EDNS(0): 512 bytes total DNS packet
    // With EDNS(0): 1232 bytes total DNS packet
    let enable_edns0 = slipstream_dns::is_edns0_enabled();
    let max_dns_packet = if enable_edns0 { 1232 } else { 512 };
    
    // Account for DNS fixed header (12 bytes), OPT record overhead (11 bytes if present),
    // QNAME with labels, and inline dots
    let dns_header_overhead = 12;
    let opt_overhead = if enable_edns0 { 11 } else { 0 };
    let qname_overhead = domain_len + 2; // domain + dots + null terminator
    
    let available_for_payload = max_dns_packet.saturating_sub(
        dns_header_overhead + opt_overhead + qname_overhead + 6 // qtype + qclass
    );
    
    if available_for_payload < 20 {
        return Err(ClientError::new(
            "Domain name is too long for DNS transport (leaves insufficient space for payload)",
        ));
    }
    
    // Estimate MTU: account for base32 encoding efficiency (~1.6x expansion with inline dots)
    let mtu = (available_for_payload as f64 / 1.6) as u32;
    
    if mtu == 0 {
        return Err(ClientError::new(
            "MTU computed to zero; check domain length",
        ));
    }
    
    Ok(mtu)
}

pub(crate) async fn bind_udp_socket() -> Result<TokioUdpSocket, ClientError> {
    bind_first_resolved_with_ipv4_fallback(
        "::",
        0,
        |addr| bind_udp_socket_addr(addr, "UDP socket"),
        "UDP socket",
    )
    .await
    .map(|(socket, _)| socket)
    .map_err(map_io)
}

pub(crate) async fn bind_tcp_listener(
    host: &str,
    port: u16,
) -> Result<(TokioTcpListener, String), std::io::Error> {
    bind_first_resolved_with_ipv4_fallback(host, port, bind_tcp_listener_addr, "TCP listener").await
}

pub(crate) fn map_io(err: std::io::Error) -> ClientError {
    ClientError::new(err.to_string())
}
