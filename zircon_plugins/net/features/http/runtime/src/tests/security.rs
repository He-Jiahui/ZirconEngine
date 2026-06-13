use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsAcceptor;
use zircon_plugin_net_runtime::{certificate_sha256_pin, rustls_server_config, TlsServerIdentity};
use zircon_runtime::core::framework::net::{
    NetError, NetHttpMethod, NetHttpRequestDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

use crate::http_runtime_manager;

#[test]
fn http_feature_manager_rejects_requests_that_violate_security_policy_before_network_io() {
    let net = http_runtime_manager();
    let mut tls_required = NetHttpRequestDescriptor::new(
        NetRequestId::new(31),
        NetHttpMethod::Get,
        "http://example.invalid/socket-health",
    );
    tls_required.security = NetSecurityPolicy::production_tls();

    assert_eq!(
        net.send_http_request(tls_required).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "HTTP request requires HTTPS by security policy".to_string(),
        }
    );

    let mut pinning_missing = NetHttpRequestDescriptor::new(
        NetRequestId::new(32),
        NetHttpMethod::Get,
        "https://example.invalid/socket-health",
    );
    pinning_missing.security.certificate_pinning = true;

    assert_eq!(
        net.send_http_request(pinning_missing).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}

#[test]
fn http_feature_manager_accepts_configured_certificate_pin_before_network_io() {
    let net = http_runtime_manager();
    let mut request = NetHttpRequestDescriptor::new(
        NetRequestId::new(33),
        NetHttpMethod::Get,
        "https://example.invalid/socket-health",
    );
    request.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("example.invalid", "sha256/example");

    let error = net.send_http_request(request).unwrap_err();
    assert_ne!(
        error,
        NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}

#[test]
fn self_signed_cert_rejected_then_pinned_accepted() {
    let leaf_certificate = certificate_chain_der()[0].clone();
    let server = spawn_tls_fixture_server();
    let net = http_runtime_manager();
    let url = format!("https://127.0.0.1:{}/tls", server.addr.port());

    let mut unpinned =
        NetHttpRequestDescriptor::new(NetRequestId::new(38), NetHttpMethod::Get, url.clone());
    unpinned.timeout_ms = 2_000;
    unpinned.security = NetSecurityPolicy::production_tls();

    let error = net.send_http_request(unpinned).unwrap_err();
    assert!(
        matches!(error, NetError::Io(_)),
        "self-signed cert should be rejected before pinning: {error:?}"
    );

    let mut pinned = NetHttpRequestDescriptor::new(NetRequestId::new(39), NetHttpMethod::Get, url);
    pinned.timeout_ms = 2_000;
    pinned.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("127.0.0.1", certificate_sha256_pin(&leaf_certificate));

    let response = net.send_http_request(pinned).unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"tls-ok");
    server.join();
}

struct TlsFixtureServer {
    addr: SocketAddr,
    thread: std::thread::JoinHandle<()>,
}

impl TlsFixtureServer {
    fn join(self) {
        self.thread
            .join()
            .expect("TLS fixture server thread should finish cleanly");
    }
}

fn spawn_tls_fixture_server() -> TlsFixtureServer {
    let (addr_tx, addr_rx) = std::sync::mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("TLS fixture runtime should build");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
                .await
                .expect("TLS fixture listener should bind");
            let addr = listener
                .local_addr()
                .expect("TLS fixture listener should expose local addr");
            addr_tx
                .send(addr)
                .expect("TLS fixture server addr receiver should be alive");
            let identity = TlsServerIdentity::new(certificate_chain_der(), private_key_der())
                .expect("TLS fixture identity should be valid");
            let acceptor = TlsAcceptor::from(Arc::new(
                rustls_server_config(&identity).expect("TLS fixture config should build"),
            ));
            serve_one_tls_response(listener, acceptor)
                .await
                .expect("TLS fixture should serve one successful response");
        });
    });
    let addr = addr_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("TLS fixture server should publish bound addr");
    TlsFixtureServer { addr, thread }
}

async fn serve_one_tls_response(
    listener: tokio::net::TcpListener,
    acceptor: TlsAcceptor,
) -> io::Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        let (stream, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TLS accept timed out"))??;
        let mut stream = match acceptor.accept(stream).await {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        let mut request = [0_u8; 1024];
        let _ = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut request)).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 6\r\nconnection: close\r\n\r\ntls-ok")
            .await?;
        let _ = stream.shutdown().await;
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "TLS fixture did not receive a pinned request",
    ))
}

fn certificate_chain_der() -> Vec<Vec<u8>> {
    pem_blocks(CHAIN_PEM, "CERTIFICATE")
}

fn private_key_der() -> Vec<u8> {
    pem_blocks(PRIVATE_KEY_PEM, "PRIVATE KEY")
        .into_iter()
        .next()
        .expect("fixture private key should decode")
}

fn pem_blocks(pem: &str, label: &str) -> Vec<Vec<u8>> {
    let begin = format!("-----BEGIN {label}-----");
    let end = format!("-----END {label}-----");
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_block = false;
    for line in pem.lines() {
        if line == begin {
            current.clear();
            in_block = true;
        } else if line == end {
            blocks.push(
                base64::engine::general_purpose::STANDARD
                    .decode(current.as_bytes())
                    .expect("fixture PEM block should decode"),
            );
            in_block = false;
        } else if in_block {
            current.push_str(line.trim());
        }
    }
    blocks
}

const CHAIN_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIBszCCAVmgAwIBAgIUUg3keFcU1xXWK8BNVb1KynPulV8wCgYIKoZIzj0EAwIw
JjEkMCIGA1UEAwwbUnVzdGxzIFJvYnVzdCBSb290IC0gUnVuZyAyMCAXDTc1MDEw
MTAwMDAwMFoYDzQwOTYwMTAxMDAwMDAwWjAhMR8wHQYDVQQDDBZyY2dlbiBzZWxm
IHNpZ25lZCBjZXJ0MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEud6w4gtZ0xbw
J3E69SSMy5TZfdIifl9L5ZY+hgEe4UiUsBWS32f6Y5NR5Jo8FO1f6o13b3+FvVHR
EHCGdvppL6NoMGYwFQYDVR0RBA4wDIIKZm9vYmFyLmNvbTAdBgNVHSUEFjAUBggr
BgEFBQcDAQYIKwYBBQUHAwIwHQYDVR0OBBYEFELvxbj5tD75n4pYFvJyr+c8qVEi
MA8GA1UdEwEB/wQFMAMBAQAwCgYIKoZIzj0EAwIDSAAwRQIhALxSSdUsrRFnwNMu
/doBqI8i8u5HdohVAheFTDwObkOMAiASSjULUtkWSD15u/7Sr01Wm9J1MpqW1pob
BVqU3CNRlA==
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
MIIBiTCCATCgAwIBAgIUHWiVYIvMMWoZEFYvSz46COf2FqowCgYIKoZIzj0EAwIw
HTEbMBkGA1UEAwwSUnVzdGxzIFJvYnVzdCBSb290MCAXDTc1MDEwMTAwMDAwMFoY
DzQwOTYwMTAxMDAwMDAwWjAmMSQwIgYDVQQDDBtSdXN0bHMgUm9idXN0IFJvb3Qg
LSBSdW5nIDIwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATAOCcBD7dXjmAZ3te5
D47cCJ9ec93PWv7BKYIL826CJsKfXQOGrBTthLm77hXLhHu6uv8E5QXNLZpfowLQ
Do1ao0MwQTAPBgNVHQ8BAf8EBQMDB4QAMB0GA1UdDgQWBBRdza76r11Ok9vRmlg6
Nn/wL/N+jTAPBgNVHRMBAf8EBTADAQH/MAoGCCqGSM49BAMCA0cAMEQCIFmZrXeK
hnfkahocvkhhNT3cDv1LWf6WBoFaCiBwZXFPAiARaKRiSCMG7PCHmSqFe82TBVmL
odHGogAVax1Dh/aYAA==
-----END CERTIFICATE-----"#;

const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgTbAQpfjAT46fgF4B
mP15n37woNG5ZNJmwcqsred/7tmhRANCAAS53rDiC1nTFvAncTr1JIzLlNl90iJ+
X0vllj6GAR7hSJSwFZLfZ/pjk1HkmjwU7V/qjXdvf4W9UdEQcIZ2+mkv
-----END PRIVATE KEY-----"#;
