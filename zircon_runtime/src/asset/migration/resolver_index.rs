use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;

use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::asset::reference_resolver::ProjectSourceLookup;
use crate::asset::{AssetUri, ReferenceResolutionError};

/// One safe regular-file projection published by the migration inventory generation.
///
/// Paths are already canonicalized and link/reparse entries are already rejected by the scan
/// owner. This type deliberately performs no filesystem validation of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MigrationSourceProjection {
    logical_root: RelPath,
    physical_root: PathBuf,
    root_relative: RelPath,
    physical_path: PathBuf,
}

impl MigrationSourceProjection {
    pub(crate) fn new(
        logical_root: RelPath,
        physical_root: PathBuf,
        root_relative: RelPath,
        physical_path: PathBuf,
    ) -> Self {
        Self {
            logical_root,
            physical_root,
            root_relative,
            physical_path,
        }
    }
}

/// A compound locator already validated against its parsed sidecar document by preflight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MigrationCompoundBinding {
    locator: AssetUri,
    physical_path: PathBuf,
    source_relative: String,
    target_relative: String,
}

impl MigrationCompoundBinding {
    pub(crate) fn new(locator: AssetUri, physical_path: PathBuf) -> Self {
        let target_relative = compound_sidecar_relative_path(&locator, ".zmeta");
        Self {
            locator,
            physical_path,
            source_relative: target_relative.clone(),
            target_relative,
        }
    }

    /// Inventory still names the v6 file, while migrated authoring references name its v7 target.
    pub(crate) fn from_retired_meta_toml(locator: AssetUri, physical_path: PathBuf) -> Self {
        Self {
            source_relative: compound_sidecar_relative_path(&locator, ".meta.toml"),
            target_relative: compound_sidecar_relative_path(&locator, ".zmeta"),
            locator,
            physical_path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PersistedSourceIdentity {
    physical_path: PathBuf,
    physical_root: PathBuf,
    logical_root: RelPath,
    root_relative: RelPath,
    project_hint: RelPath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PersistedHintIdentity {
    locator: AssetUri,
    physical_path: PathBuf,
    physical_root: PathBuf,
}

/// Ephemeral source identity index owned by one migration generation.
///
/// Registry metadata remains the logical asset authority. This index owns only the physical and
/// project-root projections needed to resolve persisted authoring references without probing the
/// filesystem for every reference.
#[derive(Clone, Debug, Default)]
pub(crate) struct MigrationResolverIndex {
    by_locator: HashMap<AssetUri, Vec<PersistedSourceIdentity>>,
    by_project_hint: HashMap<RelPath, Vec<PersistedHintIdentity>>,
    lookup_count: Cell<usize>,
}

impl MigrationResolverIndex {
    pub(crate) fn build(
        projections: impl IntoIterator<Item = MigrationSourceProjection>,
        compound_bindings: impl IntoIterator<Item = MigrationCompoundBinding>,
    ) -> Result<Self, ReferenceResolutionError> {
        let projections = projections.into_iter().collect::<Vec<_>>();
        let compound_bindings = compound_bindings.into_iter();
        let index_capacity = projections
            .len()
            .saturating_add(compound_bindings.size_hint().0);
        let mut projections_by_physical_path: HashMap<PathBuf, Vec<&MigrationSourceProjection>> =
            HashMap::with_capacity(projections.len());
        let mut index = Self {
            by_locator: HashMap::with_capacity(index_capacity),
            by_project_hint: HashMap::with_capacity(index_capacity),
            lookup_count: Cell::new(0),
        };

        for projection in &projections {
            projections_by_physical_path
                .entry(projection.physical_path.clone())
                .or_default()
                .push(projection);
            let locator = AssetUri::new(
                ResourceScheme::Res,
                projection.root_relative.as_str().to_owned(),
                None,
            )
            .map_err(|error| ReferenceResolutionError::Registry {
                message: error.to_string(),
            })?;
            index.insert(locator, projection, false, None)?;
        }

        for binding in compound_bindings {
            if binding.locator.scheme() != ResourceScheme::Res || binding.locator.label().is_some()
            {
                continue;
            }
            let Some(candidates) = projections_by_physical_path.get(&binding.physical_path) else {
                continue;
            };
            for projection in candidates {
                if projection.root_relative.as_str() == binding.source_relative {
                    index.insert(
                        binding.locator.clone(),
                        projection,
                        true,
                        Some(binding.target_relative.as_str()),
                    )?;
                }
            }
        }

        index.finish();
        Ok(index)
    }

    pub(crate) fn project_hint_for_locator(
        &self,
        locator: &AssetUri,
    ) -> Result<RelPath, ReferenceResolutionError> {
        self.record_lookup();
        let base_locator = base_project_locator(locator)?;
        match self
            .by_locator
            .get(base_locator.as_ref())
            .map(Vec::as_slice)
        {
            None | Some([]) => Err(ReferenceResolutionError::MissingPath {
                path: locator.to_string(),
            }),
            Some([identity]) => Ok(identity.project_hint.clone()),
            Some(_) => Err(ReferenceResolutionError::AmbiguousPath {
                path: locator.to_string(),
            }),
        }
    }

    pub(crate) fn locator_for_project_hint(
        &self,
        hint: &RelPath,
    ) -> Result<Option<AssetUri>, ReferenceResolutionError> {
        self.record_lookup();
        match self.by_project_hint.get(hint).map(Vec::as_slice) {
            None | Some([]) => Ok(None),
            Some([identity]) => Ok(Some(identity.locator.clone())),
            Some(_) => Err(ReferenceResolutionError::AmbiguousPath {
                path: hint.to_string(),
            }),
        }
    }

    pub(crate) fn lookup_count(&self) -> usize {
        self.lookup_count.get()
    }

    fn record_lookup(&self) {
        self.lookup_count
            .set(self.lookup_count.get().saturating_add(1));
    }

    fn insert(
        &mut self,
        locator: AssetUri,
        projection: &MigrationSourceProjection,
        compound_hint: bool,
        target_relative: Option<&str>,
    ) -> Result<(), ReferenceResolutionError> {
        let locator = into_base_project_locator(locator)?;
        let target_relative = target_relative.unwrap_or(projection.root_relative.as_str());
        let project_hint = RelPath::parse(format!(
            "{}/{}",
            projection.logical_root.as_str(),
            target_relative
        ))
        .map_err(|source| ReferenceResolutionError::Path {
            path: locator.to_string(),
            source,
        })?;
        let compound_is_shadowed_by_direct_source = compound_hint
            && self.by_locator.get(&locator).is_some_and(|identities| {
                identities.iter().any(|identity| {
                    identity.physical_root == projection.physical_root
                        && identity.logical_root == projection.logical_root
                        && identity.root_relative.as_str() == locator.path()
                })
            });
        if !compound_is_shadowed_by_direct_source {
            self.by_locator
                .entry(locator.clone())
                .or_default()
                .push(PersistedSourceIdentity {
                    physical_path: projection.physical_path.clone(),
                    physical_root: projection.physical_root.clone(),
                    logical_root: projection.logical_root.clone(),
                    root_relative: projection.root_relative.clone(),
                    project_hint: project_hint.clone(),
                });
        }
        let hints = self.by_project_hint.entry(project_hint).or_default();
        if compound_hint {
            // The filesystem resolver recognizes a validated compound locator before treating its
            // `.zmeta` file as a direct asset source. Preserve that reverse-lookup priority while
            // retaining both locator keys for forward lookup.
            hints.retain(|identity| {
                identity.physical_path != projection.physical_path
                    || identity.physical_root != projection.physical_root
                    || identity.locator == locator
            });
        }
        hints.push(PersistedHintIdentity {
            locator,
            physical_path: projection.physical_path.clone(),
            physical_root: projection.physical_root.clone(),
        });
        Ok(())
    }

    fn finish(&mut self) {
        for identities in self.by_locator.values_mut() {
            sort_locator_identities(identities);
        }
        for identities in self.by_project_hint.values_mut() {
            sort_hint_identities(identities);
        }
    }
}

fn sort_locator_identities(identities: &mut Vec<PersistedSourceIdentity>) {
    identities.sort_unstable_by(|left, right| {
        left.project_hint
            .cmp(&right.project_hint)
            .then_with(|| left.physical_root.cmp(&right.physical_root))
            .then_with(|| left.physical_path.cmp(&right.physical_path))
            .then_with(|| left.logical_root.cmp(&right.logical_root))
            .then_with(|| left.root_relative.cmp(&right.root_relative))
    });
    identities.dedup_by(|left, right| {
        left.project_hint == right.project_hint
            && left.physical_root == right.physical_root
            && left.physical_path == right.physical_path
            && left.logical_root == right.logical_root
            && left.root_relative == right.root_relative
    });
}

fn sort_hint_identities(identities: &mut Vec<PersistedHintIdentity>) {
    identities.sort_unstable_by(|left, right| {
        left.locator
            .cmp(&right.locator)
            .then_with(|| left.physical_root.cmp(&right.physical_root))
            .then_with(|| left.physical_path.cmp(&right.physical_path))
    });
    identities.dedup_by(|left, right| {
        left.locator == right.locator
            && left.physical_root == right.physical_root
            && left.physical_path == right.physical_path
    });
}

impl ProjectSourceLookup for MigrationResolverIndex {
    fn project_hint_for_locator(
        &self,
        locator: &AssetUri,
    ) -> Result<RelPath, ReferenceResolutionError> {
        MigrationResolverIndex::project_hint_for_locator(self, locator)
    }

    fn locator_for_project_hint(
        &self,
        hint: &RelPath,
    ) -> Result<Option<AssetUri>, ReferenceResolutionError> {
        MigrationResolverIndex::locator_for_project_hint(self, hint)
    }
}

fn base_project_locator(locator: &AssetUri) -> Result<Cow<'_, AssetUri>, ReferenceResolutionError> {
    if locator.scheme() != ResourceScheme::Res {
        return Err(ReferenceResolutionError::UnsupportedScheme {
            locator: locator.clone(),
        });
    }
    if locator.label().is_none() {
        return Ok(Cow::Borrowed(locator));
    }
    AssetUri::new(ResourceScheme::Res, locator.path().to_owned(), None)
        .map(Cow::Owned)
        .map_err(|error| ReferenceResolutionError::Registry {
            message: error.to_string(),
        })
}

fn into_base_project_locator(locator: AssetUri) -> Result<AssetUri, ReferenceResolutionError> {
    if locator.scheme() != ResourceScheme::Res {
        return Err(ReferenceResolutionError::UnsupportedScheme { locator });
    }
    if locator.label().is_none() {
        return Ok(locator);
    }
    AssetUri::new(ResourceScheme::Res, locator.path().to_owned(), None).map_err(|error| {
        ReferenceResolutionError::Registry {
            message: error.to_string(),
        }
    })
}

fn compound_sidecar_relative_path(locator: &AssetUri, suffix: &str) -> String {
    format!("{}{suffix}", locator.path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_project_locator_borrows_the_common_unlabeled_key() {
        let locator = AssetUri::parse("res://textures/albedo.ztexture").unwrap();
        let base = base_project_locator(&locator).unwrap();

        assert!(matches!(base, Cow::Borrowed(value) if std::ptr::eq(value, &locator)));
        assert_eq!(into_base_project_locator(locator.clone()).unwrap(), locator);
    }

    #[test]
    fn base_project_locator_owns_only_the_label_stripped_key() {
        let locator = AssetUri::parse("res://models/hero.glb#Mesh0").unwrap();
        let expected = AssetUri::parse("res://models/hero.glb").unwrap();

        assert_eq!(base_project_locator(&locator).unwrap().as_ref(), &expected);
        assert_eq!(into_base_project_locator(locator).unwrap(), expected);
    }
}

#[cfg(test)]
#[path = "resolver_index/optimization_tests.rs"]
mod optimization_tests;
