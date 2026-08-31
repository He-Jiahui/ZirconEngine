use std::collections::BTreeMap;

use toml::Value as TomlValue;

use super::super::host_nodes::RetainedUiHostNodeProjection;

const CATEGORY_ALL: &str = "All";
const CATEGORY_VISUAL: &str = "Visual";
const CATEGORY_INPUT: &str = "Input";
const CATEGORY_NUMERIC: &str = "Numeric";
const CATEGORY_SELECTION: &str = "Selection";
const CATEGORY_REFERENCE: &str = "Reference";
const CATEGORY_COLLECTIONS: &str = "Collections";
const CATEGORY_FEEDBACK: &str = "Feedback";

pub(super) fn project_selected_category_state(
    nodes: &mut [RetainedUiHostNodeProjection],
    selected_category: &str,
) {
    for node in nodes {
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        let Some(category) = nav_category_for_control(control_id) else {
            continue;
        };
        let selected = category == selected_category;
        set_projected_attribute(
            &mut node.attributes,
            "selected",
            TomlValue::Boolean(selected),
        );
        set_projected_attribute(
            &mut node.attributes,
            "selection_state",
            TomlValue::String(if selected { "selected" } else { "normal" }.to_string()),
        );
    }
}

fn set_projected_attribute(
    attributes: &mut BTreeMap<String, TomlValue>,
    key: &str,
    value: TomlValue,
) {
    if let Some(current) = attributes.get_mut(key) {
        *current = value;
    } else {
        attributes.insert(key.to_owned(), value);
    }
}

pub(super) fn should_keep_for_selected_category(
    node: &RetainedUiHostNodeProjection,
    selected_category: &str,
) -> bool {
    let Some(control_id) = node.control_id.as_deref() else {
        return true;
    };
    let Some(category) = demo_category_for_control(control_id) else {
        return true;
    };
    selected_category == CATEGORY_ALL || category == selected_category
}

fn nav_category_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "ShowAllCategory" => Some(CATEGORY_ALL),
        "ShowVisualCategory" => Some(CATEGORY_VISUAL),
        "ShowInputCategory" => Some(CATEGORY_INPUT),
        "ShowNumericCategory" => Some(CATEGORY_NUMERIC),
        "ShowSelectionCategory" => Some(CATEGORY_SELECTION),
        "ShowReferenceCategory" => Some(CATEGORY_REFERENCE),
        "ShowDataCategory" => Some(CATEGORY_COLLECTIONS),
        "ShowFeedbackCategory" => Some(CATEGORY_FEEDBACK),
        _ => None,
    }
}

fn demo_category_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "LabelDemo" | "RichLabelDemo" | "ImageDemo" | "IconDemo" | "SvgIconDemo"
        | "SeparatorDemo" => Some(CATEGORY_VISUAL),
        "ProgressBarDemo"
        | "SkeletonDemo"
        | "SpinnerDemo"
        | "BadgeDemo"
        | "HelpRowDemo"
        | "DialogDemo"
        | "ConfirmDialogDemo"
        | "CommandPaletteDemo"
        | "NotificationCenterDemo" => Some(CATEGORY_FEEDBACK),
        "ButtonDemo"
        | "ButtonOutlinedDemo"
        | "ButtonTextDemo"
        | "ButtonDangerDemo"
        | "ButtonDisabledDemo"
        | "IconButtonDemo"
        | "ToggleButtonDemo"
        | "CheckboxDemo"
        | "RadioDemo"
        | "SegmentedControlDemo"
        | "TabDemo"
        | "TabStripDemo"
        | "InputFieldDemo"
        | "TextFieldDemo" => Some(CATEGORY_INPUT),
        "NumberFieldDemo" | "RangeFieldDemo" | "SliderDemo" | "RangeSliderDemo"
        | "ColorFieldDemo" | "Vector2FieldDemo" | "Vector3FieldDemo" | "Vector4FieldDemo" => {
            Some(CATEGORY_NUMERIC)
        }
        "DropdownDemo" | "ComboBoxDemo" | "EnumFieldDemo" | "FlagsFieldDemo"
        | "SearchSelectDemo" | "ContextMenuDemo" | "DropdownPopupDemo" => Some(CATEGORY_SELECTION),
        "AssetFieldDemo" | "InstanceFieldDemo" | "ObjectFieldDemo" => Some(CATEGORY_REFERENCE),
        "GroupDemo"
        | "FoldoutDemo"
        | "PropertyRowDemo"
        | "InspectorSectionDemo"
        | "ArrayFieldDemo"
        | "MapFieldDemo"
        | "ListRowDemo"
        | "TableRowDemo"
        | "VirtualListDemo"
        | "PagedListDemo"
        | "WorldSpaceSurfaceDemo"
        | "TreeRowDemo"
        | "ContextActionMenuDemo" => Some(CATEGORY_COLLECTIONS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const UPDATES_PER_SAMPLE: usize = 65_536;

    #[test]
    fn optimization_batch_fq_editor403_reuses_existing_showcase_attribute_keys() {
        let mut attributes = BTreeMap::new();
        set_projected_attribute(&mut attributes, "selected", TomlValue::Boolean(false));
        set_projected_attribute(&mut attributes, "selected", TomlValue::Boolean(true));

        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes.get("selected"), Some(&TomlValue::Boolean(true)));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fq_editor403_borrowed_showcase_attribute_key_benchmark() {
        for _ in 0..4 {
            black_box(measure_updates(false));
            black_box(measure_updates(true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_updates(false));
                optimized_samples.push(measure_updates(true));
            } else {
                optimized_samples.push(measure_updates(true));
                legacy_samples.push(measure_updates(false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR403_BORROWED_SHOWCASE_ATTRIBUTE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updates_per_sample={UPDATES_PER_SAMPLE} legacy_owned_keys_per_sample={UPDATES_PER_SAMPLE} optimized_owned_keys_per_sample=0 value_strings_per_sample={UPDATES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_updates(optimized: bool) -> u128 {
        let mut attributes = BTreeMap::from([(
            "selection_state".to_owned(),
            TomlValue::String("normal".to_owned()),
        )]);
        let started = Instant::now();
        for update in 0..UPDATES_PER_SAMPLE {
            let value = TomlValue::String(if update & 1 == 0 {
                "selected".to_owned()
            } else {
                "normal".to_owned()
            });
            if optimized {
                set_projected_attribute(black_box(&mut attributes), "selection_state", value);
            } else {
                attributes.insert("selection_state".to_owned(), value);
            }
        }
        black_box(attributes);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
