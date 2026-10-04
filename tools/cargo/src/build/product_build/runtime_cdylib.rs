use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::capture::{open_cargo_runtime_dependency, OpenedDeclaredArtifact};
use super::cargo_protocol::{CargoBuildResolution, ResolvedRuntimeDependency};
use super::{
    run_bounded_cargo_output, CargoRuntimeArtifact, ProductBuildRequest, ProductReceiptError,
};

#[cfg(test)]
#[path = "runtime_cdylib/tests/cases.rs"]
mod tests;

pub(super) struct DeclaredRuntime {
    dependency: ResolvedRuntimeDependency,
    features: Option<Vec<String>>,
}

pub(super) fn take_declared_runtime(
    resolution: &mut CargoBuildResolution<'_>,
) -> Option<DeclaredRuntime> {
    let index = resolution
        .runtime_dependencies
        .iter()
        .position(|dependency| {
            dependency.declaration.package == "zircon_runtime"
                && dependency.declaration.target == "zircon_runtime"
        })?;
    Some(DeclaredRuntime {
        dependency: resolution.runtime_dependencies.remove(index),
        features: None,
    })
}

impl DeclaredRuntime {
    pub(super) fn observe_artifact(
        &mut self,
        package_id: &str,
        target_name: &str,
        is_binary: bool,
        message: &[u8],
    ) -> Result<(), ProductReceiptError> {
        if package_id != self.dependency.package_id
            || target_name != self.dependency.declaration.target
            || is_binary
        {
            return Ok(());
        }
        #[derive(Deserialize)]
        struct Artifact {
            features: Vec<String>,
        }
        let artifact: Artifact = serde_json::from_slice(message).map_err(|error| {
            ProductReceiptError::new(format!("could not read built Runtime features: {error}"))
        })?;
        if artifact
            .features
            .iter()
            .any(|feature| feature == "dev-dynamic-linking")
        {
            return Err(ProductReceiptError::new(
                "Runtime C ABI products cannot enable development DLL features",
            ));
        }
        if !artifact
            .features
            .iter()
            .any(|feature| feature == "dynamic-api")
        {
            return Err(ProductReceiptError::new(
                "Runtime C ABI products require the dynamic-api feature",
            ));
        }
        if self.features.replace(artifact.features).is_some() {
            return Err(ProductReceiptError::new(
                "Cargo emitted more than one built Runtime feature set",
            ));
        }
        Ok(())
    }

    pub(super) fn build(
        self,
        request: &ProductBuildRequest,
        cargo: &Path,
        snapshot: &Path,
        manifest: &Path,
        target: &Path,
        environment: &[(String, String)],
    ) -> Result<OpenedDeclaredArtifact, ProductReceiptError> {
        // Metadata unifies workspace and development units. Only the executable
        // build's compiler-artifact message identifies the actual Runtime features.
        let features = self.features.ok_or_else(|| {
            ProductReceiptError::new("Cargo executable build did not report Runtime features")
        })?;
        let arguments = build_arguments(request, manifest, target, &features);
        let output = run_bounded_cargo_output(
            cargo,
            snapshot,
            &arguments,
            environment,
            super::CARGO_METADATA_OUTPUT_LIMIT,
            "Cargo Runtime C ABI build",
        )?;
        let source_path = select_runtime_artifact(&output, &self.dependency)?;
        open_cargo_runtime_dependency(
            CargoRuntimeArtifact {
                declaration: &self.dependency.declaration,
                source_path,
            },
            target,
        )
    }
}

fn build_arguments<'a>(
    request: &'a ProductBuildRequest,
    manifest: &'a Path,
    target: &'a Path,
    features: &[String],
) -> Vec<Cow<'a, OsStr>> {
    let mut arguments = vec![
        Cow::Borrowed(OsStr::new("rustc")),
        Cow::Borrowed(OsStr::new("--manifest-path")),
        Cow::Borrowed(manifest.as_os_str()),
        Cow::Borrowed(OsStr::new("--package")),
        Cow::Borrowed(OsStr::new("zircon_runtime")),
        Cow::Borrowed(OsStr::new("--lib")),
        Cow::Borrowed(OsStr::new("--crate-type")),
        Cow::Borrowed(OsStr::new("cdylib")),
        Cow::Borrowed(OsStr::new("--no-default-features")),
        Cow::Borrowed(OsStr::new("--frozen")),
        Cow::Borrowed(OsStr::new("--target")),
        Cow::Borrowed(OsStr::new(&request.target.target_triple)),
        Cow::Borrowed(OsStr::new("--profile")),
        Cow::Borrowed(OsStr::new(&request.target.cargo_profile)),
        Cow::Borrowed(OsStr::new("--target-dir")),
        Cow::Borrowed(target.as_os_str()),
        Cow::Borrowed(OsStr::new("--message-format=json-render-diagnostics")),
    ];
    super::push_features(&mut arguments, features);
    arguments
}

#[derive(Deserialize)]
struct Message {
    reason: String,
    package_id: Option<String>,
    target: Option<Target>,
    #[serde(default)]
    filenames: Vec<PathBuf>,
    success: Option<bool>,
}

#[derive(Deserialize)]
struct Target {
    name: String,
    crate_types: Vec<String>,
}

fn select_runtime_artifact(
    output: &[u8],
    dependency: &ResolvedRuntimeDependency,
) -> Result<PathBuf, ProductReceiptError> {
    let mut selected = None;
    let mut finished = None;
    for line in output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let message: Message = serde_json::from_slice(line).map_err(|error| {
            ProductReceiptError::new(format!("could not parse Runtime C ABI Cargo JSON: {error}"))
        })?;
        if message.reason == "build-finished" {
            if finished.replace(message.success).is_some() {
                return Err(ProductReceiptError::new(
                    "duplicate Runtime C ABI build-finished message",
                ));
            }
        } else if message.reason == "compiler-artifact"
            && message.package_id.as_deref() == Some(dependency.package_id.as_str())
            && message.target.as_ref().is_some_and(|target| {
                target.name == dependency.declaration.target
                    && target.crate_types.iter().any(|kind| kind == "cdylib")
            })
        {
            for path in message.filenames {
                if path.file_name() == Some(OsStr::new(&dependency.declaration.artifact_file_name))
                    && selected.replace(path).is_some()
                {
                    return Err(ProductReceiptError::new(
                        "duplicate Runtime C ABI library artifact",
                    ));
                }
            }
        }
    }
    if finished != Some(Some(true)) {
        return Err(ProductReceiptError::new(
            "Runtime C ABI Cargo build did not finish successfully",
        ));
    }
    selected.ok_or_else(|| {
        ProductReceiptError::new("Cargo did not emit the declared Runtime C ABI library")
    })
}
