use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::ffi::OsStr;
use std::fmt;
use std::io::BufRead;
use std::path::{Component, Path, PathBuf};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use super::{
    sha256_serialized, CargoProductArtifact, CargoRuntimeArtifact,
    CargoRuntimeDependencyDeclaration, ProductReceiptError,
};

const CARGO_JSON_MESSAGE_LINE_LIMIT: usize = 4 * 1024 * 1024;
const CARGO_JSON_MESSAGE_COUNT_LIMIT: usize = 100_000;
const CARGO_METADATA_PACKAGE_LIMIT: usize = 100_000;
const CARGO_METADATA_RESOLVE_NODE_LIMIT: usize = 100_000;
const CARGO_METADATA_RESOLVE_EDGE_LIMIT: usize = 1_000_000;

pub(super) struct CargoBuildResolution<'a> {
    pub(super) product_package_id: String,
    pub(super) product_binary: &'a str,
    pub(super) runtime_dependencies: Vec<ResolvedRuntimeDependency>,
    pub(super) cargo_graph_digest: String,
}

pub(super) struct ResolvedRuntimeDependency {
    pub(super) declaration: CargoRuntimeDependencyDeclaration,
    package_id: String,
}

#[derive(Deserialize)]
struct CargoMessageHeader<'a> {
    #[serde(borrow)]
    reason: &'a str,
    #[serde(default, borrow)]
    package_id: Option<&'a str>,
    #[serde(default, borrow)]
    target: Option<CargoTargetHeader<'a>>,
    #[serde(default)]
    success: Option<bool>,
}

#[derive(Deserialize)]
struct CargoTargetHeader<'a> {
    #[serde(borrow)]
    name: &'a str,
    #[serde(rename = "kind", deserialize_with = "deserialize_cargo_target_kind")]
    is_binary: bool,
}

fn deserialize_cargo_target_kind<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct CargoTargetKindVisitor;

    impl<'de> Visitor<'de> for CargoTargetKindVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a Cargo target kind array")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut is_binary = false;
            while let Some(kind) = sequence.next_element::<&'de str>()? {
                is_binary |= kind == "bin";
            }
            Ok(is_binary)
        }
    }

    deserializer.deserialize_seq(CargoTargetKindVisitor)
}

#[derive(Deserialize)]
struct CargoArtifactPayload {
    #[serde(default)]
    filenames: Vec<PathBuf>,
    #[serde(default)]
    executable: Option<PathBuf>,
}

#[derive(Deserialize, Serialize)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
    resolve: Option<CargoResolve>,
    #[serde(default)]
    workspace_members: Vec<String>,
    #[serde(default)]
    workspace_default_members: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct CargoMetadataPackage {
    id: String,
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    checksum: Option<String>,
    #[serde(default)]
    manifest_path: PathBuf,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
    targets: Vec<CargoMetadataTarget>,
}

#[derive(Deserialize, Serialize)]
struct CargoMetadataTarget {
    name: String,
    kind: Vec<String>,
    #[serde(default)]
    crate_types: Vec<String>,
    #[serde(default, rename = "required-features")]
    required_features: Vec<String>,
    #[serde(default)]
    edition: String,
    #[serde(default)]
    src_path: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct CargoResolve {
    nodes: Vec<CargoResolveNode>,
}

#[derive(Deserialize, Serialize)]
struct CargoResolveNode {
    id: String,
    dependencies: Vec<String>,
    #[serde(default)]
    deps: Vec<CargoResolveDependency>,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct CargoResolveDependency {
    name: String,
    pkg: String,
    #[serde(default)]
    dep_kinds: Vec<CargoResolveDependencyKind>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct CargoResolveDependencyKind {
    kind: Option<String>,
    target: Option<String>,
}

pub(super) fn resolve_build<'a>(
    metadata_bytes: &[u8],
    snapshot_root: &Path,
    product_package: &str,
    product_binary: &'a str,
    runtime_dependencies: Vec<CargoRuntimeDependencyDeclaration>,
) -> Result<CargoBuildResolution<'a>, ProductReceiptError> {
    let mut metadata: CargoMetadata = serde_json::from_slice(metadata_bytes).map_err(|error| {
        ProductReceiptError::new(format!("could not parse Cargo metadata output: {error}"))
    })?;
    if metadata.packages.len() > CARGO_METADATA_PACKAGE_LIMIT {
        return Err(ProductReceiptError::new(format!(
            "Cargo metadata exceeded the {CARGO_METADATA_PACKAGE_LIMIT}-package limit"
        )));
    }
    let mut selected_package_names =
        HashSet::with_capacity(runtime_dependencies.len().saturating_add(1));
    selected_package_names.insert(product_package);
    for dependency in &runtime_dependencies {
        selected_package_names.insert(dependency.package.as_str());
    }
    let package_index = selected_package_index(&metadata, &selected_package_names)?;
    let product = package_index.get(product_package).copied().ok_or_else(|| {
        ProductReceiptError::new(format!(
            "Cargo metadata did not contain package `{product_package}`"
        ))
    })?;
    let product_package_id = product.id.clone();
    if !product
        .targets
        .iter()
        .any(|target| target.name == product_binary && target.kind.iter().any(|kind| kind == "bin"))
    {
        return Err(ProductReceiptError::new(format!(
            "Cargo metadata package `{product_package}` did not contain binary `{product_binary}`"
        )));
    }
    let reachable_packages = reachable_packages(&metadata, &product_package_id)?;

    let mut resolved_package_ids = Vec::with_capacity(runtime_dependencies.len());
    let mut identities = HashSet::with_capacity(runtime_dependencies.len());
    for declaration in &runtime_dependencies {
        let package = package_index
            .get(declaration.package.as_str())
            .copied()
            .ok_or_else(|| {
                ProductReceiptError::new(format!(
                    "Cargo metadata did not contain package `{}`",
                    declaration.package
                ))
            })?;
        if !reachable_packages.contains(package.id.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "runtime dependency package `{}` is not reachable from product package `{product_package}`",
                declaration.package
            )));
        }
        if !package
            .targets
            .iter()
            .any(|target| target.name == declaration.target)
        {
            return Err(ProductReceiptError::new(format!(
                "Cargo metadata package `{}` did not contain runtime target `{}`",
                declaration.package, declaration.target
            )));
        }
        if !identities.insert((package.id.as_str(), declaration.target.as_str())) {
            return Err(ProductReceiptError::new(format!(
                "product build declares more than one runtime artifact for Cargo target `{}` in package `{}`",
                declaration.target, declaration.package
            )));
        }
        resolved_package_ids.push(package.id.clone());
    }
    drop(identities);
    let resolved_dependencies = runtime_dependencies
        .into_iter()
        .zip(resolved_package_ids)
        .map(|(declaration, package_id)| ResolvedRuntimeDependency {
            declaration,
            package_id,
        })
        .collect();
    let cargo_graph_digest = canonical_cargo_graph_digest(&mut metadata, snapshot_root)?;

    Ok(CargoBuildResolution {
        product_package_id,
        product_binary,
        runtime_dependencies: resolved_dependencies,
        cargo_graph_digest,
    })
}

fn canonical_cargo_graph_digest(
    metadata: &mut CargoMetadata,
    snapshot_root: &Path,
) -> Result<String, ProductReceiptError> {
    let mut package_ids = HashMap::with_capacity(metadata.packages.len());
    for package in &mut metadata.packages {
        let (canonical_id, canonical_manifest_path) = canonical_package_id(package, snapshot_root)?;
        if let Some(package_root) = package.manifest_path.parent() {
            for target in &mut package.targets {
                if !target.src_path.as_os_str().is_empty() {
                    target.src_path = PathBuf::from(canonical_package_path(
                        package_root,
                        &target.src_path,
                        "Cargo target source",
                    )?);
                }
            }
        }
        package.manifest_path = canonical_manifest_path;
        for values in package.features.values_mut() {
            values.sort_unstable();
        }
        for target in &mut package.targets {
            target.kind.sort_unstable();
            target.crate_types.sort_unstable();
            target.required_features.sort_unstable();
        }
        package.targets.sort_unstable_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.crate_types.cmp(&right.crate_types))
                .then_with(|| left.required_features.cmp(&right.required_features))
                .then_with(|| left.edition.cmp(&right.edition))
                .then_with(|| left.src_path.cmp(&right.src_path))
        });
        let previous = if let Some(canonical_id) = canonical_id {
            let raw_id = std::mem::replace(&mut package.id, canonical_id);
            package_ids.insert(raw_id, Some(package.id.clone()))
        } else {
            package_ids.insert(package.id.clone(), None)
        };
        if previous.is_some() {
            return Err(ProductReceiptError::new(
                "Cargo metadata contains duplicate package identities",
            ));
        }
    }
    metadata
        .packages
        .sort_unstable_by(|left, right| left.id.cmp(&right.id));
    canonicalize_package_ids(&mut metadata.workspace_members, &package_ids)?;
    canonicalize_package_ids(&mut metadata.workspace_default_members, &package_ids)?;

    if let Some(resolve) = &mut metadata.resolve {
        for node in &mut resolve.nodes {
            canonicalize_package_id_reference_in_place(&mut node.id, &package_ids)?;
            canonicalize_package_ids(&mut node.dependencies, &package_ids)?;
            node.features.sort_unstable();
            for dependency in &mut node.deps {
                canonicalize_package_id_reference_in_place(&mut dependency.pkg, &package_ids)?;
                dependency.dep_kinds.sort_unstable_by(|left, right| {
                    left.kind
                        .cmp(&right.kind)
                        .then_with(|| left.target.cmp(&right.target))
                });
            }
            node.deps.sort_unstable_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.pkg.cmp(&right.pkg))
                    .then_with(|| left.dep_kinds.cmp(&right.dep_kinds))
            });
        }
        resolve
            .nodes
            .sort_unstable_by(|left, right| left.id.cmp(&right.id));
    }
    sha256_serialized(metadata)
}

fn canonical_package_id(
    package: &CargoMetadataPackage,
    snapshot_root: &Path,
) -> Result<(Option<String>, PathBuf), ProductReceiptError> {
    if package.manifest_path.as_os_str().is_empty() {
        return Ok((None, PathBuf::new()));
    }
    if package.source.is_some() {
        let file_name = package.manifest_path.file_name().ok_or_else(|| {
            ProductReceiptError::new("Cargo package manifest path has no file name")
        })?;
        return Ok((None, PathBuf::from(file_name)));
    }
    let manifest = canonical_snapshot_path(snapshot_root, &package.manifest_path)?;
    let canonical_id = format!(
        "path+build-set:///{manifest}#{}@{}",
        package.name, package.version
    );
    Ok((Some(canonical_id), PathBuf::from(manifest)))
}

fn canonical_snapshot_path(
    snapshot_root: &Path,
    path: &Path,
) -> Result<String, ProductReceiptError> {
    let relative = path.strip_prefix(snapshot_root).map_err(|_| {
        ProductReceiptError::new(format!(
            "Cargo path `{}` resolved outside the immutable BuildSet snapshot",
            path.display()
        ))
    })?;
    canonical_relative_path(relative, "Cargo BuildSet path")
}

fn canonical_package_path(
    package_root: &Path,
    path: &Path,
    label: &str,
) -> Result<String, ProductReceiptError> {
    let relative = path.strip_prefix(package_root).map_err(|_| {
        ProductReceiptError::new(format!(
            "{label} `{}` resolved outside its Cargo package",
            path.display()
        ))
    })?;
    canonical_relative_path(relative, label)
}

fn canonical_relative_path(path: &Path, label: &str) -> Result<String, ProductReceiptError> {
    let mut canonical = String::with_capacity(path.as_os_str().len());
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(ProductReceiptError::new(format!(
                "{label} must be a normalized relative path"
            )));
        };
        let component = component
            .to_str()
            .ok_or_else(|| ProductReceiptError::new(format!("{label} is not Unicode")))?;
        if !canonical.is_empty() {
            canonical.push('/');
        }
        canonical.push_str(component);
    }
    if canonical.is_empty() {
        return Err(ProductReceiptError::new(format!(
            "{label} must identify a file"
        )));
    }
    Ok(canonical)
}

fn canonicalize_package_ids(
    ids: &mut [String],
    package_ids: &HashMap<String, Option<String>>,
) -> Result<(), ProductReceiptError> {
    for id in ids.iter_mut() {
        canonicalize_package_id_reference_in_place(id, package_ids)?;
    }
    ids.sort_unstable();
    Ok(())
}

fn canonicalize_package_id_reference_in_place(
    id: &mut String,
    package_ids: &HashMap<String, Option<String>>,
) -> Result<(), ProductReceiptError> {
    let Some(canonical) = package_ids.get(id.as_str()) else {
        return Err(ProductReceiptError::new(format!(
            "Cargo metadata graph references unknown package `{id}`"
        )));
    };
    if let Some(canonical) = canonical {
        if canonical.as_str() != id.as_str() {
            id.clone_from(canonical);
        }
    }
    Ok(())
}

fn reachable_packages<'a>(
    metadata: &'a CargoMetadata,
    product_package_id: &'a str,
) -> Result<HashSet<&'a str>, ProductReceiptError> {
    let resolve = metadata.resolve.as_ref().ok_or_else(|| {
        ProductReceiptError::new("Cargo metadata omitted the resolved dependency graph")
    })?;
    if resolve.nodes.len() > CARGO_METADATA_RESOLVE_NODE_LIMIT {
        return Err(ProductReceiptError::new(format!(
            "Cargo metadata exceeded the {CARGO_METADATA_RESOLVE_NODE_LIMIT}-node resolve limit"
        )));
    }
    let mut nodes = HashMap::with_capacity(resolve.nodes.len());
    let mut edge_count = 0_usize;
    for node in &resolve.nodes {
        if nodes.insert(node.id.as_str(), node).is_some() {
            return Err(ProductReceiptError::new(format!(
                "Cargo metadata resolve graph contains duplicate node `{}`",
                node.id
            )));
        }
        edge_count = edge_count
            .checked_add(node.dependencies.len())
            .ok_or_else(|| {
                ProductReceiptError::new("Cargo metadata resolve edge count overflowed")
            })?;
        if edge_count > CARGO_METADATA_RESOLVE_EDGE_LIMIT {
            return Err(ProductReceiptError::new(format!(
                "Cargo metadata exceeded the {CARGO_METADATA_RESOLVE_EDGE_LIMIT}-edge resolve limit"
            )));
        }
    }
    if !nodes.contains_key(product_package_id) {
        return Err(ProductReceiptError::new(format!(
            "Cargo metadata resolve graph omitted product package `{product_package_id}`"
        )));
    }

    let mut reachable = HashSet::with_capacity(nodes.len());
    let mut pending = VecDeque::with_capacity(nodes.len());
    reachable.insert(product_package_id);
    pending.push_back(product_package_id);
    while let Some(package_id) = pending.pop_front() {
        let node = nodes.get(package_id).ok_or_else(|| {
            ProductReceiptError::new(format!(
                "Cargo metadata resolve graph references missing node `{package_id}`"
            ))
        })?;
        for dependency in &node.dependencies {
            if reachable.insert(dependency.as_str()) {
                pending.push_back(dependency.as_str());
            }
        }
    }
    Ok(reachable)
}

pub(super) fn select_product_artifact(
    messages: impl BufRead,
    expected_package_id: &str,
    expected_binary: &str,
) -> Result<CargoProductArtifact, ProductReceiptError> {
    let resolution = CargoBuildResolution {
        product_package_id: expected_package_id.to_string(),
        product_binary: expected_binary,
        runtime_dependencies: Vec::new(),
        cargo_graph_digest: String::new(),
    };
    let (product, _) = select_build_artifacts(messages, &resolution, Ok, |_| Ok(()))?;
    Ok(product)
}

pub(super) fn select_build_artifacts<'a, P, D>(
    mut messages: impl BufRead,
    resolution: &'a CargoBuildResolution<'_>,
    mut capture_product: impl FnMut(CargoProductArtifact) -> Result<P, ProductReceiptError>,
    mut capture_dependency: impl FnMut(CargoRuntimeArtifact<'a>) -> Result<D, ProductReceiptError>,
) -> Result<(P, Vec<D>), ProductReceiptError> {
    let mut dependency_index = HashMap::with_capacity(resolution.runtime_dependencies.len());
    for (index, dependency) in resolution.runtime_dependencies.iter().enumerate() {
        dependency_index.insert(
            (
                dependency.package_id.as_str(),
                dependency.declaration.target.as_str(),
            ),
            index,
        );
    }
    let mut line = Vec::new();
    let mut selected_product = None;
    let mut selected_dependencies = std::iter::repeat_with(|| None)
        .take(resolution.runtime_dependencies.len())
        .collect::<Vec<Option<D>>>();
    let mut message_count = 0_usize;
    let mut build_finished = None;

    loop {
        line.clear();
        if !read_bounded_message_line(&mut messages, &mut line)? {
            break;
        }
        message_count = message_count
            .checked_add(1)
            .ok_or_else(|| ProductReceiptError::new("Cargo JSON message count overflowed"))?;
        if message_count > CARGO_JSON_MESSAGE_COUNT_LIMIT {
            return Err(ProductReceiptError::new(format!(
                "Cargo JSON stream exceeded the {CARGO_JSON_MESSAGE_COUNT_LIMIT}-message limit"
            )));
        }

        let message_bytes = trim_line_ending(&line);
        let message: CargoMessageHeader<'_> =
            serde_json::from_slice(message_bytes).map_err(|error| {
                ProductReceiptError::new(format!("could not parse Cargo JSON message: {error}"))
            })?;
        match message.reason {
            "compiler-artifact" => {
                let package_id = message.package_id.ok_or_else(|| {
                    ProductReceiptError::new("Cargo compiler-artifact omitted package_id")
                })?;
                let target = message.target.as_ref().ok_or_else(|| {
                    ProductReceiptError::new("Cargo compiler-artifact omitted target")
                })?;
                if package_id == resolution.product_package_id
                    && target.name == resolution.product_binary
                    && target.is_binary
                {
                    if selected_product.is_some() {
                        return Err(ProductReceiptError::new(
                            "Cargo emitted more than one matching product executable",
                        ));
                    }
                    selected_product = Some(capture_product(product_artifact(
                        parse_cargo_artifact_payload(message_bytes)?,
                    )?)?);
                    continue;
                }

                let Some(index) = dependency_index.get(&(package_id, target.name)).copied() else {
                    continue;
                };
                if selected_dependencies[index].is_some() {
                    let declaration = &resolution.runtime_dependencies[index].declaration;
                    return Err(ProductReceiptError::new(format!(
                        "Cargo emitted more than one artifact for runtime dependency `{}`",
                        declaration.logical_name
                    )));
                }
                let declaration = &resolution.runtime_dependencies[index].declaration;
                let payload = parse_cargo_artifact_payload(message_bytes)?;
                let mut paths = payload.filenames.into_iter().filter(|path| {
                    path.file_name() == Some(OsStr::new(&declaration.artifact_file_name))
                });
                let source_path = paths.next().ok_or_else(|| {
                    ProductReceiptError::new(format!(
                        "Cargo artifact for runtime dependency `{}` omitted `{}`",
                        declaration.logical_name, declaration.artifact_file_name
                    ))
                })?;
                if paths.next().is_some() {
                    return Err(ProductReceiptError::new(format!(
                        "Cargo emitted duplicate files named `{}` for runtime dependency `{}`",
                        declaration.artifact_file_name, declaration.logical_name
                    )));
                }
                selected_dependencies[index] = Some(capture_dependency(CargoRuntimeArtifact {
                    declaration,
                    source_path,
                })?);
            }
            "build-finished" => {
                if build_finished.replace(message.success).is_some() {
                    return Err(ProductReceiptError::new(
                        "Cargo emitted more than one build-finished message",
                    ));
                }
            }
            _ => {}
        }
    }

    require_successful_finish(build_finished)?;
    let product = selected_product.ok_or_else(|| {
        ProductReceiptError::new(format!(
            "Cargo did not emit an executable for package `{}` binary `{}`",
            resolution.product_package_id, resolution.product_binary
        ))
    })?;
    let mut dependencies = Vec::with_capacity(selected_dependencies.len());
    for (index, selected) in selected_dependencies.into_iter().enumerate() {
        dependencies.push(selected.ok_or_else(|| {
            ProductReceiptError::new(format!(
                "Cargo did not emit runtime dependency `{}`",
                resolution.runtime_dependencies[index]
                    .declaration
                    .logical_name
            ))
        })?);
    }
    Ok((product, dependencies))
}

fn selected_package_index<'a>(
    metadata: &'a CargoMetadata,
    selected_names: &HashSet<&str>,
) -> Result<HashMap<&'a str, &'a CargoMetadataPackage>, ProductReceiptError> {
    let mut packages = HashMap::with_capacity(selected_names.len());
    for package in &metadata.packages {
        if !selected_names.contains(package.name.as_str()) {
            continue;
        }
        if packages.insert(package.name.as_str(), package).is_some() {
            return Err(ProductReceiptError::new(format!(
                "Cargo metadata contained more than one package named `{}`",
                package.name
            )));
        }
    }
    for name in selected_names {
        if !packages.contains_key(name) {
            return Err(ProductReceiptError::new(format!(
                "Cargo metadata did not contain package `{name}`"
            )));
        }
    }
    Ok(packages)
}

fn parse_cargo_artifact_payload(
    message_bytes: &[u8],
) -> Result<CargoArtifactPayload, ProductReceiptError> {
    serde_json::from_slice(message_bytes).map_err(|error| {
        ProductReceiptError::new(format!("could not parse selected Cargo artifact: {error}"))
    })
}

fn product_artifact(
    payload: CargoArtifactPayload,
) -> Result<CargoProductArtifact, ProductReceiptError> {
    let executable = payload.executable.ok_or_else(|| {
        ProductReceiptError::new("selected Cargo compiler-artifact omitted its executable path")
    })?;
    let mut executable_found = false;
    let mut symbol_files = Vec::with_capacity(payload.filenames.len());
    for path in payload.filenames {
        if path == executable {
            executable_found = true;
        }
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdb"))
        {
            symbol_files.push(path);
        }
    }
    if !executable_found {
        return Err(ProductReceiptError::new(
            "selected Cargo executable was not present in its artifact filename set",
        ));
    }
    symbol_files.sort_unstable();
    symbol_files.dedup();
    Ok(CargoProductArtifact {
        executable,
        symbol_files,
    })
}

fn require_successful_finish(
    build_finished: Option<Option<bool>>,
) -> Result<(), ProductReceiptError> {
    match build_finished {
        Some(Some(true)) => Ok(()),
        Some(Some(false)) => Err(ProductReceiptError::new(
            "Cargo reported that the product build did not finish successfully",
        )),
        Some(None) => Err(ProductReceiptError::new(
            "Cargo build-finished message omitted its success disposition",
        )),
        None => Err(ProductReceiptError::new(
            "Cargo did not emit a build-finished message",
        )),
    }
}

fn read_bounded_message_line(
    reader: &mut impl BufRead,
    line: &mut Vec<u8>,
) -> Result<bool, ProductReceiptError> {
    loop {
        let available = reader.fill_buf().map_err(|error| {
            ProductReceiptError::new(format!("could not read Cargo JSON message stream: {error}"))
        })?;
        if available.is_empty() {
            return Ok(!line.is_empty());
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(consumed) > CARGO_JSON_MESSAGE_LINE_LIMIT {
            return Err(ProductReceiptError::new(format!(
                "Cargo JSON message exceeded the {CARGO_JSON_MESSAGE_LINE_LIMIT}-byte line limit"
            )));
        }
        line.extend_from_slice(&available[..consumed]);
        reader.consume(consumed);
        if newline.is_some() {
            return Ok(true);
        }
    }
}

fn trim_line_ending(mut line: &[u8]) -> &[u8] {
    if let Some(without_newline) = line.strip_suffix(b"\n") {
        line = without_newline;
    }
    line.strip_suffix(b"\r").unwrap_or(line)
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;
