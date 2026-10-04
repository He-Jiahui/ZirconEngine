use super::*;
use serde_json::json;
use std::fmt::Write;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn package_declares_data_importer_capabilities() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"yaml".to_string())));
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(manifest
        .capabilities
        .contains(&XML_IMPORTER_CAPABILITY.to_string()));
}

#[test]
fn declaration_projects_data_asset_importer_package_metadata() {
    let descriptor = runtime_plugin_descriptor();
    let manifest = package_manifest();

    assert_eq!(
        descriptor.package_id(),
        DATA_ASSET_IMPORTER_DECLARATION.id()
    );
    assert_eq!(
        descriptor.category(),
        DATA_ASSET_IMPORTER_DECLARATION.category()
    );
    assert_eq!(
        descriptor.target_modes(),
        DATA_ASSET_IMPORTER_DECLARATION.target_modes()
    );
    assert_eq!(
        descriptor.capabilities(),
        runtime_capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        manifest.supported_platforms.as_slice(),
        DATA_ASSET_IMPORTER_DECLARATION.supported_platforms()
    );
    assert_eq!(
        manifest.default_packaging.as_slice(),
        DATA_ASSET_IMPORTER_DECLARATION.default_packaging()
    );
}

#[test]
fn data_asset_importer_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("data importer package exposes dist metadata");

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, ASSET_IMPORTER_DATA_DIST_CRATE_NAME);
    assert_eq!(
        distribution.runtime_entry,
        ASSET_IMPORTER_DATA_DIST_RUNTIME_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "asset_importer.data.dist")
        .expect("data importer package includes native dist module");
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(dist_module.crate_name, ASSET_IMPORTER_DATA_DIST_CRATE_NAME);
    assert!(dist_module.target_modes.contains(
        &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
    ));
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
    assert!(dist_module
        .capabilities
        .contains(&JSON_IMPORTER_CAPABILITY.to_string()));
}

#[test]
fn registration_contributes_module_and_data_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert_eq!(report.extensions.asset_importers().descriptors().len(), 4);
}

#[test]
fn yaml_importer_decodes_data_asset() {
    let asset = import_fixture(
        "config.yaml",
        "name: zircon\nscale: 2\nitems:\n  - a\n  - b\n",
    );

    match asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Yaml);
            assert_eq!(data.canonical_json["name"], "zircon");
            assert_eq!(data.canonical_json["scale"], 2);
            assert_eq!(data.canonical_json["items"], json!(["a", "b"]));
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn structured_data_importer_projects_canonical_asset_uri_dependencies() {
    let source = r#"[["res://textures/hero.png", "builtin://shader/pbr.wgsl"], "res://textures/hero.png", "res://data/references.json", "res://textures/../textures/noncanonical.png", "mem://preview/hero", "textures/local.png", {"nested": ["lib://shared/fonts/editor.font.toml"]}]"#;
    let outcome = import_outcome("references.json", source);
    let dependencies = &outcome.root_entry().expect("data root entry").dependencies;

    assert_eq!(
        dependencies,
        &vec![
            zircon_runtime::asset::AssetUri::parse("res://textures/hero.png").unwrap(),
            zircon_runtime::asset::AssetUri::parse("builtin://shader/pbr.wgsl").unwrap(),
            zircon_runtime::asset::AssetUri::parse("lib://shared/fonts/editor.font.toml")
                .unwrap(),
        ]
    );
}

#[test]
fn xml_importer_decodes_data_asset() {
    let asset = import_fixture(
        "panel.xml",
        r#"<panel id="main"><label>Hello</label><button enabled="true">Run</button></panel>"#,
    );

    match asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Xml);
            assert_eq!(data.canonical_json["name"], "panel");
            assert_eq!(data.canonical_json["attributes"]["id"], "main");
            assert_eq!(data.canonical_json["children"][0]["name"], "label");
            assert_eq!(data.canonical_json["children"][0]["text"], "Hello");
            assert_eq!(
                data.canonical_json["children"][1]["attributes"]["enabled"],
                "true"
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn xml_importer_preserves_ordered_namespace_and_mixed_content_metadata() {
    let source = r#"<p xmlns:x="urn:example" first="1" x:second="2">lead<x:item>one</x:item>middle<!--note--><x:item/>tail</p>"#;
    let asset = import_fixture("mixed.xml", source);

    match asset {
        ImportedAsset::Data(data) => {
            let root = &data.canonical_json;
            assert_eq!(root["source_span"], json!([0, source.len()]));
            assert_eq!(
                root["namespaces"],
                json!([{ "prefix": "x", "uri": "urn:example" }])
            );
            assert_eq!(
                root["attributes_ordered"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| {
                        (
                            entry["qname"].as_str().unwrap(),
                            entry["name"].as_str().unwrap(),
                            entry["namespace"].as_str(),
                            entry["value"].as_str().unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
                vec![
                    ("first", "first", None, "1"),
                    ("x:second", "second", Some("urn:example"), "2")
                ]
            );
            assert_eq!(
                root["content"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| entry["kind"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                vec!["text", "element", "text", "comment", "element", "text"]
            );
            assert_eq!(root["content"][0]["value"], "lead");
            assert_eq!(root["content"][2]["value"], "middle");
            assert_eq!(root["content"][3]["value"], "note");
            assert_eq!(root["content"][5]["value"], "tail");
            assert_eq!(root["content"][1]["index"], 0);
            assert_eq!(root["content"][4]["index"], 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn invalid_yaml_returns_parse_error() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("broken.yaml"))
        .unwrap();
    let context = context_for("broken.yaml", "key: [unterminated");

    let error = importer.import(&context).unwrap_err();

    assert!(error.to_string().contains("parse yaml data"));
}

#[test]
fn plugins07_importer_hotpath_xml_single_pass_matches_legacy_neutral_tree() {
    let document = roxmltree::Document::parse(
        r#"<root role="panel">lead<a id="1">one</a>middle<b>two</b>tail</root>"#,
    )
    .unwrap();

    let source = r#"<root role="panel">lead<a id="1">one</a>middle<b>two</b>tail</root>"#;
    let optimized = xml_element_to_json(document.root_element(), source);
    let legacy = legacy_xml_element_to_json(document.root_element());

    assert_eq!(optimized["name"], legacy["name"]);
    assert_eq!(optimized["attributes"], legacy["attributes"]);
    assert_eq!(
        optimized["children"][0]["name"],
        legacy["children"][0]["name"]
    );
    assert_eq!(
        optimized["children"][1]["name"],
        legacy["children"][1]["name"]
    );
    assert_eq!(optimized["text"], legacy["text"]);
    assert_eq!(optimized["children"].as_array().unwrap().len(), 2);
    assert_eq!(
        optimized["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["kind"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["text", "element", "text", "element", "text"]
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_importer_hotpath_release_xml_single_pass_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const ELEMENTS: usize = 2_048;
    const TEXT_BYTES: usize = 96;
    const ITERATIONS: usize = 4;
    const THRESHOLD_PERCENT: u128 = 20;
    let text = "x".repeat(TEXT_BYTES);
    let mut source = String::with_capacity(ELEMENTS * (TEXT_BYTES + 32));
    source.push_str("<root>");
    for index in 0..ELEMENTS {
        write!(&mut source, "<item id=\"{index}\">{text}</item>").unwrap();
    }
    source.push_str("</root>");
    let document = roxmltree::Document::parse(&source).unwrap();
    let root = document.root_element();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_legacy_xml_conversion(root, ITERATIONS);
        let optimized = || measure_single_pass_xml_conversion(root, &source, ITERATIONS);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_xml_performance_gate(
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "elements={ELEMENTS} text_bytes={TEXT_BYTES} iterations_per_sample={ITERATIONS} legacy_child_scans_per_element=2 optimized_child_scans_per_element=1 legacy_single_text_clones_per_sample={} optimized_single_text_clones_per_sample=0",
            ELEMENTS * ITERATIONS
        ),
    );
}

fn legacy_xml_element_to_json(node: roxmltree::Node<'_, '_>) -> Value {
    let mut object = Map::new();
    object.insert(
        "name".to_string(),
        Value::String(node.tag_name().name().to_string()),
    );
    if let Some(namespace) = node.tag_name().namespace() {
        object.insert(
            "namespace".to_string(),
            Value::String(namespace.to_string()),
        );
    }
    let attributes = node
        .attributes()
        .map(|attribute| {
            (
                attribute.name().to_string(),
                Value::String(attribute.value().to_string()),
            )
        })
        .collect::<Map<_, _>>();
    if !attributes.is_empty() {
        object.insert("attributes".to_string(), Value::Object(attributes));
    }
    let text_nodes = node
        .children()
        .filter_map(|child| child.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| Value::String(text.to_string()))
        .collect::<Vec<_>>();
    match text_nodes.as_slice() {
        [] => {}
        [text] => {
            object.insert("text".to_string(), text.clone());
        }
        _ => {
            object.insert("text".to_string(), Value::Array(text_nodes));
        }
    }
    let children = node
        .children()
        .filter(|child| child.is_element())
        .map(legacy_xml_element_to_json)
        .collect::<Vec<_>>();
    if !children.is_empty() {
        object.insert("children".to_string(), Value::Array(children));
    }
    Value::Object(object)
}

fn measure_legacy_xml_conversion(node: roxmltree::Node<'_, '_>, iterations: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(legacy_xml_element_to_json(black_box(node)));
    }
    started.elapsed().as_nanos()
}

fn measure_single_pass_xml_conversion(
    node: roxmltree::Node<'_, '_>,
    source: &str,
    iterations: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(xml_element_to_json(black_box(node), source));
    }
    started.elapsed().as_nanos()
}

fn emit_xml_performance_gate(
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_xml_p95(legacy_samples);
    let optimized_p95 = nearest_rank_xml_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_xml_single_pass_tree sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        xml_samples_csv(legacy_samples),
        xml_samples_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "XML single-pass conversion must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_xml_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn xml_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn import_fixture(path: &str, source: &str) -> ImportedAsset {
    import_outcome(path, source)
        .root_entry()
        .map(|entry| entry.asset.clone())
        .expect("data importer root asset")
}

fn import_outcome(path: &str, source: &str) -> AssetImportOutcome {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new(path))
        .unwrap();
    importer.import(&context_for(path, source)).unwrap()
}

fn context_for(path: &str, source: &str) -> AssetImportContext {
    let file_name = path.replace('\\', "/");
    let uri = format!("res://data/{file_name}");
    AssetImportContext::new(
        path.into(),
        zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
        source.as_bytes().to_vec(),
        Default::default(),
    )
}
