use std::{fs, path::PathBuf};

const MUI_MONOREPO_VERSION: &str = "\"version\": \"9.0.1\"";
const MUI_MATERIAL_ICON_COUNT: usize = 10_751;

const REQUIRED_MUI_WEB_THEME_LINES: &[&str] = &[
    "mui_web_source_version = \"9.0.1\"",
    "mui_web_source_package = \"dev/material-ui\"",
    "mui_web_palette_default_mode = \"light\"",
    "mui_web_palette_primary_main = \"#1976d2\"",
    "mui_web_palette_primary_dark_mode_main = \"#90caf9\"",
    "mui_web_palette_secondary_main = \"#9c27b0\"",
    "mui_web_palette_error_main = \"#d32f2f\"",
    "mui_web_palette_warning_main = \"#ed6c02\"",
    "mui_web_palette_info_main = \"#0288d1\"",
    "mui_web_palette_success_main = \"#2e7d32\"",
    "mui_web_palette_dark_background_default = \"#121212\"",
    "mui_web_palette_light_text_primary = \"rgba(0, 0, 0, 0.87)\"",
    "mui_web_palette_dark_text_secondary = \"rgba(255, 255, 255, 0.7)\"",
    "mui_web_action_light_hover_opacity = 0.04",
    "mui_web_action_dark_selected_opacity = 0.16",
    "mui_web_action_disabled_opacity = 0.38",
    "mui_web_shape_border_radius = 4.0",
    "mui_web_spacing_unit = 8.0",
    "mui_web_typography_font_size = 14.0",
    "mui_web_typography_font_weight_medium = 500.0",
    "mui_web_shadow_key_umbra_opacity = 0.20",
    "mui_web_shadow_1 = \"0px 2px 1px -1px rgba(0,0,0,0.2),0px 1px 1px 0px rgba(0,0,0,0.14),0px 1px 3px 0px rgba(0,0,0,0.12)\"",
    "mui_web_z_index_modal = 1300.0",
    "mui_web_z_index_tooltip = 1500.0",
    "mui_web_transition_ease_in_out = \"cubic-bezier(0.4, 0, 0.2, 1)\"",
    "mui_web_transition_duration_standard = 300.0",
];

const EXPECTED_MUI_MATERIAL_DEFAULT_EXPORTS: &[&str] = &[
    "Accordion",
    "AccordionActions",
    "AccordionDetails",
    "AccordionSummary",
    "Alert",
    "AlertTitle",
    "AppBar",
    "Autocomplete",
    "Avatar",
    "AvatarGroup",
    "Backdrop",
    "Badge",
    "BottomNavigation",
    "BottomNavigationAction",
    "Box",
    "Breadcrumbs",
    "Button",
    "ButtonBase",
    "ButtonGroup",
    "Card",
    "CardActionArea",
    "CardActions",
    "CardContent",
    "CardHeader",
    "CardMedia",
    "Checkbox",
    "Chip",
    "CircularProgress",
    "ClickAwayListener",
    "Collapse",
    "Container",
    "CssBaseline",
    "darkScrollbar",
    "Dialog",
    "DialogActions",
    "DialogContent",
    "DialogContentText",
    "DialogTitle",
    "Divider",
    "Drawer",
    "Fab",
    "Fade",
    "FilledInput",
    "FormControl",
    "FormControlLabel",
    "FormGroup",
    "FormHelperText",
    "FormLabel",
    "Grid",
    "Grow",
    "Icon",
    "IconButton",
    "ImageList",
    "ImageListItem",
    "ImageListItemBar",
    "Input",
    "InputAdornment",
    "InputBase",
    "InputLabel",
    "LinearProgress",
    "Link",
    "List",
    "ListItem",
    "ListItemAvatar",
    "ListItemButton",
    "ListItemIcon",
    "ListItemSecondaryAction",
    "ListItemText",
    "ListSubheader",
    "Menu",
    "MenuItem",
    "MenuList",
    "MobileStepper",
    "Modal",
    "NativeSelect",
    "NoSsr",
    "OutlinedInput",
    "Pagination",
    "PaginationItem",
    "Paper",
    "Popover",
    "Popper",
    "Portal",
    "Radio",
    "RadioGroup",
    "Rating",
    "ScopedCssBaseline",
    "Select",
    "Skeleton",
    "Slide",
    "Slider",
    "Snackbar",
    "SnackbarContent",
    "SpeedDial",
    "SpeedDialAction",
    "SpeedDialIcon",
    "Stack",
    "Step",
    "StepButton",
    "StepConnector",
    "StepContent",
    "StepIcon",
    "StepLabel",
    "Stepper",
    "SvgIcon",
    "SwipeableDrawer",
    "Switch",
    "Tab",
    "Table",
    "TableBody",
    "TableCell",
    "TableContainer",
    "TableFooter",
    "TableHead",
    "TablePagination",
    "TablePaginationActions",
    "TableRow",
    "TableSortLabel",
    "Tabs",
    "TabScrollButton",
    "TextField",
    "TextareaAutosize",
    "ToggleButton",
    "ToggleButtonGroup",
    "Toolbar",
    "Tooltip",
    "Typography",
    "useMediaQuery",
    "usePagination",
    "useScrollTrigger",
    "Zoom",
    "useAutocomplete",
    "GlobalStyles",
    "unstable_composeClasses",
    "generateUtilityClass",
    "generateUtilityClasses",
    "Unstable_TrapFocus",
    "InitColorSchemeScript",
];

const EXPECTED_MUI_LAB_DEFAULT_EXPORTS: &[&str] = &[
    "CalendarPicker",
    "ClockPicker",
    "DatePicker",
    "DateRangePicker",
    "DateRangePickerDay",
    "DateTimePicker",
    "DesktopDatePicker",
    "DesktopDateRangePicker",
    "DesktopDateTimePicker",
    "DesktopTimePicker",
    "LoadingButton",
    "LocalizationProvider",
    "MobileDatePicker",
    "MobileDateRangePicker",
    "MobileDateTimePicker",
    "MobileTimePicker",
    "MonthPicker",
    "CalendarPickerSkeleton",
    "PickersDay",
    "StaticDatePicker",
    "StaticDateRangePicker",
    "StaticDateTimePicker",
    "StaticTimePicker",
    "TabContext",
    "TabList",
    "TabPanel",
    "TimePicker",
    "Timeline",
    "TimelineConnector",
    "TimelineContent",
    "TimelineDot",
    "TimelineItem",
    "TimelineOppositeContent",
    "TimelineSeparator",
    "TreeItem",
    "TreeView",
    "YearPicker",
    "useAutocomplete",
    "Masonry",
];

const EXPECTED_RETAINED_MUI_X_TARGETS: &[&str] = &[
    "mui_x_tree_view",
    "mui_x_data_grid",
    "mui_x_date_time_pickers",
    "mui_x_charts",
    "mui_x_line_chart",
    "mui_x_bar_chart",
    "mui_x_pie_chart",
    "mui_x_sparkline",
    "mui_x_gauge",
    "mui_x_agent_chat",
    "mui_x_chat_composer",
];

const REQUIRED_MUI_ICON_RESOLVER_LINES: &[&str] = &[
    "mod mui_icons;",
    "mui_icons::module_candidates",
    "mui_icons::render_module_pixels",
    "mui_icons::render_module_image",
];

const REQUIRED_MUI_ICON_MODULE_LINES: &[&str] = &[
    "dev/material-ui/packages/mui-icons-material/lib",
    "fn module_candidates",
    "fn module_svg",
    "fn render_module_pixels",
    "fn path_elements",
    "viewBox=\"0 0 24 24\"",
];

#[test]
fn mui_web_parity_pins_local_material_ui_source_baseline() {
    let package_json = workspace_file("dev/material-ui/package.json");
    assert!(
        package_json.contains(MUI_MONOREPO_VERSION),
        "MUI parity must stay pinned to local dev/material-ui v9.0.1"
    );

    assert_eq!(
        collect_default_exports("dev/material-ui/packages/mui-material/src/index.js"),
        expected_names(EXPECTED_MUI_MATERIAL_DEFAULT_EXPORTS),
        "mui-material default exports changed; update the retained MUI parity inventory before implementing component slices"
    );
    assert_eq!(
        collect_default_exports("dev/material-ui/packages/mui-lab/src/index.js"),
        expected_names(EXPECTED_MUI_LAB_DEFAULT_EXPORTS),
        "mui-lab default exports changed; update the retained MUI parity inventory before implementing Lab slices"
    );
}

#[test]
fn mui_web_parity_pins_local_material_svg_icon_source() {
    let icons_root = workspace_path("dev/material-ui/packages/mui-icons-material/lib");
    let icon_count = fs::read_dir(&icons_root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", icons_root.display()))
        .map(|entry| entry.expect("MUI icon entry should be readable").path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("js"))
        .count();

    assert_eq!(
        icon_count, MUI_MATERIAL_ICON_COUNT,
        "mui-icons-material SVG module inventory changed; update the retained icon registry plan before importing aliases"
    );
    for icon in ["Add", "Check", "Close", "Delete", "Edit", "Menu", "Search"] {
        let icon_path = icons_root.join(format!("{icon}.js"));
        assert!(
            icon_path.exists(),
            "required Material SVG icon source missing: {}",
            icon_path.display()
        );
    }
}

#[test]
fn editor_mui_web_parity_registers_local_material_icon_resolver() {
    let visual_assets = editor_file("src/ui/retained_host/host_contract/painter/visual_assets.rs");
    for required_line in REQUIRED_MUI_ICON_RESOLVER_LINES {
        assert!(
            visual_assets.contains(required_line),
            "editor visual asset resolver should include MUI icon source contract `{required_line}`"
        );
    }
    let mui_icons =
        editor_file("src/ui/retained_host/host_contract/painter/visual_assets/mui_icons.rs");
    for required_line in REQUIRED_MUI_ICON_MODULE_LINES {
        assert!(
            mui_icons.contains(required_line),
            "MUI icon module resolver should include source contract `{required_line}`"
        );
    }
    for accepted_alias in ["mui:", "@mui/icons-material/"] {
        assert!(
            mui_icons.contains(accepted_alias),
            "MUI icon resolver should accept the alias prefix `{accepted_alias}`"
        );
    }
}

#[test]
fn mui_web_parity_tracks_explicit_retained_mui_x_targets() {
    for key in EXPECTED_RETAINED_MUI_X_TARGETS {
        let prototype = editor_path(format!(
            "assets/ui/editor/material_components/material_{key}.zui"
        ));
        assert!(
            prototype.exists(),
            "retained MUI X target `{key}` should be backed by a Material Lab prototype at {}",
            prototype.display()
        );
    }

    let plan = workspace_file(".codex/plans/ZirconEditor MUI Web Parity Plan.md");
    for required in ["Data Grid", "Charts", "Date/Time Pickers", "Tree View"] {
        assert!(
            plan.contains(required),
            "MUI Web parity plan should name the MUI X target `{required}`"
        );
    }
}

#[test]
fn editor_material_theme_declares_mui_web_default_theme_tokens() {
    let theme = editor_file("assets/ui/theme/editor_material.v2.ui.toml");
    for expected_line in REQUIRED_MUI_WEB_THEME_LINES {
        assert!(
            theme.contains(expected_line),
            "editor Material theme should pin MUI Web token line `{expected_line}`"
        );
    }
}

#[test]
fn editor_mui_web_parity_keeps_no_extra_slint_boundary() {
    let editor_manifest = editor_file("Cargo.toml");
    for forbidden in [
        ["sli", "nt"].concat(),
        ["i-sli", "nt"].concat(),
        ["sli", "nt-build"].concat(),
    ] {
        assert!(
            !editor_manifest.contains(&forbidden),
            "zircon_editor/Cargo.toml must not add `{forbidden}` for retained MUI parity"
        );
    }

    for relative in collect_editor_ui_asset_files() {
        let source = fs::read_to_string(&relative)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", relative.display()));
        assert!(
            !source.contains("@material") && !source.contains("material.slint"),
            "retained editor UI asset must not import Slint Material: {}",
            relative.display()
        );
    }
}

fn collect_default_exports(relative: &str) -> Vec<String> {
    workspace_file(relative)
        .lines()
        .filter_map(|line| {
            line.strip_prefix("export { default as ")
                .and_then(|rest| rest.split_once(" }"))
                .map(|(name, _)| name.to_string())
        })
        .collect()
}

fn expected_names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn collect_editor_ui_asset_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_with_extension(editor_path("assets/ui"), "toml", &mut files);
    collect_files_with_extension(editor_path("assets/ui"), "zui", &mut files);
    files.sort();
    files.dedup();
    files
}

fn collect_files_with_extension(root: PathBuf, extension: &str, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| {
                panic!("entry under {} should be readable: {error}", root.display())
            })
            .path();
        if path.is_dir() {
            collect_files_with_extension(path, extension, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
}

fn editor_file(relative: &str) -> String {
    fs::read_to_string(editor_path(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn workspace_file(relative: &str) -> String {
    fs::read_to_string(workspace_path(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn editor_path(relative: impl Into<PathBuf>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative.into())
}

fn workspace_path(relative: impl Into<PathBuf>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under the workspace root")
        .join(relative.into())
}
