use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::render::ShaderAssetKind;

use super::revision::asset_scan_revision_from_base_revision_and_content_hashes;
use super::ShaderPrewarmSource;

pub(super) fn shader_sources_with_module_dependency_hashes(
    mut sources: Vec<ShaderPrewarmSource>,
) -> Vec<ShaderPrewarmSource> {
    let include_modules = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            if source.kind != ShaderAssetKind::Include {
                return None;
            }
            source
                .import_path
                .as_ref()
                .map(|import_path| (import_path.clone(), index))
        })
        .collect::<HashMap<_, _>>();

    for index in 0..sources.len() {
        let imports = sources[index].imports.clone();
        let mut visited = HashSet::new();
        let mut visiting = Vec::new();
        let mut dependency_hashes = Vec::new();
        for import in imports {
            collect_include_module_hashes(
                import.as_str(),
                &sources,
                &include_modules,
                &mut visited,
                &mut visiting,
                &mut dependency_hashes,
            );
        }
        if dependency_hashes.is_empty() {
            continue;
        }
        sources[index].revision = asset_scan_revision_from_base_revision_and_content_hashes(
            sources[index].revision,
            &dependency_hashes,
        );
        sources[index]
            .include_content_hashes
            .extend(dependency_hashes);
    }

    sources
}

fn collect_include_module_hashes(
    import_path: &str,
    sources: &[ShaderPrewarmSource],
    include_modules: &HashMap<String, usize>,
    visited: &mut HashSet<String>,
    visiting: &mut Vec<String>,
    output: &mut Vec<String>,
) {
    if visited.contains(import_path) || visiting.iter().any(|entry| entry == import_path) {
        return;
    }
    let Some(index) = include_modules.get(import_path).copied() else {
        return;
    };
    visiting.push(import_path.to_string());
    for dependency in &sources[index].imports {
        collect_include_module_hashes(
            dependency,
            sources,
            include_modules,
            visited,
            visiting,
            output,
        );
    }
    visiting.pop();
    visited.insert(import_path.to_string());
    output.extend(sources[index].include_content_hashes.iter().cloned());
}
