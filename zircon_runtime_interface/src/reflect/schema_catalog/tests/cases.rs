//! 回归测试固定字段槽位与旧别名，拒绝损坏快照，并比较增量注册、替换、移除和全量重建的目录结果。
//!
//! 夹具在本测试模块拥有中立元数据，不安装实时对象适配器；断言只覆盖各用例明确构造的情形。

use super::{ReflectSchemaCatalog, ReflectSchemaCatalogEntry};
use crate::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
};

/// 构造单字段注册夹具，显式指定稳定 ID、当前名和别名，并使用 Value 序列化策略。
/// 路径构造错误直接使夹具失败；目录身份与依赖仍由各用例的接纳入口检查。
fn registration(
    type_path: &str,
    short_type_path: &str,
    field_id: ReflectFieldId,
    field_name: &str,
    aliases: &[&str],
) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path).unwrap(),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            field_id,
            field_name,
            "f32",
            ReflectEditorHint::Scalar,
        )
        .with_aliases(aliases.iter().map(|alias| (*alias).to_string()).collect())]),
        ReflectSerializationStrategy::Value,
    )
}

/// 按独立身份键生成字段 ID，再构造尚未接纳的目录项。
/// `field_name` 可与 `field_key` 不同，用于区分当前显示/访问名称与稳定身份。
fn entry(
    type_path: &str,
    short_type_path: &str,
    field_key: &str,
    field_name: &str,
) -> ReflectSchemaCatalogEntry {
    ReflectSchemaCatalogEntry::new(registration(
        type_path,
        short_type_path,
        ReflectFieldId::from_stable_keys(type_path, field_key),
        field_name,
        &[],
    ))
}

#[test]
/// 比较反向目录输入和反向别名列表；接纳后的指纹与完整快照应相等。
/// 本夹具只含一条依赖，依赖列表多项排序由接纳实现的显式排序保证。
fn catalog_fingerprint_is_independent_of_input_and_metadata_set_order() {
    let alpha_id = ReflectFieldId::from_stable_keys("game.Alpha", "value");
    let alpha = ReflectSchemaCatalogEntry::new(registration(
        "game.Alpha",
        "Alpha",
        alpha_id,
        "value",
        &["old_value", "legacy_value"],
    ))
    .with_dependencies(vec!["game.Dependency".to_string()]);
    let dependency = entry("game.Dependency", "Dependency", "enabled", "enabled");

    let reversed_metadata = ReflectSchemaCatalogEntry::new(registration(
        "game.Alpha",
        "Alpha",
        alpha_id,
        "value",
        &["legacy_value", "old_value"],
    ))
    .with_dependencies(vec!["game.Dependency".to_string()]);

    let first = ReflectSchemaCatalog::try_new(vec![alpha, dependency.clone()]).unwrap();
    let second = ReflectSchemaCatalog::try_new(vec![dependency, reversed_metadata]).unwrap();

    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_eq!(first.snapshot(), second.snapshot());
}

#[test]
/// 短名冲突返回歧义错误，完整路径仍可定位，冲突投影只保留该短名一次。
fn catalog_preserves_ambiguous_short_paths_without_selecting_a_winner() {
    let catalog = ReflectSchemaCatalog::try_new(vec![
        entry("game.first.Shared", "Shared", "value", "value"),
        entry("game.second.Shared", "Shared", "value", "value"),
    ])
    .unwrap();

    assert!(matches!(
        catalog.resolve_type_path("Shared"),
        Err(ReflectError::AmbiguousShortTypePath { .. })
    ));
    assert_eq!(
        catalog.resolve_type_path("game.first.Shared").unwrap(),
        "game.first.Shared"
    );
    assert_eq!(
        catalog.ambiguous_short_type_paths().collect::<Vec<_>>(),
        vec!["Shared"]
    );
}

#[test]
/// 分别拒绝重复完整路径和跨类型共享字段 ID，不把短名或字段名当全局身份。
fn catalog_rejects_duplicate_full_paths_and_global_field_ids() {
    let duplicate_path = ReflectSchemaCatalog::try_new(vec![
        entry("game.Actor", "Actor", "first", "first"),
        entry("game.Actor", "Actor", "second", "second"),
    ]);
    assert!(matches!(
        duplicate_path,
        Err(ReflectError::DuplicateTypePath { .. })
    ));

    let shared_id = ReflectFieldId::from_stable_keys("stable.Owner", "value");
    let duplicate_field_id = ReflectSchemaCatalog::try_new(vec![
        ReflectSchemaCatalogEntry::new(registration(
            "game.First",
            "First",
            shared_id,
            "value",
            &[],
        )),
        ReflectSchemaCatalogEntry::new(registration(
            "game.Second",
            "Second",
            shared_id,
            "value",
            &[],
        )),
    ]);
    assert!(matches!(
        duplicate_field_id,
        Err(ReflectError::InvalidFieldRegistration { .. })
    ));
}

#[test]
/// 同一类型内当前名和别名都解析到稳定 ID，未知名拒绝；槽位仍为原字段序列位置。
fn catalog_resolves_stable_slots_and_scoped_legacy_aliases() {
    let field_id = ReflectFieldId::from_stable_keys("game.Transform", "translation");
    let catalog =
        ReflectSchemaCatalog::try_new(vec![ReflectSchemaCatalogEntry::new(registration(
            "game.Transform",
            "Transform",
            field_id,
            "translation",
            &["position"],
        ))])
        .unwrap();

    assert_eq!(
        catalog
            .field_slot_by_id("game.Transform", field_id)
            .unwrap(),
        0
    );
    assert_eq!(
        catalog
            .resolve_legacy_field_id("game.Transform", "translation")
            .unwrap(),
        field_id
    );
    assert_eq!(
        catalog
            .resolve_legacy_field_id("game.Transform", "position")
            .unwrap(),
        field_id
    );
    assert!(matches!(
        catalog.resolve_legacy_field_id("game.Transform", "missing"),
        Err(ReflectError::UnknownField { .. })
    ));
}

#[test]
/// 分别构造缺失依赖目标和二项环，验证整批接纳拒绝不完整或有环的图。
fn catalog_requires_a_complete_acyclic_dependency_closure() {
    let missing =
        ReflectSchemaCatalog::try_new(vec![entry("game.Actor", "Actor", "value", "value")
            .with_dependencies(vec!["game.Missing".to_string()])]);
    assert!(matches!(
        missing,
        Err(ReflectError::InvalidRegistration { .. })
    ));

    let cycle = ReflectSchemaCatalog::try_new(vec![
        entry("game.First", "First", "value", "value")
            .with_dependencies(vec!["game.Second".to_string()]),
        entry("game.Second", "Second", "value", "value")
            .with_dependencies(vec!["game.First".to_string()]),
    ]);
    assert!(matches!(
        cycle,
        Err(ReflectError::InvalidRegistration { .. })
    ));
}

#[test]
/// 三项依赖链必须返回 Leaf、Middle、Root，检验依赖先于消费者的投影。
fn catalog_dependency_order_places_dependencies_before_consumers() {
    let catalog = ReflectSchemaCatalog::try_new(vec![
        entry("game.Leaf", "Leaf", "value", "value"),
        entry("game.Middle", "Middle", "value", "value")
            .with_dependencies(vec!["game.Leaf".to_string()]),
        entry("game.Root", "Root", "value", "value")
            .with_dependencies(vec!["game.Middle".to_string()]),
    ])
    .unwrap();

    assert_eq!(
        catalog.dependency_order().collect::<Vec<_>>(),
        vec!["game.Leaf", "game.Middle", "game.Root"]
    );
}

#[test]
/// 更改注册显示名使重算指纹不符，更改算法版本在重建前被拒绝。
fn catalog_rejects_tampered_or_unknown_version_snapshots() {
    let catalog =
        ReflectSchemaCatalog::try_new(vec![entry("game.Actor", "Actor", "value", "value")])
            .unwrap();
    let mut tampered = catalog.snapshot();
    tampered.entries[0].registration.display_name = "Tampered".to_string();
    assert!(matches!(
        ReflectSchemaCatalog::try_from_snapshot(tampered),
        Err(ReflectError::InvalidRegistration { .. })
    ));

    let mut unknown_version = catalog.snapshot();
    unknown_version.algorithm_version += 1;
    assert!(matches!(
        ReflectSchemaCatalog::try_from_snapshot(unknown_version),
        Err(ReflectError::InvalidRegistration { .. })
    ));
}

#[test]
/// 先插入依赖再插入消费者，得到与整批乱序接纳相同的快照。
fn incremental_catalog_matches_batch_publication() {
    let leaf = entry("game.Leaf", "Leaf", "value", "value");
    let root = entry("game.Root", "Root", "value", "value")
        .with_dependencies(vec!["game.Leaf".to_string()]);
    let batch = ReflectSchemaCatalog::try_new(vec![root.clone(), leaf.clone()]).unwrap();

    let mut incremental = ReflectSchemaCatalog::default();
    incremental.try_insert(leaf).unwrap();
    incremental.try_insert(root).unwrap();

    assert_eq!(incremental.snapshot(), batch.snapshot());
}

#[test]
/// 冲突 ID 替换失败后快照不变；有消费者时拒绝移除依赖，先移除消费者后才可清空。
fn catalog_replace_is_atomic_and_remove_rejects_live_dependents() {
    let leaf = entry("game.Leaf", "Leaf", "value", "value");
    let root = entry("game.Root", "Root", "value", "value")
        .with_dependencies(vec!["game.Leaf".to_string()]);
    let mut catalog = ReflectSchemaCatalog::try_new(vec![leaf, root]).unwrap();
    let before = catalog.snapshot();

    let conflicting_id = catalog
        .entries()
        .next()
        .unwrap()
        .registration
        .type_info
        .fields[0]
        .id;
    let invalid_replacement = ReflectSchemaCatalogEntry::new(registration(
        "game.Root",
        "Root",
        conflicting_id,
        "other",
        &[],
    ));
    assert!(catalog.try_replace(invalid_replacement).is_err());
    assert_eq!(catalog.snapshot(), before);

    assert!(matches!(
        catalog.try_remove("game.Leaf"),
        Err(ReflectError::InvalidRegistration { .. })
    ));
    assert!(catalog.try_remove("game.Root").unwrap().is_some());
    assert!(catalog.try_remove("game.Leaf").unwrap().is_some());
    assert!(catalog.is_empty());
}
