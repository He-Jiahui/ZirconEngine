use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::export::ExportTargetMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformBundleLayout {
    pub engine_root: PathBuf,
    pub launcher: PathBuf,
    pub runtime_library: PathBuf,
    pub assets_root: PathBuf,
}

impl PlatformBundleLayout {
    pub fn expected(build_output_root: impl AsRef<Path>, target_mode: ExportTargetMode) -> Self {
        let engine_root = build_output_root.as_ref().join("ZirconEngine");
        let assets_root = engine_root.join("assets");
        let runtime_library = engine_root.join(runtime_library_name());
        let launcher = match target_mode {
            ExportTargetMode::ClientRuntime => engine_root.join(executable_name("zircon_hub")),
            ExportTargetMode::ServerRuntime => engine_root.join(executable_name("zircon_runtime")),
        };
        Self {
            engine_root,
            launcher,
            runtime_library,
            assets_root,
        }
    }

    pub fn validate(
        build_output_root: impl AsRef<Path>,
        target_mode: ExportTargetMode,
    ) -> Result<Self, PlatformBundleLayoutError> {
        let expected = Self::expected(build_output_root, target_mode);
        let engine_root = expected.engine_root;
        require_directory(&engine_root)?;
        let assets_root = engine_root.join("assets");
        require_directory(&assets_root)?;
        let runtime_library = expected.runtime_library;
        require_file(&runtime_library)?;
        let launcher = match target_mode {
            ExportTargetMode::ClientRuntime => {
                let editor = engine_root.join(executable_name("zircon_editor"));
                require_file(&editor)?;
                let hub = expected.launcher;
                require_file(&hub)?;
                hub
            }
            ExportTargetMode::ServerRuntime => {
                let runtime = expected.launcher;
                require_file(&runtime)?;
                runtime
            }
        };
        Ok(Self {
            engine_root,
            launcher,
            runtime_library,
            assets_root,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformBundleLayoutError {
    MissingDirectory { path: PathBuf },
    MissingFile { path: PathBuf },
}

impl fmt::Display for PlatformBundleLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDirectory { path } => write!(
                formatter,
                "platform bundle requires directory {}",
                path.display()
            ),
            Self::MissingFile { path } => {
                write!(
                    formatter,
                    "platform bundle requires file {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for PlatformBundleLayoutError {}

fn require_directory(path: &Path) -> Result<(), PlatformBundleLayoutError> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(PlatformBundleLayoutError::MissingDirectory {
            path: path.to_path_buf(),
        })
    }
}

fn require_file(path: &Path) -> Result<(), PlatformBundleLayoutError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(PlatformBundleLayoutError::MissingFile {
            path: path.to_path_buf(),
        })
    }
}

#[cfg(target_os = "windows")]
const fn runtime_library_name() -> &'static str {
    "zircon_runtime.dll"
}

#[cfg(target_os = "macos")]
const fn runtime_library_name() -> &'static str {
    "libzircon_runtime.dylib"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const fn runtime_library_name() -> &'static str {
    "libzircon_runtime.so"
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}
