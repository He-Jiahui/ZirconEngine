use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::ui::component::UiComponentDescriptorRegistry;

const DIRECT_MUI_MATERIAL_DESCRIPTOR_EXPORTS: &[(&str, &str)] = &[
    ("Accordion", "Accordion"),
    ("AccordionActions", "AccordionActions"),
    ("AccordionDetails", "AccordionDetails"),
    ("AccordionSummary", "AccordionSummary"),
    ("Alert", "Alert"),
    ("AlertTitle", "AlertTitle"),
    ("AppBar", "AppBar"),
    ("Autocomplete", "Autocomplete"),
    ("Avatar", "Avatar"),
    ("AvatarGroup", "AvatarGroup"),
    ("Backdrop", "Backdrop"),
    ("Badge", "Badge"),
    ("BottomNavigation", "BottomNavigation"),
    ("BottomNavigationAction", "BottomNavigationAction"),
    ("Box", "Box"),
    ("Breadcrumbs", "Breadcrumbs"),
    ("Button", "Button"),
    ("ButtonBase", "ButtonBase"),
    ("ButtonGroup", "ButtonGroup"),
    ("Card", "Card"),
    ("CardActionArea", "CardActionArea"),
    ("CardActions", "CardActions"),
    ("CardContent", "CardContent"),
    ("CardHeader", "CardHeader"),
    ("CardMedia", "CardMedia"),
    ("Checkbox", "Checkbox"),
    ("Chip", "Chip"),
    ("CircularProgress", "Progress"),
    ("ClickAwayListener", "ClickAwayListener"),
    ("Collapse", "Collapse"),
    ("Container", "Container"),
    ("CssBaseline", "CssBaseline"),
    ("Dialog", "Dialog"),
    ("DialogActions", "DialogActions"),
    ("DialogContent", "DialogContent"),
    ("DialogContentText", "DialogContentText"),
    ("DialogTitle", "DialogTitle"),
    ("Divider", "Divider"),
    ("Drawer", "Drawer"),
    ("Fab", "FloatingActionButton"),
    ("Fade", "Fade"),
    ("FilledInput", "FilledInput"),
    ("FormControl", "FormControl"),
    ("FormControlLabel", "FormControlLabel"),
    ("FormGroup", "FormGroup"),
    ("FormHelperText", "FormHelperText"),
    ("FormLabel", "FormLabel"),
    ("Grid", "Grid"),
    ("Grow", "Grow"),
    ("Icon", "Icon"),
    ("IconButton", "IconButton"),
    ("ImageList", "ImageList"),
    ("ImageListItem", "ImageListItem"),
    ("ImageListItemBar", "ImageListItemBar"),
    ("Input", "Input"),
    ("InputAdornment", "InputAdornment"),
    ("InputBase", "InputBase"),
    ("InputLabel", "InputLabel"),
    ("LinearProgress", "Progress"),
    ("Link", "Link"),
    ("List", "List"),
    ("ListItem", "ListItem"),
    ("ListItemAvatar", "ListItemAvatar"),
    ("ListItemButton", "ListItemButton"),
    ("ListItemIcon", "ListItemIcon"),
    ("ListItemSecondaryAction", "ListItemSecondaryAction"),
    ("ListItemText", "ListItemText"),
    ("ListSubheader", "ListSubheader"),
    ("Menu", "Menu"),
    ("MenuItem", "MenuItem"),
    ("MenuList", "MenuList"),
    ("Modal", "Modal"),
    ("MobileStepper", "MobileStepper"),
    ("NativeSelect", "NativeSelect"),
    ("NoSsr", "NoSsr"),
    ("OutlinedInput", "OutlinedInput"),
    ("Pagination", "Pagination"),
    ("PaginationItem", "PaginationItem"),
    ("Paper", "Paper"),
    ("Popover", "Popover"),
    ("Popper", "Popper"),
    ("Portal", "Portal"),
    ("Radio", "Radio"),
    ("RadioGroup", "RadioGroup"),
    ("Rating", "Rating"),
    ("ScopedCssBaseline", "ScopedCssBaseline"),
    ("Select", "Select"),
    ("Skeleton", "Skeleton"),
    ("Slide", "Slide"),
    ("Slider", "Slider"),
    ("Snackbar", "Snackbar"),
    ("SnackbarContent", "SnackbarContent"),
    ("SpeedDial", "SpeedDial"),
    ("SpeedDialAction", "SpeedDialAction"),
    ("SpeedDialIcon", "SpeedDialIcon"),
    ("Stack", "Stack"),
    ("Step", "Step"),
    ("StepButton", "StepButton"),
    ("StepConnector", "StepConnector"),
    ("StepContent", "StepContent"),
    ("StepIcon", "StepIcon"),
    ("StepLabel", "StepLabel"),
    ("Stepper", "Stepper"),
    ("SvgIcon", "SvgIcon"),
    ("SwipeableDrawer", "SwipeableDrawer"),
    ("Switch", "Switch"),
    ("Table", "Table"),
    ("TableBody", "TableBody"),
    ("TableCell", "TableCell"),
    ("TableContainer", "TableContainer"),
    ("TableFooter", "TableFooter"),
    ("TableHead", "TableHead"),
    ("TablePagination", "TablePagination"),
    ("TablePaginationActions", "TablePaginationActions"),
    ("TableRow", "TableRow"),
    ("TableSortLabel", "TableSortLabel"),
    ("Tab", "Tab"),
    ("TabScrollButton", "TabScrollButton"),
    ("Tabs", "Tabs"),
    ("TextField", "TextField"),
    ("TextareaAutosize", "TextareaAutosize"),
    ("ToggleButton", "ToggleButton"),
    ("ToggleButtonGroup", "ToggleButtonGroup"),
    ("Tooltip", "Tooltip"),
    ("Toolbar", "Toolbar"),
    ("Typography", "Typography"),
    ("useMediaQuery", "UseMediaQuery"),
    ("Zoom", "Zoom"),
    ("InitColorSchemeScript", "InitColorSchemeScript"),
];

const GROUPED_MUI_MATERIAL_EXPORTS: &[&str] = &[];

const UTILITY_MUI_MATERIAL_EXPORTS: &[&str] = &[
    "GlobalStyles",
    "Unstable_TrapFocus",
    "darkScrollbar",
    "generateUtilityClass",
    "generateUtilityClasses",
    "unstable_composeClasses",
    "useAutocomplete",
    "usePagination",
    "useScrollTrigger",
];

const DIRECT_MUI_LAB_DESCRIPTOR_EXPORTS: &[(&str, &str)] = &[
    ("CalendarPicker", "DateTimePickers"),
    ("CalendarPickerSkeleton", "DateTimePickers"),
    ("ClockPicker", "DateTimePickers"),
    ("DatePicker", "DateTimePickers"),
    ("DateRangePicker", "DateTimePickers"),
    ("DateRangePickerDay", "DateTimePickers"),
    ("DateTimePicker", "DateTimePickers"),
    ("DesktopDatePicker", "DateTimePickers"),
    ("DesktopDateRangePicker", "DateTimePickers"),
    ("DesktopDateTimePicker", "DateTimePickers"),
    ("DesktopTimePicker", "DateTimePickers"),
    ("LoadingButton", "Button"),
    ("LocalizationProvider", "DateTimePickers"),
    ("Masonry", "Masonry"),
    ("MobileDatePicker", "DateTimePickers"),
    ("MobileDateRangePicker", "DateTimePickers"),
    ("MobileDateTimePicker", "DateTimePickers"),
    ("MobileTimePicker", "DateTimePickers"),
    ("MonthPicker", "DateTimePickers"),
    ("PickersDay", "DateTimePickers"),
    ("StaticDatePicker", "DateTimePickers"),
    ("StaticDateRangePicker", "DateTimePickers"),
    ("StaticDateTimePicker", "DateTimePickers"),
    ("StaticTimePicker", "DateTimePickers"),
    ("TabContext", "TabContext"),
    ("TabList", "TabList"),
    ("TabPanel", "TabPanel"),
    ("TimePicker", "DateTimePickers"),
    ("Timeline", "Timeline"),
    ("TimelineConnector", "TimelineConnector"),
    ("TimelineContent", "TimelineContent"),
    ("TimelineDot", "TimelineDot"),
    ("TimelineItem", "TimelineItem"),
    ("TimelineOppositeContent", "TimelineOppositeContent"),
    ("TimelineSeparator", "TimelineSeparator"),
    ("TreeItem", "TreeItem"),
    ("TreeView", "MaterialTreeView"),
    ("YearPicker", "DateTimePickers"),
];

const GROUPED_MUI_LAB_EXPORTS: &[&str] = &[];

const UTILITY_MUI_LAB_EXPORTS: &[&str] = &["useAutocomplete"];

#[test]
fn material_foundation_catalog_tracks_local_mui_material_exports() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let source_exports =
        collect_default_exports("dev/material-ui/packages/mui-material/src/index.js");
    let tracked_exports = tracked_exports(
        DIRECT_MUI_MATERIAL_DESCRIPTOR_EXPORTS,
        GROUPED_MUI_MATERIAL_EXPORTS,
        UTILITY_MUI_MATERIAL_EXPORTS,
    );

    assert_eq!(
        source_exports, tracked_exports,
        "every mui-material default export must be classified before component parity work proceeds"
    );
    for (export, descriptor_id) in DIRECT_MUI_MATERIAL_DESCRIPTOR_EXPORTS {
        assert!(
            registry.contains(descriptor_id),
            "mui-material export `{export}` should be covered by Material foundation descriptor `{descriptor_id}`"
        );
    }
}

#[test]
fn material_foundation_catalog_tracks_local_mui_lab_exports() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let source_exports = collect_default_exports("dev/material-ui/packages/mui-lab/src/index.js");
    let tracked_exports = tracked_exports(
        DIRECT_MUI_LAB_DESCRIPTOR_EXPORTS,
        GROUPED_MUI_LAB_EXPORTS,
        UTILITY_MUI_LAB_EXPORTS,
    );

    assert_eq!(
        source_exports, tracked_exports,
        "every mui-lab default export must be classified before Lab parity work proceeds"
    );
    for (export, descriptor_id) in DIRECT_MUI_LAB_DESCRIPTOR_EXPORTS {
        assert!(
            registry.contains(descriptor_id),
            "mui-lab export `{export}` should be covered by Material foundation descriptor `{descriptor_id}`"
        );
    }
}

fn tracked_exports(
    direct: &[(&str, &str)],
    grouped: &[&str],
    utility: &[&str],
) -> BTreeSet<String> {
    direct
        .iter()
        .map(|(export, _)| (*export).to_string())
        .chain(grouped.iter().map(|export| (*export).to_string()))
        .chain(utility.iter().map(|export| (*export).to_string()))
        .collect()
}

fn collect_default_exports(relative: &str) -> BTreeSet<String> {
    fs::read_to_string(workspace_path(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
        .lines()
        .filter_map(|line| {
            line.strip_prefix("export { default as ")
                .and_then(|rest| rest.split_once(" }"))
                .map(|(name, _)| name.to_string())
        })
        .collect()
}

fn workspace_path(relative: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime lives directly under the workspace root")
        .join(relative)
}
