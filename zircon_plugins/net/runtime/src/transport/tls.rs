use rustls::crypto::ring::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use zircon_runtime::core::framework::net::{NetError, NetSecurityPolicy};

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
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
