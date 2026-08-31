use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use toml::Value;

use super::support::{collect_zui_document_files, editor_asset_root, load_zui_document};

const FROZEN_EDITOR_ASSET_COUNT: usize = 248;
const FROZEN_RAW_COLOR_COUNT: usize = 267;
const FROZEN_RAW_COLOR_FILE_COUNT: usize = 12;
const FROZEN_RAW_COLOR_INVENTORY_HASH: u64 = 0xc918_0721_4acc_04c3;
const FROZEN_FIXED_WORD_COUNT: usize = 5_262;
const FROZEN_FIXED_WORD_FILE_COUNT: usize = 232;
const FROZEN_FIXED_WORD_INVENTORY_HASH: u64 = 0x51f7_905c_7f61_032f;
const FNV_1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

struct RawColorAllowance {
    path: &'static str,
    maximum: usize,
    reason: &'static str,
}

const RAW_COLOR_ALLOWANCES: &[RawColorAllowance] = &[
    RawColorAllowance {
        path: "theme/editor_tokens.zui",
        maximum: 29,
        reason: "the single editor theme-token owner",
    },
    RawColorAllowance {
        path: "components/workbench/shell/workbench_viewport_panel.zui",
        maximum: 109,
        reason: "viewport scene visualization colors",
    },
    RawColorAllowance {
        path: "material_component_lab.zui",
        maximum: 108,
        reason: "Material reference and brand-color laboratory",
    },
    RawColorAllowance {
        path: "material_demo_window.zui",
        maximum: 8,
        reason: "Material reference theme-token fixture",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_pie_chart.zui",
        maximum: 3,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_bar_chart.zui",
        maximum: 2,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_charts.zui",
        maximum: 2,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_line_chart.zui",
        maximum: 2,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_gauge.zui",
        maximum: 1,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/mui_x/material_mui_x_sparkline.zui",
        maximum: 1,
        reason: "data-visualization series palette",
    },
    RawColorAllowance {
        path: "material_components/data_display/material_material_icons.zui",
        maximum: 1,
        reason: "Material htmlColor reference fixture",
    },
    RawColorAllowance {
        path: "components/showcase/showcase_input_section.zui",
        maximum: 1,
        reason: "literal color-input sample value, not interaction chrome",
    },
];

struct FixedGroupAllowance {
    path: &'static str,
    node_id: &'static str,
    component: &'static str,
    reason: &'static str,
}

const FIXED_GROUP_ALLOWANCES: &[FixedGroupAllowance] = &[
    FixedGroupAllowance {
        path: "asset_browser.zui",
        node_id: "details_preview_visual_panel",
        component: "Overlay",
        reason: "stable asset-thumbnail format",
    },
    FixedGroupAllowance {
        path: "asset_browser.zui",
        node_id: "preview_visual_panel",
        component: "Overlay",
        reason: "stable asset-thumbnail format",
    },
    FixedGroupAllowance {
        path: "assets_activity.zui",
        node_id: "preview_visual_panel",
        component: "Overlay",
        reason: "stable asset-thumbnail format",
    },
    FixedGroupAllowance {
        path: "components/workbench/floating/workbench_preferences.zui",
        node_id: "preferences",
        component: "HorizontalGroup",
        reason: "bounded preferences dialog composition",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "left_drawer_shell",
        component: "Overlay",
        reason: "runtime-resizable primary drawer region",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "left_drawer_header",
        component: "Container",
        reason: "drawer header chrome",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "left_drawer_content",
        component: "Container",
        reason: "runtime-resizable primary drawer region",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "right_drawer_shell",
        component: "Overlay",
        reason: "runtime-resizable auxiliary drawer region",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "right_drawer_header",
        component: "Container",
        reason: "drawer header chrome",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "right_drawer_content",
        component: "Container",
        reason: "runtime-resizable auxiliary drawer region",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "bottom_drawer_shell",
        component: "Overlay",
        reason: "runtime-resizable output drawer region",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "bottom_drawer_header",
        component: "Container",
        reason: "drawer header chrome",
    },
    FixedGroupAllowance {
        path: "host/workbench_shell.zui",
        node_id: "bottom_drawer_content",
        component: "Container",
        reason: "runtime-resizable output drawer region",
    },
    FixedGroupAllowance {
        path: "workbench_activity_rail.zui",
        node_id: "activity_rail_button_0",
        component: "Container",
        reason: "stable square activity-rail control",
    },
    FixedGroupAllowance {
        path: "workbench_activity_rail.zui",
        node_id: "activity_rail_button_1",
        component: "Container",
        reason: "stable square activity-rail control",
    },
];

struct AbsolutePopupAnchorAllowance {
    path: &'static str,
    node_id: &'static str,
    reason: &'static str,
}

const ABSOLUTE_POPUP_ANCHOR_ALLOWANCES: &[AbsolutePopupAnchorAllowance] = &[
    AbsolutePopupAnchorAllowance {
        path: "components/showcase/showcase_collections_section.zui",
        node_id: "context_action_menu_demo",
        reason: "showcase fixture",
    },
    AbsolutePopupAnchorAllowance {
        path: "components/showcase/showcase_selection_section.zui",
        node_id: "context_menu_demo",
        reason: "showcase fixture",
    },
];

const FIXED_GROUP_COMPONENTS: &[&str] = &[
    "Container",
    "Grid",
    "HorizontalGroup",
    "Overlay",
    "VerticalGroup",
];

#[test]
fn editor_zui_raw_color_and_fixed_word_inventory_does_not_grow() {
    let editor_root = editor_asset_root().join("ui/editor");
    let files = collect_zui_document_files(&editor_root);
    let allowance_by_path = RAW_COLOR_ALLOWANCES
        .iter()
        .map(|allowance| (allowance.path, allowance))
        .collect::<BTreeMap<_, _>>();
    let mut raw_color_inventory = BTreeMap::new();
    let mut fixed_word_inventory = BTreeMap::new();
    let mut offenders = Vec::new();

    for path in &files {
        let relative = relative_editor_path(&editor_root, path);
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let raw_color_count = count_raw_colors(&source);
        let fixed_word_count = count_fixed_words(&source);
        if raw_color_count > 0 {
            raw_color_inventory.insert(relative.clone(), raw_color_count);
            match allowance_by_path.get(relative.as_str()) {
                Some(allowance) if raw_color_count <= allowance.maximum => {}
                Some(allowance) => offenders.push(format!(
                    "{relative} has {raw_color_count} raw colors, above {} allowed for {}",
                    allowance.maximum, allowance.reason
                )),
                None => offenders.push(format!(
                    "{relative} has {raw_color_count} unclassified raw colors; page interaction chrome must use editor theme tokens"
                )),
            }
        }
        if fixed_word_count > 0 {
            fixed_word_inventory.insert(relative, fixed_word_count);
        }
    }

    let raw_color_total = raw_color_inventory.values().sum::<usize>();
    let fixed_word_total = fixed_word_inventory.values().sum::<usize>();
    assert_eq!(
        files.len(),
        FROZEN_EDITOR_ASSET_COUNT,
        "the frozen editor/**/*.zui audit scope changed; review and refresh the per-file inventories explicitly"
    );
    assert!(
        raw_color_total <= FROZEN_RAW_COLOR_COUNT,
        "raw color count grew above the frozen baseline: {raw_color_inventory:#?}"
    );
    assert_eq!(
        raw_color_inventory.len(),
        FROZEN_RAW_COLOR_FILE_COUNT,
        "the raw-color file inventory changed; review every per-file count before refreshing the frozen inventory"
    );
    assert_eq!(
        stable_count_inventory_hash(&raw_color_inventory),
        FROZEN_RAW_COLOR_INVENTORY_HASH,
        "the raw-color per-file inventory changed; token migration and visualization allowances require an explicit inventory refresh: {raw_color_inventory:#?}"
    );
    assert!(
        fixed_word_total <= FROZEN_FIXED_WORD_COUNT,
        "Fixed word count grew above the frozen baseline: {fixed_word_inventory:#?}"
    );
    assert_eq!(
        fixed_word_inventory.len(),
        FROZEN_FIXED_WORD_FILE_COUNT,
        "the Fixed-word file inventory changed; review every per-file count before refreshing the frozen inventory"
    );
    assert_eq!(
        stable_count_inventory_hash(&fixed_word_inventory),
        FROZEN_FIXED_WORD_INVENTORY_HASH,
        "the Fixed-word per-file inventory changed; responsive-layout debt requires an explicit inventory refresh: {fixed_word_inventory:#?}"
    );
    assert!(
        offenders.is_empty(),
        "editor page interaction colors must not bypass the token owner; audited inventory: {raw_color_inventory:#?}; offenders: {offenders:#?}"
    );
}

#[test]
fn editor_zui_fixed_layout_groups_use_an_audited_family_contract() {
    let editor_root = editor_asset_root().join("ui/editor");
    let allowances = FIXED_GROUP_ALLOWANCES
        .iter()
        .map(|allowance| {
            (
                (allowance.path, allowance.node_id, allowance.component),
                allowance.reason,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeSet::new();
    let mut offenders = Vec::new();

    for path in collect_zui_document_files(&editor_root) {
        let relative = relative_editor_path(&editor_root, &path);
        for (node_id, node) in &load_zui_document(&path).nodes {
            if !FIXED_GROUP_COMPONENTS.contains(&node.component.as_str())
                || !node_axis_is_fixed(node.layout.as_ref(), "width")
                || !node_axis_is_fixed(node.layout.as_ref(), "height")
            {
                continue;
            }
            let key = (relative.as_str(), node_id.as_str(), node.component.as_str());
            if !allowances.contains_key(&key) {
                offenders.push(format!(
                    "{relative} node `{node_id}` fixes both axes of layout group `{}` without an audited stable-format family reason",
                    node.component
                ));
            }
            observed.insert((relative.clone(), node_id.clone(), node.component.clone()));
        }
    }

    let expected = FIXED_GROUP_ALLOWANCES
        .iter()
        .map(|allowance| {
            (
                allowance.path.to_string(),
                allowance.node_id.to_string(),
                allowance.component.to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "the fixed-group family inventory changed; remove stale allowances or document each new stable-format family"
    );
    assert!(
        offenders.is_empty(),
        "whole layout groups must remain responsive unless a node-level stable-format reason is audited: {offenders:#?}"
    );
}

#[test]
fn editor_zui_absolute_popup_anchors_are_limited_to_audited_fixtures_and_frozen_debt() {
    let editor_root = editor_asset_root().join("ui/editor");
    let allowances = ABSOLUTE_POPUP_ANCHOR_ALLOWANCES
        .iter()
        .map(|allowance| ((allowance.path, allowance.node_id), allowance.reason))
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeSet::new();
    let mut offenders = Vec::new();

    for path in collect_zui_document_files(&editor_root) {
        let relative = relative_editor_path(&editor_root, &path);
        for (node_id, node) in &load_zui_document(&path).nodes {
            if !declares_authored_popup_anchor(&node.props) {
                continue;
            }
            let key = (relative.as_str(), node_id.as_str());
            if !allowances.contains_key(&key) {
                offenders.push(format!(
                    "{relative} node `{node_id}` authors an absolute popup anchor instead of supplying a Runtime-owned trigger frame"
                ));
            }
            observed.insert((relative.clone(), node_id.clone()));
        }
    }

    let expected = ABSOLUTE_POPUP_ANCHOR_ALLOWANCES
        .iter()
        .map(|allowance| (allowance.path.to_string(), allowance.node_id.to_string()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "the absolute-popup-anchor inventory changed; remove stale frozen debt or classify any fixture explicitly"
    );
    assert!(
        offenders.is_empty(),
        "main editor surfaces must not calculate popup coordinates in .zui: {offenders:#?}"
    );
}

#[test]
fn lexical_governance_counters_match_the_frozen_regex_semantics() {
    assert_eq!(count_raw_colors("#112233 #aBcDeF80 #1234567"), 3);
    assert_eq!(count_raw_colors("$editor.accent #12345 nope"), 0);
    assert_eq!(count_fixed_words("Fixed FixedWidth NotFixed _Fixed"), 1);
    assert!(declares_authored_popup_anchor(&BTreeMap::from([(
        "popup_anchor_y".to_string(),
        Value::String("$editor.control.height.compact".to_string()),
    )])));
    assert!(!declares_authored_popup_anchor(&BTreeMap::from([(
        "popup_anchor_y".to_string(),
        Value::Float(0.0),
    )])));
}

fn relative_editor_path(editor_root: &Path, path: &Path) -> String {
    path.strip_prefix(editor_root)
        .unwrap_or_else(|error| {
            panic!(
                "strip editor root `{}` from `{}`: {error}",
                editor_root.display(),
                path.display()
            )
        })
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn count_raw_colors(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0usize;
    let mut index = 0usize;
    while index + 6 < bytes.len() {
        if bytes[index] == b'#'
            && bytes[index + 1..=index + 6]
                .iter()
                .all(u8::is_ascii_hexdigit)
        {
            count += 1;
            index += if index + 8 < bytes.len()
                && bytes[index + 7..=index + 8]
                    .iter()
                    .all(u8::is_ascii_hexdigit)
            {
                9
            } else {
                7
            };
            continue;
        }
        index += 1;
    }
    count
}

fn count_fixed_words(source: &str) -> usize {
    let bytes = source.as_bytes();
    bytes
        .windows(b"Fixed".len())
        .enumerate()
        .filter(|(index, window)| {
            *window == b"Fixed"
                && (*index == 0 || !is_word_byte(bytes[*index - 1]))
                && (*index + window.len() == bytes.len()
                    || !is_word_byte(bytes[*index + window.len()]))
        })
        .count()
}

fn stable_count_inventory_hash(inventory: &BTreeMap<String, usize>) -> u64 {
    let mut hash = FNV_1A_64_OFFSET_BASIS;
    for (path, count) in inventory {
        hash = hash_bytes(hash, path.as_bytes());
        hash = hash_byte(hash, 0);
        hash = hash_bytes(hash, count.to_string().as_bytes());
        hash = hash_byte(hash, b'\n');
    }
    hash
}

fn hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash = hash_byte(hash, *byte);
    }
    hash
}

fn hash_byte(hash: u64, byte: u8) -> u64 {
    (hash ^ u64::from(byte)).wrapping_mul(FNV_1A_64_PRIME)
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn node_axis_is_fixed(layout: Option<&BTreeMap<String, Value>>, axis: &str) -> bool {
    layout
        .and_then(|layout| layout.get(axis))
        .and_then(Value::as_table)
        .and_then(|axis| axis.get("stretch"))
        .and_then(Value::as_str)
        == Some("Fixed")
}

fn declares_authored_popup_anchor(props: &BTreeMap<String, Value>) -> bool {
    ["popup_anchor_x", "popup_anchor_y"].iter().any(|key| {
        props.get(*key).is_some_and(|value| {
            value.as_str().is_some_and(|value| !value.trim().is_empty())
                || value
                    .as_float()
                    .or_else(|| value.as_integer().map(|value| value as f64))
                    .is_some_and(|value| value != 0.0)
        })
    })
}
