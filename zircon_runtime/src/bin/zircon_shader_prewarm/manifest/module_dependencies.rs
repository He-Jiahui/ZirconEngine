use std::collections::{BTreeMap, HashMap, HashSet};

use zircon_runtime::core::framework::render::ShaderAssetKind;

use super::revision::asset_scan_revision_from_base_revision_and_content_hashes;
use super::ShaderPrewarmSource;

pub(super) fn shader_sources_with_module_dependency_hashes(
    mut sources: Vec<ShaderPrewarmSource>,
    external_include_modules: &BTreeMap<String, String>,
) -> Vec<ShaderPrewarmSource> {
    let mut include_modules = external_include_modules
        .iter()
        .map(|(import_path, content_hash)| {
            (
                import_path.clone(),
                IncludeModuleDependency::External(content_hash.clone()),
            )
        })
        .collect::<HashMap<_, _>>();
    include_modules.extend(sources.iter().enumerate().filter_map(|(index, source)| {
        if source.kind != ShaderAssetKind::Include {
            return None;
        }
        source
            .import_path
            .as_ref()
            .map(|import_path| (import_path.clone(), IncludeModuleDependency::Local(index)))
    }));

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
    include_modules: &HashMap<String, IncludeModuleDependency>,
    visited: &mut HashSet<String>,
    visiting: &mut Vec<String>,
    output: &mut Vec<String>,
) {
    if visited.contains(import_path) || visiting.iter().any(|entry| entry == import_path) {
        return;
    }
    let Some(dependency) = include_modules.get(import_path).cloned() else {
        return;
    };
    let index = match dependency {
        IncludeModuleDependency::External(content_hash) => {
            visited.insert(import_path.to_string());
            output.push(content_hash);
            return;
        }
        IncludeModuleDependency::Local(index) => index,
    };
    visiting.push(import_path.to_string());
    for dependency_import in &sources[index].imports {
        collect_include_module_hashes(
            dependency_import,
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

#[derive(Clone, Debug)]
enum IncludeModuleDependency {
    Local(usize),
    External(String),
}
