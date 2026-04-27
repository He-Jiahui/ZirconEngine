use crate::ui::workbench::snapshot::ViewContentKind;

use super::super::{PaneConstraints, ShellRegionId};
use super::axis_factory::stretch_axis;

pub fn default_region_constraints(region: ShellRegionId) -> PaneConstraints {
    match region {
        ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
            width: stretch_axis(240.0, 308.0, 50, 1.0),
            height: stretch_axis(180.0, 320.0, 50, 1.0),
        },
        ShellRegionId::Bottom => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(120.0, 164.0, 50, 1.0),
        },
        ShellRegionId::Document => PaneConstraints {
            width: stretch_axis(520.0, 960.0, 100, 3.0),
            height: stretch_axis(280.0, 640.0, 100, 3.0),
        },
    }
}

pub fn default_constraints_for_content(kind: ViewContentKind) -> PaneConstraints {
    match kind {
        ViewContentKind::Welcome => default_region_constraints(ShellRegionId::Document),
        ViewContentKind::Scene | ViewContentKind::Game | ViewContentKind::PrefabEditor => {
            PaneConstraints {
                width: stretch_axis(640.0, 1080.0, 100, 4.0),
                height: stretch_axis(360.0, 720.0, 100, 4.0),
            }
        }
        ViewContentKind::Inspector => PaneConstraints {
            width: stretch_axis(260.0, 312.0, 60, 1.0),
            height: stretch_axis(220.0, 360.0, 60, 1.0),
        },
        ViewContentKind::Hierarchy | ViewContentKind::Project => PaneConstraints {
            width: stretch_axis(220.0, 280.0, 55, 1.0),
            height: stretch_axis(180.0, 320.0, 55, 1.0),
        },
        ViewContentKind::Console
        | ViewContentKind::RuntimeDiagnostics
        | ViewContentKind::ModulePlugins => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(140.0, 200.0, 50, 1.0),
        },
        ViewContentKind::Assets
        | ViewContentKind::AssetBrowser
        | ViewContentKind::UiAssetEditor
        | ViewContentKind::UiComponentShowcase
        | ViewContentKind::AnimationSequenceEditor
        | ViewContentKind::AnimationGraphEditor => PaneConstraints {
            width: stretch_axis(420.0, 720.0, 80, 2.0),
            height: stretch_axis(260.0, 480.0, 80, 2.0),
        },
        ViewContentKind::Placeholder => default_region_constraints(ShellRegionId::Document),
    }
}
