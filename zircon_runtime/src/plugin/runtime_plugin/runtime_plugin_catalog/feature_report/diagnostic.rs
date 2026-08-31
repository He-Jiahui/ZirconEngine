use std::fmt::Write as _;

use super::RuntimePluginFeatureBlock;

const HEADER_SUFFIX: &str = " is blocked: ";
const DETAIL_SEPARATOR: &str = "; ";
const LIST_SEPARATOR: &str = ", ";
const UNKNOWN_FEATURE: &str = "feature is not declared by the plugin catalog";
const INVALID_OWNER_DEPENDENCY: &str =
    "owner dependency is missing, not marked primary, or not the only primary dependency";
const PROVIDER_MISSING: &str = "concrete runtime feature provider registration is missing";
const TARGET_UNSUPPORTED: &str = "target mode is not supported";
const MISSING_PLUGINS_PREFIX: &str = "missing plugins: ";
const MISSING_CAPABILITIES_PREFIX: &str = "missing capabilities: ";
const DEPENDENCY_CYCLE: &str = "feature capability dependencies form a cycle";
const UNRESOLVED: &str = "dependency status is unresolved";

impl RuntimePluginFeatureBlock {
    pub fn to_diagnostic(&self) -> String {
        let severity = if self.required {
            "required feature"
        } else {
            "optional feature"
        };
        let mut diagnostic =
            String::with_capacity(feature_block_diagnostic_capacity(self, severity));
        write!(diagnostic, "{severity} {}{HEADER_SUFFIX}", self.feature_id)
            .expect("writing feature block diagnostic to String cannot fail");

        let mut has_detail = false;
        if self.unknown_feature {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, UNKNOWN_FEATURE);
        }
        if self.invalid_owner_dependency {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, INVALID_OWNER_DEPENDENCY);
        }
        if self.provider_missing {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, PROVIDER_MISSING);
        }
        if self.target_unsupported {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, TARGET_UNSUPPORTED);
        }
        append_feature_block_list(
            &mut diagnostic,
            &mut has_detail,
            MISSING_PLUGINS_PREFIX,
            &self.missing_plugins,
        );
        append_feature_block_list(
            &mut diagnostic,
            &mut has_detail,
            MISSING_CAPABILITIES_PREFIX,
            &self.missing_capabilities,
        );
        if self.cycle {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, DEPENDENCY_CYCLE);
        }
        if !has_detail {
            append_feature_block_detail(&mut diagnostic, &mut has_detail, UNRESOLVED);
        }
        diagnostic
    }
}

fn append_feature_block_detail(output: &mut String, has_detail: &mut bool, detail: &str) {
    if *has_detail {
        output.push_str(DETAIL_SEPARATOR);
    }
    output.push_str(detail);
    *has_detail = true;
}

fn append_feature_block_list(
    output: &mut String,
    has_detail: &mut bool,
    prefix: &str,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    if *has_detail {
        output.push_str(DETAIL_SEPARATOR);
    }
    output.push_str(prefix);
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push_str(LIST_SEPARATOR);
        }
        output.push_str(value);
    }
    *has_detail = true;
}

fn feature_block_diagnostic_capacity(block: &RuntimePluginFeatureBlock, severity: &str) -> usize {
    let mut detail_count = 0usize;
    let mut detail_bytes = 0usize;
    if block.unknown_feature {
        add_feature_block_detail_len(&mut detail_count, &mut detail_bytes, UNKNOWN_FEATURE.len());
    }
    if block.invalid_owner_dependency {
        add_feature_block_detail_len(
            &mut detail_count,
            &mut detail_bytes,
            INVALID_OWNER_DEPENDENCY.len(),
        );
    }
    if block.provider_missing {
        add_feature_block_detail_len(&mut detail_count, &mut detail_bytes, PROVIDER_MISSING.len());
    }
    if block.target_unsupported {
        add_feature_block_detail_len(
            &mut detail_count,
            &mut detail_bytes,
            TARGET_UNSUPPORTED.len(),
        );
    }
    if !block.missing_plugins.is_empty() {
        add_feature_block_detail_len(
            &mut detail_count,
            &mut detail_bytes,
            MISSING_PLUGINS_PREFIX
                .len()
                .saturating_add(joined_feature_block_list_len(&block.missing_plugins)),
        );
    }
    if !block.missing_capabilities.is_empty() {
        add_feature_block_detail_len(
            &mut detail_count,
            &mut detail_bytes,
            MISSING_CAPABILITIES_PREFIX
                .len()
                .saturating_add(joined_feature_block_list_len(&block.missing_capabilities)),
        );
    }
    if block.cycle {
        add_feature_block_detail_len(&mut detail_count, &mut detail_bytes, DEPENDENCY_CYCLE.len());
    }
    if detail_count == 0 {
        add_feature_block_detail_len(&mut detail_count, &mut detail_bytes, UNRESOLVED.len());
    }

    severity
        .len()
        .saturating_add(1)
        .saturating_add(block.feature_id.len())
        .saturating_add(HEADER_SUFFIX.len())
        .saturating_add(detail_bytes)
        .saturating_add(
            detail_count
                .saturating_sub(1)
                .saturating_mul(DETAIL_SEPARATOR.len()),
        )
}

fn add_feature_block_detail_len(detail_count: &mut usize, detail_bytes: &mut usize, len: usize) {
    *detail_count = detail_count.saturating_add(1);
    *detail_bytes = detail_bytes.saturating_add(len);
}

fn joined_feature_block_list_len(values: &[String]) -> usize {
    values
        .iter()
        .map(String::len)
        .sum::<usize>()
        .saturating_add(
            values
                .len()
                .saturating_sub(1)
                .saturating_mul(LIST_SEPARATOR.len()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_feature_block_diagnostic_preserves_contract() {
        let block = RuntimePluginFeatureBlock {
            feature_id: "rendering.shadow".to_string(),
            required: true,
            missing_plugins: vec!["rendering".to_string(), "lighting".to_string()],
            missing_capabilities: vec!["render.shadow".to_string(), "gpu.compute".to_string()],
            target_unsupported: true,
            cycle: true,
            invalid_owner_dependency: true,
            provider_missing: true,
            unknown_feature: true,
            ..RuntimePluginFeatureBlock::default()
        };

        let diagnostic = block.to_diagnostic();
        assert_eq!(
            diagnostic,
            "required feature rendering.shadow is blocked: feature is not declared by the plugin catalog; owner dependency is missing, not marked primary, or not the only primary dependency; concrete runtime feature provider registration is missing; target mode is not supported; missing plugins: rendering, lighting; missing capabilities: render.shadow, gpu.compute; feature capability dependencies form a cycle"
        );
        assert_eq!(diagnostic.len(), diagnostic.capacity());

        assert_eq!(
            RuntimePluginFeatureBlock {
                feature_id: "audio.spatial".to_string(),
                ..RuntimePluginFeatureBlock::default()
            }
            .to_diagnostic(),
            "optional feature audio.spatial is blocked: dependency status is unresolved"
        );
    }
}
