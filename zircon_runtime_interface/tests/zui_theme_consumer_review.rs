//! 读取生成的主题消费者清单与 ZUI v2 产物，核对根节点、导入、依赖指纹及摘要和运行时解析契约。
use std::path::PathBuf;

use serde_json::Value;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

#[test]
fn generated_theme_consumers_use_runtime_v2_contract_and_product_controls() {
    let repo = std::env::var_os("ZUI_LAYOUT_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    let output = repo.join("docs/layout");
    let catalog: Value =
        serde_json::from_slice(&std::fs::read(output.join("catalog.json")).unwrap()).unwrap();
    let mut checked = 0;
    for entry in catalog["entries"].as_array().unwrap() {
        let Some(path) = entry["cases"][0]["reviewHost"]["path"].as_str() else {
            continue;
        };
        let bytes = std::fs::read(output.join(path)).unwrap();
        let digest = ZrRuntimeDigestV1::sha256(&bytes);
        assert_eq!(
            digest.as_str(),
            entry["cases"][0]["reviewHost"]["sha256"].as_str().unwrap()
        );
        let document: UiV2AssetDocument =
            toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(document.asset.kind, UiV2AssetKind::View);
        assert_eq!(document.root_node_id(), Some("review_theme_host"));
        let theme: UiV2AssetDocument = toml::from_str(
            &std::fs::read_to_string(repo.join(entry["sourcePath"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        assert_eq!(document.imports.styles, vec![theme.asset.id]);
        let map: Value = serde_json::from_slice(
            &std::fs::read(output.join(path).with_file_name("theme-consumers.json")).unwrap(),
        )
        .unwrap();
        for (index, sample) in map.as_array().unwrap().iter().enumerate() {
            let product: UiV2AssetDocument = toml::from_str(
                &std::fs::read_to_string(repo.join(sample["sourcePath"].as_str().unwrap()))
                    .unwrap(),
            )
            .unwrap();
            let mut authored = product.nodes[sample["nodeId"].as_str().unwrap()].clone();
            let consumer = &document.nodes[&format!("sample_{index}")];
            authored.layout = consumer.layout.clone();
            assert_eq!(
                &authored, consumer,
                "only the host mounting layout may differ"
            );
            assert!(entry["dependencyFingerprints"]
                .as_array()
                .unwrap()
                .iter()
                .any(|dependency| dependency["sourcePath"] == sample["sourcePath"]));
        }
        checked += 1;
    }
    assert_eq!(
        checked, 2,
        "regenerate the layout catalog before this contract test"
    );
}
