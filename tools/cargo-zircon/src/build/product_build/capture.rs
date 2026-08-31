use std::fs::File;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{
    canonical_file, open_locked_source, sha256_serialized, CargoProductArtifact,
    CargoRuntimeArtifact, ProductBuildSdkSource, ProductBuildToolchain,
};
use crate::build::receipt::{
    digest_open_file_handle_with_buffer, ArtifactKind, FileDigestBuffer, ProductReceiptError,
    ReceiptArtifact, ReceiptArtifactSource, ToolchainComponentDigests, ToolchainSet,
};

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;

pub(super) struct PreparedProductBuildToolchain {
    cargo_path: PathBuf,
    rustc_path: PathBuf,
    linker_path: Option<PathBuf>,
    components: ToolchainComponentDigests,
    digest_buffer: FileDigestBuffer,
    _cargo_file: File,
    _rustc_file: File,
    _linker_file: Option<File>,
    _sdk_files: Vec<OpenedSdkSource>,
}

pub(super) struct OpenedCargoProduct {
    pub(super) executable: File,
    pub(super) symbols: Vec<(String, File)>,
}

pub(super) struct OpenedDeclaredArtifact {
    source: ReceiptArtifactSource,
    file: File,
}

pub(super) struct OpenedSdkSource {
    logical_name: String,
    file: File,
}

#[derive(Serialize)]
struct SdkFingerprintEntry<'a> {
    logical_name: &'a str,
    sha256: String,
    byte_length: u64,
}

impl PreparedProductBuildToolchain {
    pub(super) fn open(toolchain: &mut ProductBuildToolchain) -> Result<Self, ProductReceiptError> {
        let cargo_path = canonical_file(&toolchain.cargo_path, "Cargo executable")?;
        let rustc_path = canonical_file(&toolchain.rustc_path, "rustc executable")?;
        let linker_path = toolchain
            .linker_path
            .as_deref()
            .map(|path| canonical_file(path, "linker executable"))
            .transpose()?;

        let mut cargo_file = open_locked_source(&cargo_path, "Cargo executable")?;
        let mut rustc_file = open_locked_source(&rustc_path, "rustc executable")?;
        let mut linker_file = linker_path
            .as_deref()
            .map(|path| open_locked_source(path, "linker executable"))
            .transpose()?;
        let mut sdk_files = open_sdk_sources(std::mem::take(&mut toolchain.sdk_files))?;
        let mut digest_buffer = FileDigestBuffer::new();
        let sdk_fingerprint = capture_sdk_fingerprint(&mut sdk_files, &mut digest_buffer)?;
        let components = ToolchainComponentDigests::capture_from_file_handles(
            &mut cargo_file,
            &mut rustc_file,
            linker_file.as_mut(),
            sdk_fingerprint,
            &mut digest_buffer,
        )?;

        Ok(Self {
            cargo_path,
            rustc_path,
            linker_path,
            components,
            digest_buffer,
            _cargo_file: cargo_file,
            _rustc_file: rustc_file,
            _linker_file: linker_file,
            _sdk_files: sdk_files,
        })
    }

    pub(super) fn cargo_path(&self) -> &Path {
        &self.cargo_path
    }

    pub(super) fn rustc_path(&self) -> &Path {
        &self.rustc_path
    }

    pub(super) fn linker_path(&self) -> Option<&Path> {
        self.linker_path.as_deref()
    }

    pub(super) fn receipt_toolchain(
        &self,
        environment_digest: String,
    ) -> Result<ToolchainSet, ProductReceiptError> {
        self.components.to_toolchain(environment_digest)
    }

    pub(super) fn digest_buffer(&mut self) -> &mut FileDigestBuffer {
        &mut self.digest_buffer
    }
}

pub(super) fn open_cargo_product(
    artifact: CargoProductArtifact,
    target_directory: &Path,
) -> Result<OpenedCargoProduct, ProductReceiptError> {
    let executable_path = canonical_build_output(
        &artifact.executable,
        target_directory,
        "Cargo product executable",
    )?;
    let executable = open_locked_source(&executable_path, "Cargo product executable")?;
    let mut symbols = Vec::with_capacity(artifact.symbol_files.len());
    for symbol_path in artifact.symbol_files {
        let symbol_path =
            canonical_build_output(&symbol_path, target_directory, "Cargo product symbol")?;
        let file_name = symbol_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ProductReceiptError::new("Cargo product symbol name is not Unicode"))?
            .to_string();
        let file = open_locked_source(&symbol_path, "Cargo product symbol")?;
        symbols.push((file_name, file));
    }
    Ok(OpenedCargoProduct {
        executable,
        symbols,
    })
}

pub(super) fn open_cargo_runtime_dependency(
    artifact: CargoRuntimeArtifact<'_>,
    target_directory: &Path,
) -> Result<OpenedDeclaredArtifact, ProductReceiptError> {
    let source_path = canonical_build_output(
        &artifact.source_path,
        target_directory,
        "Cargo runtime dependency",
    )?;
    let file = open_locked_source(&source_path, "Cargo runtime dependency")?;
    Ok(OpenedDeclaredArtifact {
        source: ReceiptArtifactSource {
            logical_name: artifact.declaration.logical_name.clone(),
            relative_path: artifact.declaration.relative_path.clone(),
            kind: ArtifactKind::DynamicLibrary,
            source_path,
        },
        file,
    })
}

fn canonical_build_output(
    path: &Path,
    target_directory: &Path,
    label: &str,
) -> Result<PathBuf, ProductReceiptError> {
    if !path.is_absolute() {
        return Err(ProductReceiptError::new(format!(
            "{label} path must be absolute"
        )));
    }
    let canonical = canonical_file(path, label)?;
    if !canonical.starts_with(target_directory) {
        return Err(ProductReceiptError::new(format!(
            "{label} resolved outside the owned Cargo target directory"
        )));
    }
    Ok(canonical)
}

pub(super) fn open_sdk_sources(
    sources: Vec<ProductBuildSdkSource>,
) -> Result<Vec<OpenedSdkSource>, ProductReceiptError> {
    sources
        .into_iter()
        .map(|source| {
            let path = canonical_file(&source.source_path, &source.logical_name)?;
            let file = open_locked_source(&path, &source.logical_name)?;
            Ok(OpenedSdkSource {
                logical_name: source.logical_name,
                file,
            })
        })
        .collect()
}

pub(super) fn capture_sdk_fingerprint(
    sources: &mut [OpenedSdkSource],
    digest_buffer: &mut FileDigestBuffer,
) -> Result<String, ProductReceiptError> {
    let mut entries = Vec::with_capacity(sources.len());
    for source in sources {
        let digest = digest_open_file_handle_with_buffer(&mut source.file, digest_buffer)?;
        entries.push(SdkFingerprintEntry {
            logical_name: &source.logical_name,
            sha256: digest.sha256,
            byte_length: digest.byte_length,
        });
    }
    sha256_serialized(&entries)
}

pub(super) fn open_declared_artifact(
    source: ReceiptArtifactSource,
) -> Result<OpenedDeclaredArtifact, ProductReceiptError> {
    let file = open_locked_source(&source.source_path, &source.logical_name)?;
    Ok(OpenedDeclaredArtifact { source, file })
}

pub(super) fn capture_declared_artifacts(
    sources: Vec<OpenedDeclaredArtifact>,
    digest_buffer: &mut FileDigestBuffer,
) -> Result<Vec<ReceiptArtifact>, ProductReceiptError> {
    let mut artifacts = Vec::with_capacity(sources.len());
    for source in sources {
        artifacts.push(capture_declared_artifact(source, digest_buffer)?);
    }
    Ok(artifacts)
}

pub(super) fn capture_declared_artifact(
    source: OpenedDeclaredArtifact,
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

pub(super) fn capture_symbol_artifacts(
    symbols: Vec<(String, File)>,
    product_logical_name: &str,
    relative_directory: &str,
    digest_buffer: &mut FileDigestBuffer,
) -> Result<Vec<ReceiptArtifact>, ProductReceiptError> {
    symbols
        .into_iter()
        .map(|(file_name, file)| {
            ReceiptArtifact::capture_from_file_with_buffer(
                format!("{product_logical_name}-{file_name}"),
                format!("{relative_directory}/{file_name}"),
                ArtifactKind::SymbolFile,
                file,
                digest_buffer,
            )
        })
        .collect()
}
