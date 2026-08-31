use std::fs::File;

use serde::{Deserialize, Serialize};

use super::{
    canonical::{serialized_sha256_matches, sha256_serialized},
    file_digest::{digest_open_file_handle_with_buffer, FileDigestBuffer},
    ProductReceiptError,
};

const TOOLCHAIN_SET_SCHEMA_VERSION: u32 = 1;
const TOOLCHAIN_SET_KIND: &str = "zircon_toolchain_set";
const TOOLCHAIN_SET_IDENTITY_SERIALIZATION_ERROR: &str =
    "could not serialize ToolchainSet identity";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainSet {
    pub toolchain_set_id: String,
    pub cargo_sha256: String,
    pub rustc_sha256: String,
    pub linker_sha256: Option<String>,
    pub sdk_fingerprint: String,
    pub environment_digest: String,
}

pub(crate) struct ToolchainComponentDigests {
    cargo_sha256: String,
    rustc_sha256: String,
    linker_sha256: Option<String>,
    sdk_fingerprint: String,
}

#[derive(Serialize)]
struct CanonicalToolchainSet<'a> {
    schema_version: u32,
    toolchain_set_kind: &'a str,
    cargo_sha256: &'a str,
    rustc_sha256: &'a str,
    linker_sha256: Option<&'a str>,
    sdk_fingerprint: &'a str,
    environment_digest: &'a str,
}

impl ToolchainSet {
    pub fn new(
        cargo_sha256: String,
        rustc_sha256: String,
        linker_sha256: Option<String>,
        sdk_fingerprint: String,
        environment_digest: String,
    ) -> Result<Self, ProductReceiptError> {
        let mut toolchain = Self {
            toolchain_set_id: String::new(),
            cargo_sha256,
            rustc_sha256,
            linker_sha256,
            sdk_fingerprint,
            environment_digest,
        };
        toolchain.normalize_components()?;
        toolchain.toolchain_set_id = toolchain.derived_id()?;
        Ok(toolchain)
    }

    pub fn capture_from_files(
        cargo: File,
        rustc: File,
        linker: Option<File>,
        sdk_fingerprint: String,
        environment_digest: String,
    ) -> Result<Self, ProductReceiptError> {
        let mut digest_buffer = FileDigestBuffer::new();
        Self::capture_from_files_with_buffer(
            cargo,
            rustc,
            linker,
            sdk_fingerprint,
            environment_digest,
            &mut digest_buffer,
        )
    }

    pub(crate) fn capture_from_files_with_buffer(
        mut cargo: File,
        mut rustc: File,
        mut linker: Option<File>,
        sdk_fingerprint: String,
        environment_digest: String,
        digest_buffer: &mut FileDigestBuffer,
    ) -> Result<Self, ProductReceiptError> {
        ToolchainComponentDigests::capture_from_file_handles(
            &mut cargo,
            &mut rustc,
            linker.as_mut(),
            sdk_fingerprint,
            digest_buffer,
        )
        .into_toolchain(environment_digest)
    }

    pub(crate) fn normalize_and_verify_identity(&mut self) -> Result<(), ProductReceiptError> {
        normalize_digest("toolchain set id", &mut self.toolchain_set_id)?;
        self.normalize_components()?;
        self.verify_declared_identity()
    }

    #[cfg(test)]
    pub(crate) fn is_normalized(&self) -> bool {
        digest_is_normalized(&self.toolchain_set_id)
            && digest_is_normalized(&self.cargo_sha256)
            && digest_is_normalized(&self.rustc_sha256)
            && self
                .linker_sha256
                .as_deref()
                .is_none_or(digest_is_normalized)
            && digest_is_normalized(&self.sdk_fingerprint)
            && digest_is_normalized(&self.environment_digest)
    }

    pub(crate) fn validate_identity_if_normalized(&self) -> Result<bool, ProductReceiptError> {
        if !validate_digest_if_normalized("toolchain set id", &self.toolchain_set_id)?
            || !validate_digest_if_normalized("cargo fingerprint", &self.cargo_sha256)?
            || !validate_digest_if_normalized("rustc fingerprint", &self.rustc_sha256)?
        {
            return Ok(false);
        }
        if let Some(linker_sha256) = &self.linker_sha256 {
            if !validate_digest_if_normalized("linker fingerprint", linker_sha256)? {
                return Ok(false);
            }
        }
        if !validate_digest_if_normalized("SDK fingerprint", &self.sdk_fingerprint)?
            || !validate_digest_if_normalized("environment digest", &self.environment_digest)?
        {
            return Ok(false);
        }
        self.verify_declared_identity()?;
        Ok(true)
    }

    fn verify_declared_identity(&self) -> Result<(), ProductReceiptError> {
        let payload = self.canonical_payload();
        if !serialized_sha256_matches(
            &payload,
            &self.toolchain_set_id,
            TOOLCHAIN_SET_IDENTITY_SERIALIZATION_ERROR,
        )? {
            return Err(ProductReceiptError::new(
                "product receipt ToolchainSet identity does not match its declared components",
            ));
        }
        Ok(())
    }

    fn normalize_components(&mut self) -> Result<(), ProductReceiptError> {
        normalize_digest("cargo fingerprint", &mut self.cargo_sha256)?;
        normalize_digest("rustc fingerprint", &mut self.rustc_sha256)?;
        if let Some(linker_sha256) = &mut self.linker_sha256 {
            normalize_digest("linker fingerprint", linker_sha256)?;
        }
        normalize_digest("SDK fingerprint", &mut self.sdk_fingerprint)?;
        normalize_digest("environment digest", &mut self.environment_digest)
    }

    fn derived_id(&self) -> Result<String, ProductReceiptError> {
        // The set ID is content-addressed so callers cannot pair real component digests with an arbitrary ID.
        let payload = self.canonical_payload();
        sha256_serialized(&payload, TOOLCHAIN_SET_IDENTITY_SERIALIZATION_ERROR)
    }

    fn canonical_payload(&self) -> CanonicalToolchainSet<'_> {
        CanonicalToolchainSet {
            schema_version: TOOLCHAIN_SET_SCHEMA_VERSION,
            toolchain_set_kind: TOOLCHAIN_SET_KIND,
            cargo_sha256: &self.cargo_sha256,
            rustc_sha256: &self.rustc_sha256,
            linker_sha256: self.linker_sha256.as_deref(),
            sdk_fingerprint: &self.sdk_fingerprint,
            environment_digest: &self.environment_digest,
        }
    }
}

impl ToolchainComponentDigests {
    pub(crate) fn capture_from_file_handles(
        cargo: &mut File,
        rustc: &mut File,
        linker: Option<&mut File>,
        sdk_fingerprint: String,
        digest_buffer: &mut FileDigestBuffer,
    ) -> Result<Self, ProductReceiptError> {
        let cargo_sha256 = digest_open_file_handle_with_buffer(cargo, digest_buffer)?.sha256;
        let rustc_sha256 = digest_open_file_handle_with_buffer(rustc, digest_buffer)?.sha256;
        let linker_sha256 = linker
            .map(|linker| digest_open_file_handle_with_buffer(linker, digest_buffer))
            .transpose()?
            .map(|digest| digest.sha256);
        Ok(Self {
            cargo_sha256,
            rustc_sha256,
            linker_sha256,
            sdk_fingerprint,
        })
    }

    pub(crate) fn to_toolchain(
        &self,
        environment_digest: String,
    ) -> Result<ToolchainSet, ProductReceiptError> {
        ToolchainSet::new(
            self.cargo_sha256.clone(),
            self.rustc_sha256.clone(),
            self.linker_sha256.clone(),
            self.sdk_fingerprint.clone(),
            environment_digest,
        )
    }

    fn into_toolchain(
        self,
        environment_digest: String,
    ) -> Result<ToolchainSet, ProductReceiptError> {
        ToolchainSet::new(
            self.cargo_sha256,
            self.rustc_sha256,
            self.linker_sha256,
            self.sdk_fingerprint,
            environment_digest,
        )
    }
}

fn normalize_digest(label: &str, value: &mut String) -> Result<(), ProductReceiptError> {
    validate_digest(label, value)?;
    value.make_ascii_uppercase();
    Ok(())
}

fn validate_digest_if_normalized(label: &str, value: &str) -> Result<bool, ProductReceiptError> {
    validate_digest(label, value)?;
    Ok(digest_is_normalized(value))
}

fn validate_digest(label: &str, value: &str) -> Result<(), ProductReceiptError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ProductReceiptError::new(format!(
            "product receipt {label} must be a SHA-256 hex digest"
        )));
    }
    Ok(())
}

fn digest_is_normalized(value: &str) -> bool {
    !value.bytes().any(|byte| byte.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;
