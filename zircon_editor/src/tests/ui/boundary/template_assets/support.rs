use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

pub(super) fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert!(bytes.len() >= 24, "PNG data should include an IHDR header");
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "reference asset should be a PNG"
    );
    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("PNG width bytes"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("PNG height bytes"));
    (width, height)
}

pub(super) fn assert_no_files_with_extension(root: PathBuf, extension: &str) {
    if !root.exists() {
        return;
    }

    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
            {
                stack.push(
                    entry
                        .unwrap_or_else(|error| {
                            panic!("read entry under `{}`: {error}", path.display())
                        })
                        .path(),
                );
            }
            continue;
        }

        assert_ne!(
            path.extension().and_then(|value| value.to_str()),
            Some(extension),
            "active editor UI tree should not contain `{}`",
            path.display()
        );
    }
}

pub(super) fn assert_no_legacy_ui_document_suffixes(root: PathBuf) {
    if !root.exists() {
        return;
    }

    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
            {
                stack.push(
                    entry
                        .unwrap_or_else(|error| {
                            panic!("read entry under `{}`: {error}", path.display())
                        })
                        .path(),
                );
            }
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        assert!(
            !file_name.ends_with(".ui.toml"),
            "production UI asset tree must not contain legacy UI document suffix `{}`",
            path.display()
        );
    }
}

pub(super) fn collect_zui_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
            {
                stack.push(
                    entry
                        .unwrap_or_else(|error| {
                            panic!("read entry under `{}`: {error}", path.display())
                        })
                        .path(),
                );
            }
            continue;
        }

        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zui"))
        {
            files.push(path);
        }
    }
    files.sort();
    files
}
