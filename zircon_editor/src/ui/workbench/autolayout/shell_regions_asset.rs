use std::collections::BTreeSet;

use serde::Deserialize;
use thiserror::Error;

use super::workbench_skeleton::WorkbenchSkeleton;
use super::{EditorRegion, EditorRegionRole, RegionBinding, WorkbenchConstraintTokenName};

pub const WORKBENCH_SHELL_REGIONS_ASSET_KIND: &str = "layout_regions";
pub const WORKBENCH_SHELL_REGIONS_ASSET_ID: &str = "zircon.editor.workbench.shell_regions";
pub const WORKBENCH_SHELL_REGIONS_ASSET_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbenchShellRegionsAsset {
    pub header: WorkbenchShellRegionsAssetHeader,
    pub regions: Vec<RegionBinding>,
}

impl WorkbenchShellRegionsAsset {
    pub fn from_toml_str(source: &str) -> Result<Self, WorkbenchShellRegionsAssetError> {
        let raw: RawWorkbenchShellRegionsAsset = toml::from_str(source)?;
        validate_header(&raw.asset)?;
        let regions = raw
            .regions
            .into_iter()
            .map(RawRegionBinding::into_region_binding)
            .collect::<Result<Vec<_>, _>>()?;
        validate_complete_region_set(&regions)?;

        Ok(Self {
            header: raw.asset,
            regions,
        })
    }

    pub fn into_regions(self) -> Vec<RegionBinding> {
        self.regions
    }
}

impl WorkbenchSkeleton {
    pub fn from_shell_regions_asset_str(
        source: &str,
    ) -> Result<Self, WorkbenchShellRegionsAssetError> {
        Ok(Self::from_shell_regions_asset(
            WorkbenchShellRegionsAsset::from_toml_str(source)?,
        ))
    }

    pub fn from_shell_regions_asset(asset: WorkbenchShellRegionsAsset) -> Self {
        let mut skeleton = Self::jetbrains_default();
        skeleton.regions = asset.into_regions();
        skeleton
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WorkbenchShellRegionsAssetHeader {
    pub kind: String,
    pub id: String,
    pub version: u32,
    pub display_name: String,
}

#[derive(Debug, Error)]
pub enum WorkbenchShellRegionsAssetError {
    #[error("failed to parse workbench shell regions TOML asset: {0}")]
    ParseToml(#[from] toml::de::Error),
    #[error("unexpected shell regions asset kind `{actual}`")]
    UnexpectedKind { actual: String },
    #[error("unexpected shell regions asset id `{actual}`")]
    UnexpectedId { actual: String },
    #[error("unsupported shell regions asset version `{actual}`")]
    UnsupportedVersion { actual: u32 },
    #[error("duplicate workbench shell region `{region:?}`")]
    DuplicateRegion { region: EditorRegion },
    #[error("missing workbench shell region `{region:?}`")]
    MissingRegion { region: EditorRegion },
    #[error(
        "workbench shell region `{region:?}` expected role `{expected_role:?}` but asset declared `{actual_role:?}`"
    )]
    RoleMismatch {
        region: EditorRegion,
        expected_role: EditorRegionRole,
        actual_role: EditorRegionRole,
    },
}

#[derive(Debug, Deserialize)]
struct RawWorkbenchShellRegionsAsset {
    asset: WorkbenchShellRegionsAssetHeader,
    regions: Vec<RawRegionBinding>,
}

#[derive(Debug, Deserialize)]
struct RawRegionBinding {
    region: EditorRegion,
    role: EditorRegionRole,
    panel_asset: String,
    size_token: Option<WorkbenchConstraintTokenName>,
}

impl RawRegionBinding {
    fn into_region_binding(self) -> Result<RegionBinding, WorkbenchShellRegionsAssetError> {
        RegionBinding::new(self.region, self.role, self.panel_asset, self.size_token).map_err(
            |error| WorkbenchShellRegionsAssetError::RoleMismatch {
                region: error.region(),
                expected_role: error.expected_role(),
                actual_role: error.actual_role(),
            },
        )
    }
}

fn validate_header(
    header: &WorkbenchShellRegionsAssetHeader,
) -> Result<(), WorkbenchShellRegionsAssetError> {
    if header.kind != WORKBENCH_SHELL_REGIONS_ASSET_KIND {
        return Err(WorkbenchShellRegionsAssetError::UnexpectedKind {
            actual: header.kind.clone(),
        });
    }
    if header.id != WORKBENCH_SHELL_REGIONS_ASSET_ID {
        return Err(WorkbenchShellRegionsAssetError::UnexpectedId {
            actual: header.id.clone(),
        });
    }
    if header.version != WORKBENCH_SHELL_REGIONS_ASSET_VERSION {
        return Err(WorkbenchShellRegionsAssetError::UnsupportedVersion {
            actual: header.version,
        });
    }
    Ok(())
}

fn validate_complete_region_set(
    regions: &[RegionBinding],
) -> Result<(), WorkbenchShellRegionsAssetError> {
    let mut seen = BTreeSet::new();
    for binding in regions {
        if !seen.insert(binding.region) {
            return Err(WorkbenchShellRegionsAssetError::DuplicateRegion {
                region: binding.region,
            });
        }
    }
    for region in EditorRegion::ALL {
        if !seen.contains(&region) {
            return Err(WorkbenchShellRegionsAssetError::MissingRegion { region });
        }
    }
    Ok(())
}
