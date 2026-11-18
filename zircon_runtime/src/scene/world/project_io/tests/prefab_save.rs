use crate::asset::assets::ProjectDocumentError;
use crate::asset::{AssetKind, AssetReference};
use serde_json::json;

use super::*;

const PREFAB_SOURCE: &str = r#"
uri = "res://prefabs/hero.prefab.toml"
name = "Hero"

[[scene.entities]]
entity = 100
name = "Root"
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }

[[scene.entities]]
entity = 101
parent = 100
name = "Weapon"
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
[scene.entities.joint]
connected_entity = 100
"#;
const OVERRIDE_ROW: &str = r#"
[[entities.prefab_instance.overrides]]
entity_path = "Root/Weapon"
property_path = "Joint.connected_entity"
"#;

fn setup() -> (
    Fixture,
    ProjectManager,
    AssetUri,
    PathBuf,
    AssetReference,
    String,
) {
    let root = std::env::temp_dir().join(format!("prefab-save-{}", crate::asset::AssetUuid::new()));
    fs::create_dir(&root).expect("exclusive test project");
    let owned = Fixture(root);
    let mut project = project(&owned.0);
    let prefab_uri = AssetUri::parse("res://prefabs/hero.prefab.toml").unwrap();
    let prefab_path = project
        .paths()
        .asset_root(&RelPath::project_assets())
        .join("prefabs/hero.prefab.toml");
    fs::create_dir_all(prefab_path.parent().unwrap()).unwrap();
    fs::write(&prefab_path, PREFAB_SOURCE).unwrap();
    project.scan_and_import().unwrap();
    let entry = project.asset_registry().entry_by_path(&prefab_uri).unwrap();
    assert_eq!(entry.type_marker(), AssetKind::Prefab);
    let reference = AssetReference::new(entry.uuid(), prefab_uri.clone());
    let persisted = project.persist_runtime_reference(&reference).unwrap();
    let persisted = persisted.project_ref().unwrap();
    let ImportedAsset::Prefab(template) = project.load_artifact(&prefab_uri).unwrap() else {
        panic!("fixture must supply a real prefab artifact");
    };
    assert_eq!(
        template.scene.entities[1]
            .joint
            .as_ref()
            .unwrap()
            .connected_entity,
        Some(100),
    );

    let uri = project.manifest().default_scene.clone();
    let source_path = project.source_path_for_uri(&uri).unwrap();
    let scene_prefix = format!(
        r#"
[[entities]]
entity = 7
name = "HeroInstance"
transform = {{ translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}

[entities.prefab_instance.prefab]
kind = "project"
guid = "{}"
path_hint = "{}"
"#,
        persisted.guid(),
        persisted.path_hint().as_str(),
    );

    (owned, project, uri, source_path, reference, scene_prefix)
}

#[test]
fn prefab_override_save_roundtrips_four_source_forms() {
    let (_owned, project, uri, source_path, reference, scene_prefix) = setup();
    for (case, suffix, expected_value) in [
        ("absent row", String::new(), None),
        (
            "numeric reference",
            format!("{OVERRIDE_ROW}value = 100\n"),
            Some(json!(100)),
        ),
        (
            "literal text",
            format!("{OVERRIDE_ROW}value = \"null\"\n"),
            Some(json!("null")),
        ),
        ("present null", OVERRIDE_ROW.to_owned(), Some(json!(null))),
    ] {
        let text = format!("# original before Save\n{scene_prefix}{suffix}");
        fs::write(&source_path, &text).unwrap();
        let (world, revision) = World::load_scene_document(&project, &uri)
            .unwrap()
            .into_parts();
        let before = world.to_scene_asset(&project).unwrap();
        let instance = before.entities[0].prefab_instance.as_ref().unwrap();
        assert_eq!(instance.prefab, reference, "{case}");
        assert_eq!(
            instance.overrides.len(),
            usize::from(expected_value.is_some()),
            "{case}",
        );
        assert_eq!(
            instance.overrides.first().map(|row| &row.value),
            expected_value.as_ref(),
            "{case}",
        );
        if let Some(row) = instance.overrides.first() {
            assert_eq!(row.entity_path, "Root/Weapon");
            assert_eq!(row.property_path, "Joint.connected_entity");
        }
        eprintln!(
            "prefab-save single={case:?} read_rows={} value={:?} source_current={}",
            instance.overrides.len(),
            instance.overrides.first().map(|r| &r.value),
            revision.matches_path(&source_path).unwrap()
        );
        let receipt = world
            .save_scene_to_project(&project, &uri, &revision)
            .unwrap_or_else(|error| {
                assert_eq!(fs::read(&source_path).unwrap(), text.as_bytes());
                if case == "present null"
                    && matches!(
                        &error,
                        SceneProjectError::ProjectDocument(ProjectDocumentError::Serialize { source })
                            if source.to_string() == "unsupported unit type"
                    )
                {
                    panic!("PREFAB_PRESENT_NULL_SAVE_RED: {error}");
                }
                panic!("unexpected setup/publication failure for {case}: {error}");
            });
        receipt.ensure_durable().unwrap();
        assert!(receipt
            .source_revision()
            .matches_path(&source_path)
            .unwrap());
        assert_ne!(
            fs::read(&source_path).unwrap(),
            text.as_bytes(),
            "Save must not return a no-op receipt"
        );
        let saved: toml::Value =
            toml::from_str(&fs::read_to_string(&source_path).unwrap()).unwrap();
        if case == "present null" {
            assert!(saved["entities"][0]["prefab_instance"]["overrides"][0]
                .as_table()
                .unwrap()
                .get("value")
                .is_none());
        }
        let (reopened, revision) = World::load_scene_document(&project, &uri)
            .unwrap()
            .into_parts();
        assert_eq!(reopened.to_scene_asset(&project).unwrap(), before, "{case}");
        reopened
            .save_scene_to_project(&project, &uri, &revision)
            .unwrap()
            .ensure_durable()
            .unwrap();
        let (again, _) = World::load_scene_document(&project, &uri)
            .unwrap()
            .into_parts();
        assert_eq!(again.to_scene_asset(&project).unwrap(), before, "{case}");
        eprintln!("prefab-save completed={case:?} durable_writes=2 rereads=2");
    }
}

#[test]
fn prefab_override_save_keeps_mixed_rows_on_their_own_entities() {
    let (_owned, project, uri, source_path, reference, prefix) = setup();
    let mut text = String::from("# mixed input before Save\n");
    let expected: [&[(&str, Option<&str>)]; 3] = [
        &[
            ("number", Some("100")),
            ("unset", None),
            ("literal", Some("\"null\"")),
        ],
        &[],
        &[("other_unset", None), ("enabled", Some("false"))],
    ];
    for (i, rows) in expected.iter().enumerate() {
        text.push_str(
            &prefix
                .replace("entity = 7", &format!("entity = {}", 7 + i))
                .replace("HeroInstance", &format!("Instance{i}")),
        );
        for (name, value) in *rows {
            text.push_str(&OVERRIDE_ROW.replace("Joint.connected_entity", name));
            if let Some(v) = value {
                text.push_str(&format!("value = {v}\n"));
            }
        }
    }
    fs::write(&source_path, &text).unwrap();
    let (world, revision) = World::load_scene_document(&project, &uri)
        .unwrap()
        .into_parts();
    let before = world.to_scene_asset(&project).unwrap();
    let expected_values = [
        vec![json!(100), json!(null), json!("null")],
        vec![],
        vec![json!(null), json!(false)],
    ];
    for (i, rows) in expected.iter().enumerate() {
        let en = before
            .entities
            .iter()
            .find(|e| e.entity == 7 + i as u64)
            .unwrap();
        let instance = en.prefab_instance.as_ref().unwrap();
        assert_eq!(instance.prefab, reference);
        assert_eq!(instance.overrides.len(), rows.len());
        for ((row, (name, _)), value) in instance
            .overrides
            .iter()
            .zip(*rows)
            .zip(&expected_values[i])
        {
            assert_eq!(row.entity_path, "Root/Weapon");
            assert_eq!(row.property_path.as_str(), *name);
            assert_eq!(&row.value, value);
        }
    }
    eprintln!("prefab-save mixed preconditions=true rows_by_entity=[3,0,2]");
    let receipt = world
        .save_scene_to_project(&project, &uri, &revision)
        .unwrap_or_else(|e| {
            assert_eq!(fs::read(&source_path).unwrap(), text.as_bytes());
            panic!("mixed rows refused at Save: {e}")
        });
    receipt.ensure_durable().unwrap();
    assert!(receipt
        .source_revision()
        .matches_path(&source_path)
        .unwrap());
    let saved = fs::read_to_string(&source_path).unwrap();
    assert_ne!(saved, text);
    let doc: toml::Value = toml::from_str(&saved).unwrap();
    for (i, rows) in expected.iter().enumerate() {
        let en = doc["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["entity"].as_integer() == Some(7 + i as i64))
            .unwrap();
        let actual = en["prefab_instance"]["overrides"].as_array().unwrap();
        assert_eq!(actual.len(), rows.len());
        for (row, (name, val)) in actual.iter().zip(*rows) {
            assert_eq!(row["property_path"].as_str(), Some(*name));
            assert_eq!(row.as_table().unwrap().contains_key("value"), val.is_some());
        }
    }
    let (again, _) = World::load_scene_document(&project, &uri)
        .unwrap()
        .into_parts();
    assert_eq!(again.to_scene_asset(&project).unwrap(), before);
    eprintln!("prefab-save mixed durable=true rows_by_entity=[3,0,2] reopen_equal=true");
}
