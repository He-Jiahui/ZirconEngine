use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use toml::Value;

use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::UiTemplateInstance;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetKind, UiResourceDependency,
    UiResourceDiagnostic, UiStyleSheet,
};

use super::value_normalizer::compose_tokens;

#[derive(Clone, Debug, PartialEq)]
pub struct UiCompiledDocument {
    pub asset: UiAssetHeader,
    pub(super) instance: UiTemplateInstance,
    pub resource_dependencies: Vec<UiResourceDependency>,
    pub resource_diagnostics: Vec<UiResourceDiagnostic>,
}

impl UiCompiledDocument {
    pub fn into_template_instance(self) -> UiTemplateInstance {
        self.instance
    }

    pub fn template_instance(&self) -> &UiTemplateInstance {
        &self.instance
    }

    pub fn resource_dependencies(&self) -> &[UiResourceDependency] {
        &self.resource_dependencies
    }

    pub fn resource_diagnostics(&self) -> &[UiResourceDiagnostic] {
        &self.resource_diagnostics
    }
}

pub struct UiDocumentCompiler {
    pub(super) widget_imports: BTreeMap<String, UiAssetDocument>,
    pub(super) style_imports: BTreeMap<String, UiAssetDocument>,
    component_registry: Cow<'static, UiComponentDescriptorRegistry>,
}

impl Default for UiDocumentCompiler {
    fn default() -> Self {
        Self {
            widget_imports: BTreeMap::new(),
            style_imports: BTreeMap::new(),
            component_registry: Cow::Borrowed(
                UiComponentDescriptorRegistry::editor_showcase_shared(),
            ),
        }
    }
}

impl UiDocumentCompiler {
    pub fn with_component_registry(mut self, registry: UiComponentDescriptorRegistry) -> Self {
        self.component_registry = Cow::Owned(registry);
        self
    }

    pub fn with_shared_component_registry(
        mut self,
        registry: &'static UiComponentDescriptorRegistry,
    ) -> Self {
        self.component_registry = Cow::Borrowed(registry);
        self
    }

    pub(super) fn component_descriptor(
        &self,
        component_id: &str,
    ) -> Option<&UiComponentDescriptor> {
        self.component_registry.descriptor(component_id)
    }

    pub(super) fn component_registry_revision(&self) -> u64 {
        self.component_registry.revision()
    }

    pub fn component_registry(&self) -> &UiComponentDescriptorRegistry {
        self.component_registry.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn legacy_owned_default_for_benchmark() -> Self {
        Self {
            widget_imports: BTreeMap::new(),
            style_imports: BTreeMap::new(),
            component_registry: Cow::Owned(
                UiComponentDescriptorRegistry::editor_showcase_shared().clone(),
            ),
        }
    }

    pub fn register_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if !matches!(
            document.asset.kind,
            UiAssetKind::Layout | UiAssetKind::Widget
        ) {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Widget,
                actual: document.asset.kind,
            });
        }
        let _ = self.widget_imports.insert(reference, document);
        Ok(self)
    }

    pub fn register_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<&mut Self, UiAssetError> {
        let reference = reference.into();
        if document.asset.kind != UiAssetKind::Style {
            return Err(UiAssetError::ImportKindMismatch {
                reference,
                expected: UiAssetKind::Style,
                actual: document.asset.kind,
            });
        }
        let _ = self.style_imports.insert(reference, document);
        Ok(self)
    }
}

#[derive(Default)]
pub(super) struct CompilationArtifacts {
    widget_styles: Vec<ResolvedStyleSheet>,
    seen_widget_assets: BTreeSet<String>,
}

impl CompilationArtifacts {
    pub(super) fn record_widget_styles(
        &mut self,
        document: &UiAssetDocument,
        inherited: &BTreeMap<String, Value>,
    ) {
        if !self.seen_widget_assets.insert(document.asset.id.clone()) {
            return;
        }
        let tokens = compose_tokens(inherited, &document.tokens);
        append_resolved_stylesheets(&mut self.widget_styles, &document.stylesheets, tokens);
    }

    pub(super) fn widget_styles(&self) -> &[ResolvedStyleSheet] {
        &self.widget_styles
    }
}

#[derive(Clone)]
pub(super) struct ResolvedStyleSheet {
    pub(super) stylesheet: UiStyleSheet,
    pub(super) tokens: BTreeMap<String, Value>,
}

fn append_resolved_stylesheets(
    output: &mut Vec<ResolvedStyleSheet>,
    stylesheets: &[UiStyleSheet],
    tokens: BTreeMap<String, Value>,
) {
    let Some((last, preceding)) = stylesheets.split_last() else {
        return;
    };
    output.extend(preceding.iter().map(|stylesheet| ResolvedStyleSheet {
        stylesheet: stylesheet.clone(),
        tokens: tokens.clone(),
    }));
    output.push(ResolvedStyleSheet {
        stylesheet: last.clone(),
        tokens,
    });
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_ee_widget_styles_keep_order_and_handle_empty_input() {
        let tokens = token_fixture(4);
        let stylesheets = [
            UiStyleSheet {
                id: "first".to_string(),
                rules: Vec::new(),
            },
            UiStyleSheet {
                id: "last".to_string(),
                rules: Vec::new(),
            },
        ];
        let mut output = Vec::new();

        append_resolved_stylesheets(&mut output, &stylesheets, tokens.clone());

        assert_eq!(
            output
                .iter()
                .map(|sheet| sheet.stylesheet.id.as_str())
                .collect::<Vec<_>>(),
            ["first", "last"]
        );
        assert!(output.iter().all(|sheet| sheet.tokens == tokens));

        append_resolved_stylesheets(&mut output, &[], token_fixture(4));
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn optimization_batch_ee_last_widget_token_map_is_moved() {
        let source = include_str!("ui_document_compiler.rs");
        let production = source
            .split("fn append_resolved_stylesheets")
            .nth(1)
            .expect("resolved stylesheet append implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("resolved stylesheet production implementation");

        assert!(production.contains("stylesheets.split_last()"));
        assert!(production.contains("tokens: tokens.clone()"));
        assert!(production.contains("tokens,"));
    }

    #[test]
    #[ignore = "release-only final widget token-map move benchmark"]
    fn optimization_batch_ee_final_widget_token_map_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const BUILDS_PER_SAMPLE: usize = 32;
        const TOKEN_COUNT: usize = 512;

        fn measure(
            fixture: &BTreeMap<String, Value>,
            append: fn(&mut Vec<ResolvedStyleSheet>, &[UiStyleSheet], BTreeMap<String, Value>),
        ) -> u128 {
            let stylesheet = UiStyleSheet {
                id: "single-widget-sheet".to_string(),
                rules: Vec::new(),
            };
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..BUILDS_PER_SAMPLE {
                let mut output = Vec::with_capacity(1);
                append(
                    &mut output,
                    std::slice::from_ref(black_box(&stylesheet)),
                    black_box(fixture.clone()),
                );
                checksum = checksum.wrapping_add(output[0].tokens.len());
                black_box(output);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn legacy_append(
            output: &mut Vec<ResolvedStyleSheet>,
            stylesheets: &[UiStyleSheet],
            tokens: BTreeMap<String, Value>,
        ) {
            for stylesheet in stylesheets {
                output.push(ResolvedStyleSheet {
                    stylesheet: stylesheet.clone(),
                    tokens: tokens.clone(),
                });
            }
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let fixture = token_fixture(TOKEN_COUNT);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure(&fixture, legacy_append));
                optimized_samples.push(measure(&fixture, append_resolved_stylesheets));
            } else {
                optimized_samples.push(measure(&fixture, append_resolved_stylesheets));
                legacy_samples.push(measure(&fixture, legacy_append));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME439_FINAL_WIDGET_TOKEN_MAP_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             builds_per_sample={BUILDS_PER_SAMPLE} token_count={TOKEN_COUNT} \
             pair_order=alternating_legacy_even legacy_token_entry_clones_per_sample={} \
             optimized_token_entry_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            BUILDS_PER_SAMPLE * TOKEN_COUNT,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "moving the final widget token map must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn token_fixture(count: usize) -> BTreeMap<String, Value> {
        (0..count)
            .map(|index| {
                (
                    format!("token.{index:04}"),
                    Value::String(format!("value-{index:04}-{}", "payload".repeat(8))),
                )
            })
            .collect()
    }
}
