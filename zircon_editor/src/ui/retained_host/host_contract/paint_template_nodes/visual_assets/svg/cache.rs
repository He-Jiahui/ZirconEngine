use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use resvg::usvg;

use super::parse::parse_svg_tree_data;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_svg_tree(
    path: &Path,
) -> Option<Arc<usvg::Tree>> {
    let cache_key = SvgTreeCacheKey::from_path(path);
    let cache = svg_tree_cache();
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_lookup"
        );
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&cache_key)
        {
            return cached.clone();
        }
    }

    let tree = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_parse");
        parse_svg_tree_file(path).map(Arc::new)
    };
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_store"
        );
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(cache_key, tree.clone());
    }
    tree
}

fn parse_svg_tree_file(path: &Path) -> Option<usvg::Tree> {
    let svg = fs::read(path).ok()?;
    let resources_dir = fs::canonicalize(path)
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    parse_svg_tree_data(&svg, resources_dir)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SvgTreeCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

impl SvgTreeCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

fn svg_tree_cache() -> &'static Mutex<BTreeMap<SvgTreeCacheKey, Option<Arc<usvg::Tree>>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<SvgTreeCacheKey, Option<Arc<usvg::Tree>>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}
