use std::path::{Path, PathBuf};

const COVER_BASENAMES: &[&str] = &["cover", "thumbnail", "project"];
const COVER_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "svg"];
const COVER_DIRECTORIES: &[&[&str]] = &[&[".zircon"], &[], &["Assets"], &["assets"]];

pub fn project_cover_path(project_root: impl AsRef<Path>) -> Option<PathBuf> {
    let project_root = project_root.as_ref();
    if project_root.as_os_str().is_empty() || !project_root.is_dir() {
        return None;
    }

    cover_candidates(project_root)
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn cover_candidates(project_root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for directory in COVER_DIRECTORIES {
        let base_dir = directory
            .iter()
            .fold(project_root.to_path_buf(), |path, segment| {
                path.join(segment)
            });
        for basename in COVER_BASENAMES {
            for extension in COVER_EXTENSIONS {
                candidates.push(base_dir.join(format!("{basename}.{extension}")));
            }
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn project_cover_prefers_zircon_metadata_cover() {
        let root = temp_project_root("cover-priority");
        let metadata_dir = root.join(".zircon");
        fs::create_dir_all(&metadata_dir).unwrap();
        fs::write(root.join("cover.png"), "root").unwrap();
        let expected = metadata_dir.join("cover.png");
        fs::write(&expected, "metadata").unwrap();

        let cover = project_cover_path(&root);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(cover, Some(expected));
    }

    #[test]
    fn project_cover_accepts_asset_thumbnail_when_root_cover_is_missing() {
        let root = temp_project_root("cover-assets");
        let assets_dir = root.join("Assets");
        fs::create_dir_all(&assets_dir).unwrap();
        let expected = assets_dir.join("thumbnail.jpg");
        fs::write(&expected, "asset").unwrap();

        let cover = project_cover_path(&root);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(cover, Some(expected));
    }

    #[test]
    fn project_cover_ignores_missing_or_non_directory_roots() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cover-missing-{}",
            crate::projects::now_unix_ms()
        ));

        assert_eq!(project_cover_path(&root), None);
        assert_eq!(project_cover_path(""), None);
    }

    fn temp_project_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-{label}-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
