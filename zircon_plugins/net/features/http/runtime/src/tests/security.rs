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
fn untrusted_cert_rejected_then_trusted_and_pinned_accepted() {
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
        .with_certificate_root_der(certificate_chain_der().last().unwrap().clone())
        .with_certificate_pin("127.0.0.1", certificate_sha256_pin(&leaf_certificate));

    let response = net.send_http_request(pinned).unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"tls-ok");
    server.join();
}

#[test]
fn mismatched_pin_sends_no_http_request_bytes() {
    let server = spawn_tls_fixture_server();
    let net = http_runtime_manager();
    let url = format!("https://127.0.0.1:{}/secret", server.addr.port());
    let mut request = NetHttpRequestDescriptor::new(NetRequestId::new(40), NetHttpMethod::Post, url);
    request.timeout_ms = 2_000;
    request
        .headers
        .push(("Authorization".to_string(), "Bearer secret".to_string()));
    request.body = b"private payload".to_vec();
    request.security = NetSecurityPolicy::production_tls()
        .with_certificate_root_der(certificate_chain_der().last().unwrap().clone())
        .with_certificate_pin("127.0.0.1", "sha256/wrong");
    assert!(net.send_http_request(request.clone()).is_err());

    // The fixture exits after its first HTTP request. If the failed pin leaked
    // even a request, this subsequent trusted request could not succeed.
    request.security.certificate_pins.clear();
    request.security = request.security.with_certificate_pin(
        "127.0.0.1",
        certificate_sha256_pin(&certificate_chain_der()[0]),
    );
    assert_eq!(net.send_http_request(request).unwrap().status_code, 200);
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

// Test-only CA and localhost leaf, valid 2020-2040. Never used outside this fixture.
const CHAIN_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIBdTCCARugAwIBAgIUL63CdrtpV/dOVTchm3N4+xO+hLswCgYIKoZIzj0EAwIw
HjEcMBoGA1UEAwwTWmlyY29uIEhUVFAgdGVzdCBDQTAeFw0yMDAxMDEwMDAwMDBa
Fw00MDAxMDEwMDAwMDBaMBQxEjAQBgNVBAMMCWxvY2FsaG9zdDBZMBMGByqGSM49
AgEGCCqGSM49AwEHA0IABC+DJLIuDlhThxnuXeAVMjP+DeaM0Z1X0+LpwaJ5UN0p
sFw+vhdwATvdFx/lmKBhGa/ox9bhG6MU2pwCtgreDWajQTA/MAwGA1UdEwEB/wQC
MAAwGgYDVR0RBBMwEYIJbG9jYWxob3N0hwR/AAABMBMGA1UdJQQMMAoGCCsGAQUF
BwMBMAoGCCqGSM49BAMCA0gAMEUCIDz+v1XHN0UkMpOrdC6QPGnpzKBBbUnvYxMV
QYjI1LM1AiEAv44UN01kS9pz9MvyN5btSbujTUeSzmrRqo6retACXE0=
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
MIIBZDCCAQqgAwIBAgIUOhwjFjBq2I2TwgcQ53M8MYrSYIUwCgYIKoZIzj0EAwIw
HjEcMBoGA1UEAwwTWmlyY29uIEhUVFAgdGVzdCBDQTAeFw0yMDAxMDEwMDAwMDBa
Fw00MDAxMDEwMDAwMDBaMB4xHDAaBgNVBAMME1ppcmNvbiBIVFRQIHRlc3QgQ0Ew
WTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATiklyxuO0S9ARTa/m+Hx6/6lr1Z1Ez
NjLmCFUb2DgR1/I23HWe1qa+6ZsBa3ANuKF7OF8G4flLWeE9E4/fl87DoyYwJDAS
BgNVHRMBAf8ECDAGAQH/AgEAMA4GA1UdDwEB/wQEAwIBhjAKBggqhkjOPQQDAgNI
ADBFAiB51AiQS1Z/9Qb6xB4IlWtPPCYKz1E4kt4JAC8CRLD1rwIhAMiQsB2mEqbt
U50cNFRfYzUO8Q5W2zUyDdSqMT03beUq
-----END CERTIFICATE-----
"#;

const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgn+sy6C69QA/8mQpm
Vz+F/b51hTJ56bNqXlMPuEY804ehRANCAAQvgySyLg5YU4cZ7l3gFTIz/g3mjNGd
V9Pi6cGieVDdKbBcPr4XcAE73Rcf5ZigYRmv6MfW4RujFNqcArYK3g1m
-----END PRIVATE KEY-----
"#;
