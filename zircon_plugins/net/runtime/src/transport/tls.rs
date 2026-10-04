use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::crypto::ring::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{
    ClientConfig, DigitallySignedStruct, DistinguishedName, Error as RustlsError, RootCertStore,
    ServerConfig, SignatureScheme,
};
use zircon_runtime::core::framework::net::{NetError, NetSecurityPolicy};

#[cfg(test)]
#[path = "tls/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/tls.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlsServerIdentity {
    certificate_chain_der: Vec<Vec<u8>>,
    private_key_der: Vec<u8>,
}

impl TlsServerIdentity {
    pub fn new(
        certificate_chain_der: impl IntoIterator<Item = Vec<u8>>,
        private_key_der: impl Into<Vec<u8>>,
    ) -> Result<Self, NetError> {
        let certificate_chain_der = certificate_chain_der.into_iter().collect::<Vec<_>>();
        if certificate_chain_der.is_empty() {
            return Err(NetError::SecurityPolicyViolation {
                reason: "TLS server identity requires at least one certificate".to_string(),
            });
        }
        let private_key_der = private_key_der.into();
        if private_key_der.is_empty() {
            return Err(NetError::SecurityPolicyViolation {
                reason: "TLS server identity requires a private key".to_string(),
            });
        }
        Ok(Self {
            certificate_chain_der,
            private_key_der,
        })
    }

    pub fn certificate_chain_der(&self) -> &[Vec<u8>] {
        &self.certificate_chain_der
    }

    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key_der
    }
}

pub fn rustls_client_config(policy: &NetSecurityPolicy) -> Result<ClientConfig, NetError> {
    let root_store = rustls_root_store(policy)?;
    Ok(
        ClientConfig::builder_with_provider(default_provider().into())
            .with_safe_default_protocol_versions()
            .map_err(|error| NetError::Io(error.to_string()))?
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
}

/// Build the rustls client configuration used by WebSocket connections that
/// opt into a custom root set or certificate pinning.
///
/// The normal WebSocket path keeps tokio-tungstenite's stock WebPKI
/// connector.  This path is deliberately opt-in so a project that supplies a
/// root or pin gets the same chain/hostname validation plus the requested
/// policy, instead of silently falling back to a connector that ignores it.
pub fn rustls_client_config_for_websocket(
    policy: &NetSecurityPolicy,
    host: impl Into<String>,
) -> Result<ClientConfig, NetError> {
    let root_store = websocket_root_store(policy)?;
    let verifier = WebPkiServerVerifier::builder_with_provider(
        Arc::new(root_store),
        default_provider().into(),
    )
    .build()
    .map_err(|error| NetError::SecurityPolicyViolation {
        reason: format!("TLS verifier could not be built: {error}"),
    })?;
    let verifier: Arc<dyn ServerCertVerifier> = if policy.certificate_pinning {
        Arc::new(PinningServerCertVerifier {
            inner: verifier,
            policy: policy.clone(),
            host: host.into(),
        })
    } else {
        verifier
    };

    Ok(
        ClientConfig::builder_with_provider(default_provider().into())
            .with_safe_default_protocol_versions()
            .map_err(|error| NetError::Io(error.to_string()))?
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth(),
    )
}

pub fn rustls_server_config(identity: &TlsServerIdentity) -> Result<ServerConfig, NetError> {
    let certificate_chain = identity
        .certificate_chain_der
        .iter()
        .cloned()
        .map(CertificateDer::from)
        .collect::<Vec<_>>();
    let private_key =
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(identity.private_key_der.clone()));
    ServerConfig::builder_with_provider(default_provider().into())
        .with_safe_default_protocol_versions()
        .map_err(|error| NetError::Io(error.to_string()))?
        .with_no_client_auth()
        .with_single_cert(certificate_chain, private_key)
        .map_err(|error| NetError::SecurityPolicyViolation {
            reason: format!("TLS server identity rejected by rustls: {error}"),
        })
}

pub fn rustls_root_store(policy: &NetSecurityPolicy) -> Result<RootCertStore, NetError> {
    let mut roots = RootCertStore::empty();
    for root in &policy.certificate_roots {
        roots
            .add(CertificateDer::from(root.der.clone()))
            .map_err(|error| NetError::SecurityPolicyViolation {
                reason: format!("TLS root certificate rejected by rustls: {error}"),
            })?;
    }
    Ok(roots)
}

fn websocket_root_store(policy: &NetSecurityPolicy) -> Result<RootCertStore, NetError> {
    if !policy.certificate_roots.is_empty() {
        return rustls_root_store(policy);
    }

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(roots)
}

#[derive(Debug)]
struct PinningServerCertVerifier {
    inner: Arc<WebPkiServerVerifier>,
    policy: NetSecurityPolicy,
    host: String,
}

impl ServerCertVerifier for PinningServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        let verified = self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;
        if certificate_pin_matches(&self.policy, &self.host, end_entity.as_ref()) {
            Ok(verified)
        } else {
            Err(RustlsError::General(format!(
                "certificate pin mismatch for host: {}",
                self.host
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }

    fn root_hint_subjects(&self) -> Option<&[DistinguishedName]> {
        self.inner.root_hint_subjects()
    }
}

pub fn certificate_sha256_pin(der: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, der);
    format!("sha256/{}", hex_encode(digest.as_ref()))
}

pub fn certificate_pin_matches(policy: &NetSecurityPolicy, host: &str, der: &[u8]) -> bool {
    let actual = normalize_certificate_pin(&certificate_sha256_pin(der));
    policy.certificate_pins.iter().any(|pin| {
        pin.host.eq_ignore_ascii_case(host) && normalize_certificate_pin(&pin.sha256) == actual
    })
}

fn normalize_certificate_pin(pin: &str) -> String {
    pin.trim()
        .strip_prefix("sha256/")
        .unwrap_or_else(|| pin.trim())
        .chars()
        .filter(|character| !matches!(character, ':' | ' ' | '\t' | '\r' | '\n'))
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for &byte in bytes {
        encoded.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        encoded.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}
