use std::fs;
use std::io;
use std::path::Path;

use super::{
    error::NativeDynamicCleanupError, native_dynamic_preparation::NativeDynamicPreparation,
    NativeDynamicPreparationError,
};

pub(in crate::ui::host) fn cleanup_native_dynamic_preparation(
    preparation: &NativeDynamicPreparation,
) -> Result<(), NativeDynamicPreparationError> {
    cleanup_native_dynamic_roots(&preparation.plugin_root, &preparation.build_root)
}

pub(super) fn cleanup_native_dynamic_roots(
    plugin_root: &Path,
    build_root: &Path,
) -> Result<(), NativeDynamicPreparationError> {
    cleanup_native_dynamic_roots_with(plugin_root, build_root, |root| fs::remove_dir_all(root))
}

fn cleanup_native_dynamic_roots_with(
    plugin_root: &Path,
    build_root: &Path,
    mut remove_root: impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), NativeDynamicPreparationError> {
    let mut errors = Vec::new();
    for root in [plugin_root, build_root] {
        match remove_root(root) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                errors.push(NativeDynamicCleanupError::new(root.to_path_buf(), source));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(NativeDynamicPreparationError::cleanup_batch(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_attempts_both_roots_and_aggregates_both_failures() {
        let plugin_root = Path::new("plugin-root");
        let build_root = Path::new("build-root");
        let mut attempted = Vec::new();

        let error = cleanup_native_dynamic_roots_with(plugin_root, build_root, |root| {
            attempted.push(root.to_path_buf());
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("cannot remove {}", root.display()),
            ))
        })
        .expect_err("both synthetic root removals should fail");

        assert_eq!(
            attempted,
            vec![plugin_root.to_path_buf(), build_root.to_path_buf()]
        );
        assert!(matches!(
            &error,
            NativeDynamicPreparationError::CleanupBatch { additional, .. }
                if additional.len() == 1
        ));
        let source = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<NativeDynamicCleanupError>())
            .expect("cleanup batch must expose the first typed cleanup failure");
        let io_source = std::error::Error::source(source)
            .and_then(|source| source.downcast_ref::<io::Error>())
            .expect("typed cleanup failure must expose its IO source");
        assert_eq!(io_source.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn cleanup_ignores_missing_root_but_still_attempts_the_other_root() {
        let plugin_root = Path::new("missing-plugin-root");
        let build_root = Path::new("build-root");
        let mut attempted = Vec::new();

        cleanup_native_dynamic_roots_with(plugin_root, build_root, |root| {
            attempted.push(root.to_path_buf());
            if root == plugin_root {
                Err(io::Error::new(io::ErrorKind::NotFound, "already absent"))
            } else {
                Ok(())
            }
        })
        .expect("a missing root and a removed root are a successful cleanup");

        assert_eq!(
            attempted,
            vec![plugin_root.to_path_buf(), build_root.to_path_buf()]
        );
    }
}
