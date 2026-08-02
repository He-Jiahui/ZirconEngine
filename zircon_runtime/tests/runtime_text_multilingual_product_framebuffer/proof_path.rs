use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

const PROOF_FILE_NAME: &str = "runtime_text_mvp_foundation_product_framebuffer_20260801.png";

pub(super) fn proof_path() -> PathBuf {
    workspace_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(PROOF_FILE_NAME)
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest must have a workspace parent")
        .to_path_buf()
}

pub(super) fn configured_cargo_target_dir() -> Option<PathBuf> {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(require_absolute_target_dir)
}

pub(super) fn assert_product_proof_is_outside_target(output: &Path, target_dir: &Path) {
    assert!(
        product_proof_is_outside_target(output, target_dir),
        "runtime text proof must not be written under cargo target: output={}, target={}",
        output.display(),
        target_dir.display(),
    );
}

pub(super) fn product_proof_is_outside_target(output: &Path, target_dir: &Path) -> bool {
    let output = canonicalize_or_normalize_path(output);
    let target_dir = canonicalize_or_normalize_path(target_dir);
    !path_starts_with(&output, &target_dir)
}

fn require_absolute_target_dir(target_dir: PathBuf) -> PathBuf {
    assert!(
        target_dir.is_absolute(),
        "CARGO_TARGET_DIR must be an absolute coordinator path before exporting a framebuffer proof: {}",
        target_dir.display(),
    );
    target_dir
}

fn canonicalize_or_normalize_path(path: &Path) -> PathBuf {
    let normalized = normalize_path_lexically(path);
    let mut existing_ancestor = normalized.as_path();
    let mut missing_components = Vec::<OsString>::new();
    while !existing_ancestor.exists() {
        let Some(component) = existing_ancestor.file_name() else {
            return normalized;
        };
        missing_components.push(component.to_os_string());
        let Some(parent) = existing_ancestor.parent() else {
            return normalized;
        };
        existing_ancestor = parent;
    }

    let Ok(mut canonical) = existing_ancestor.canonicalize() else {
        return normalized;
    };
    for component in missing_components.iter().rev() {
        canonical.push(component);
    }
    canonical
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    #[cfg(windows)]
    {
        let mut path_components = path.components();
        return prefix.components().all(|prefix_component| {
            path_components.next().is_some_and(|path_component| {
                path_component
                    .as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&prefix_component.as_os_str().to_string_lossy())
            })
        });
    }

    #[cfg(not(windows))]
    path.starts_with(prefix)
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_framebuffer_path_stays_in_docs_text_and_outside_target() {
        let workspace_root = workspace_root();
        let output = proof_path();

        assert_eq!(
            output,
            workspace_root
                .join("docs")
                .join("tests")
                .join("runtime")
                .join("text")
                .join(PROOF_FILE_NAME),
        );
        assert!(product_proof_is_outside_target(
            &output,
            &workspace_root.join("target"),
        ));
        assert!(!product_proof_is_outside_target(
            &output,
            &workspace_root.join("docs"),
        ));
        #[cfg(windows)]
        assert!(!product_proof_is_outside_target(
            &output,
            &workspace_root.join("DOCS"),
        ));
    }

    #[test]
    #[should_panic(expected = "CARGO_TARGET_DIR must be an absolute coordinator path")]
    fn relative_external_target_is_rejected() {
        let _ = require_absolute_target_dir(PathBuf::from("cargo-targets").join("text-proof"));
    }

    #[test]
    fn absolute_external_target_is_accepted() {
        let workspace_root = workspace_root();
        let target_dir = workspace_root.join("external-cargo-target");

        assert_eq!(require_absolute_target_dir(target_dir.clone()), target_dir,);
    }
}
