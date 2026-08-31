use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    file_digest::FileDigestBuffer, ArtifactKind, BuildAction, ProducerIdentity,
    ProductReceiptDraft, ProductReceiptError, ReceiptArtifact, TargetProfile, ToolchainSet,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainSource {
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub linker_path: Option<PathBuf>,
    pub sdk_fingerprint: String,
    pub environment_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptArtifactSource {
    pub logical_name: String,
    pub relative_path: String,
    pub kind: ArtifactKind,
    pub source_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductReceiptClosure {
    pub build_set_id: String,
    pub toolchain: ToolchainSource,
    pub target_profile: TargetProfile,
    pub action: BuildAction,
    pub producer: ProducerIdentity,
    pub build_products: Vec<ReceiptArtifactSource>,
    pub runtime_dependencies: Vec<ReceiptArtifactSource>,
    pub symbols: Vec<ReceiptArtifactSource>,
    pub sbom: Option<ReceiptArtifactSource>,
}

impl ProductReceiptClosure {
    pub fn capture(self) -> Result<ProductReceiptDraft, ProductReceiptError> {
        require_immutable_capture_platform()?;
        let mut digest_buffer = FileDigestBuffer::new();
        let cargo = open_source(&self.toolchain.cargo_path, "cargo")?;
        let rustc = open_source(&self.toolchain.rustc_path, "rustc")?;
        let linker = self
            .toolchain
            .linker_path
            .as_deref()
            .map(|path| open_source(path, "linker"))
            .transpose()?;
        let build_products = open_artifacts(self.build_products)?;
        let runtime_dependencies = open_artifacts(self.runtime_dependencies)?;
        let symbols = open_artifacts(self.symbols)?;
        let sbom = self.sbom.map(open_artifact).transpose()?;
        let toolchain = ToolchainSet::capture_from_files_with_buffer(
            cargo,
            rustc,
            linker,
            self.toolchain.sdk_fingerprint,
            self.toolchain.environment_digest,
            &mut digest_buffer,
        )?;

        Ok(ProductReceiptDraft {
            build_set_id: self.build_set_id,
            toolchain,
            target_profile: self.target_profile,
            action: self.action,
            producer: self.producer,
            build_products: capture_artifacts(build_products, &mut digest_buffer)?,
            runtime_dependencies: capture_artifacts(runtime_dependencies, &mut digest_buffer)?,
            symbols: capture_artifacts(symbols, &mut digest_buffer)?,
            sbom: sbom
                .map(|source| capture_artifact(source, &mut digest_buffer))
                .transpose()?,
        })
    }
}

#[cfg(windows)]
fn require_immutable_capture_platform() -> Result<(), ProductReceiptError> {
    Ok(())
}

#[cfg(not(windows))]
fn require_immutable_capture_platform() -> Result<(), ProductReceiptError> {
    Err(ProductReceiptError::new(
        "the immutable ProductReceipt capture backend is not implemented on this platform",
    ))
}

struct OpenedArtifactSource {
    source: ReceiptArtifactSource,
    file: File,
}

fn open_artifacts(
    sources: Vec<ReceiptArtifactSource>,
) -> Result<Vec<OpenedArtifactSource>, ProductReceiptError> {
    let mut opened = Vec::with_capacity(sources.len());
    for source in sources {
        opened.push(open_artifact(source)?);
    }
    Ok(opened)
}

fn open_artifact(
    source: ReceiptArtifactSource,
) -> Result<OpenedArtifactSource, ProductReceiptError> {
    let file = open_source(&source.source_path, &source.logical_name)?;
    Ok(OpenedArtifactSource { source, file })
}

fn capture_artifacts(
    sources: Vec<OpenedArtifactSource>,
    digest_buffer: &mut FileDigestBuffer,
) -> Result<Vec<ReceiptArtifact>, ProductReceiptError> {
    let mut artifacts = Vec::with_capacity(sources.len());
    for source in sources {
        artifacts.push(capture_artifact(source, digest_buffer)?);
    }
    Ok(artifacts)
}

fn capture_artifact(
    source: OpenedArtifactSource,
    digest_buffer: &mut FileDigestBuffer,
) -> Result<ReceiptArtifact, ProductReceiptError> {
    ReceiptArtifact::capture_from_file_with_buffer(
        source.source.logical_name,
        source.source.relative_path,
        source.source.kind,
        source.file,
        digest_buffer,
    )
}

#[cfg(test)]
mod tests;

fn open_source(path: &Path, label: &str) -> Result<File, ProductReceiptError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        // Keep peer readers working while the captured bytes cannot be replaced or mutated.
        options.share_mode(0x0000_0001);
    }
    options.open(path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not open product receipt {label} source `{}`: {error}",
            path.display()
        ))
    })
}
