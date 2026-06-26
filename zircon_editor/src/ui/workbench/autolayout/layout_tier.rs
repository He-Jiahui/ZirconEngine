use zircon_runtime_interface::ui::design_tokens::EditorDensityTokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkbenchLayoutTier {
    Ultra,
    Narrow,
    Regular,
    Wide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkbenchBreakpointDefaults {
    pub ultra_max_width: f32,
    pub narrow_max_width: f32,
    pub wide_min_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkbenchCompactSideDefaults {
    pub available_width: f32,
    pub ultra_available_width: f32,
    pub left_max_width: f32,
    pub right_max_width: f32,
    pub side_min_width: f32,
    pub ultra_left_max_width: f32,
    pub ultra_right_max_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkbenchCompactBottomDefaults {
    pub available_height: f32,
    pub max_height: f32,
    pub max_available_fraction: f32,
    pub min_height: f32,
    pub ultra_available_height: f32,
    pub ultra_max_height: f32,
    pub ultra_max_available_fraction: f32,
    pub ultra_min_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkbenchWindowMinimumDefaults {
    pub min_width: f32,
    pub min_height: f32,
    pub ultra_min_width: f32,
    pub ultra_min_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkbenchLayoutDefaults {
    pub breakpoints: WorkbenchBreakpointDefaults,
    pub compact_side: WorkbenchCompactSideDefaults,
    pub compact_bottom: WorkbenchCompactBottomDefaults,
    pub window_minimums: WorkbenchWindowMinimumDefaults,
}

impl WorkbenchLayoutDefaults {
    pub(crate) fn from_density_tokens(tokens: EditorDensityTokens) -> Self {
        Self {
            breakpoints: WorkbenchBreakpointDefaults {
                ultra_max_width: tokens.breakpoint_ultra_width,
                narrow_max_width: tokens.breakpoint_narrow_width,
                wide_min_width: tokens.breakpoint_wide_width,
            },
            compact_side: WorkbenchCompactSideDefaults {
                available_width: tokens.compact_side_width,
                ultra_available_width: tokens.ultra_compact_side_width,
                left_max_width: tokens.compact_left_drawer_max_width,
                right_max_width: tokens.compact_right_drawer_max_width,
                side_min_width: tokens.compact_side_min_width,
                ultra_left_max_width: tokens.ultra_compact_left_drawer_max_width,
                ultra_right_max_width: tokens.ultra_compact_right_drawer_max_width,
            },
            compact_bottom: WorkbenchCompactBottomDefaults {
                available_height: tokens.compact_bottom_available_height,
                max_height: tokens.compact_bottom_max_height,
                max_available_fraction: tokens.compact_bottom_max_available_fraction,
                min_height: tokens.compact_bottom_min_height,
                ultra_available_height: tokens.ultra_compact_bottom_available_height,
                ultra_max_height: tokens.ultra_compact_bottom_max_height,
                ultra_max_available_fraction: tokens.ultra_compact_bottom_max_available_fraction,
                ultra_min_height: tokens.ultra_compact_bottom_min_height,
            },
            window_minimums: WorkbenchWindowMinimumDefaults {
                min_width: tokens.minimum_window_width,
                min_height: tokens.minimum_window_height,
                ultra_min_width: tokens.ultra_minimum_window_width,
                ultra_min_height: tokens.ultra_minimum_window_height,
            },
        }
    }
}

pub(crate) fn workbench_layout_defaults() -> WorkbenchLayoutDefaults {
    WorkbenchLayoutDefaults::from_density_tokens(EditorDensityTokens::workbench_dense())
}

pub(crate) fn workbench_layout_tier_for_width(width: f32) -> WorkbenchLayoutTier {
    let width = width.max(0.0);
    let defaults = workbench_layout_defaults().breakpoints;
    if width <= defaults.ultra_max_width {
        WorkbenchLayoutTier::Ultra
    } else if width <= defaults.narrow_max_width {
        WorkbenchLayoutTier::Narrow
    } else if width >= defaults.wide_min_width {
        WorkbenchLayoutTier::Wide
    } else {
        WorkbenchLayoutTier::Regular
    }
}

pub(crate) fn right_drawer_should_collapse_for_width(width: f32) -> bool {
    matches!(
        workbench_layout_tier_for_width(width),
        WorkbenchLayoutTier::Ultra | WorkbenchLayoutTier::Narrow
    )
}

pub(crate) fn compact_side_defaults() -> WorkbenchCompactSideDefaults {
    workbench_layout_defaults().compact_side
}

pub(crate) fn compact_bottom_defaults() -> WorkbenchCompactBottomDefaults {
    workbench_layout_defaults().compact_bottom
}

pub(crate) fn window_min_width_limit_for_width(width: f32) -> f32 {
    let defaults = workbench_layout_defaults();
    match workbench_layout_tier_for_width(width) {
        WorkbenchLayoutTier::Ultra => defaults.window_minimums.ultra_min_width,
        WorkbenchLayoutTier::Narrow | WorkbenchLayoutTier::Regular | WorkbenchLayoutTier::Wide => {
            defaults.window_minimums.min_width
        }
    }
}

pub(crate) fn window_min_height_limit_for_height(height: f32) -> f32 {
    let height = height.max(0.0);
    let defaults = workbench_layout_defaults();
    if height <= defaults.window_minimums.min_height {
        defaults.window_minimums.ultra_min_height
    } else {
        defaults.window_minimums.min_height
    }
}
