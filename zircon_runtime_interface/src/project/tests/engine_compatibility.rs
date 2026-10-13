//! 验证项目引擎兼容性公共评估入口输出的快照、错误、方向和保守分类。
//! 比较器组合的细粒度边界由同目录的 `directional_range.rs` 单独覆盖。

use super::super::{
    assess_project_engine_compatibility, ProjectEngineCompatibility,
    ProjectEngineCompatibilityDisposition, ProjectEngineVersion,
};

/// 兼容与不兼容结果都保留形成判断时的要求文本和引擎版本。
#[test]
fn project_engine_compatibility_preserves_the_requirement_and_running_engine_identity() {
    let running = ProjectEngineVersion::parse("0.1.7").unwrap();

    let compatible =
        assess_project_engine_compatibility(Some(">=0.1.0, <0.2.0"), &running).unwrap();
    assert_eq!(
        compatible,
        ProjectEngineCompatibility::new(
            Some(">=0.1.0, <0.2.0".to_string()),
            running.clone(),
            ProjectEngineCompatibilityDisposition::Compatible,
        )
    );

    let incompatible = assess_project_engine_compatibility(Some("<0.1.0"), &running).unwrap();
    assert_eq!(
        incompatible.disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
    assert_eq!(incompatible.requirement(), Some("<0.1.0"));
    assert_eq!(incompatible.running_engine(), &running);
}

/// 无效要求产生解析错误，版本值则以单个 semver 字符串完成 serde 往返。
#[test]
fn project_engine_compatibility_rejects_an_invalid_requirement_and_uses_canonical_version_wire() {
    let running = ProjectEngineVersion::parse("0.1.7").unwrap();

    assert!(assess_project_engine_compatibility(Some("not a semver range"), &running).is_err());
    assert_eq!(serde_json::to_string(&running).unwrap(), "\"0.1.7\"");
    assert_eq!(
        serde_json::from_str::<ProjectEngineVersion>("\"0.1.7\"").unwrap(),
        running
    );
}

/// 要求区间完全位于当前版本一侧时，结果分别指出需要更高或更旧的引擎。
#[test]
fn project_engine_compatibility_classifies_proven_newer_and_older_requirements() {
    let running = ProjectEngineVersion::parse("0.1.7").unwrap();

    assert_eq!(
        assess_project_engine_compatibility(Some(">=0.2.0, <0.3.0"), &running)
            .unwrap()
            .disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresNewerEngine
    );
    assert_eq!(
        assess_project_engine_compatibility(
            Some("^0.1.0"),
            &ProjectEngineVersion::parse("0.2.0").unwrap()
        )
        .unwrap()
        .disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

/// 预发布语义与矛盾区间都不能被错误归为单向升级或降级。
#[test]
fn project_engine_compatibility_keeps_prerelease_and_empty_ranges_indeterminate() {
    let prerelease = ProjectEngineVersion::parse("0.1.7-preview.1").unwrap();

    assert_eq!(
        assess_project_engine_compatibility(Some(">=0.2.0"), &prerelease)
            .unwrap()
            .disposition(),
        ProjectEngineCompatibilityDisposition::Incompatible
    );
    assert_eq!(
        assess_project_engine_compatibility(
            Some(">=0.2.0, <0.1.0"),
            &ProjectEngineVersion::parse("0.1.7").unwrap(),
        )
        .unwrap()
        .disposition(),
        ProjectEngineCompatibilityDisposition::Incompatible
    );
}

/// 精确版本两侧和严格部分比较器都按归一化的开闭端点判定方向。
#[test]
fn project_engine_compatibility_normalizes_exact_and_partial_comparator_boundaries() {
    assert_eq!(
        assess_project_engine_compatibility(
            Some("=0.1.7"),
            &ProjectEngineVersion::parse("0.1.6").unwrap(),
        )
        .unwrap()
        .disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresNewerEngine
    );
    assert_eq!(
        assess_project_engine_compatibility(
            Some("=0.1.7"),
            &ProjectEngineVersion::parse("0.1.8").unwrap(),
        )
        .unwrap()
        .disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
    assert_eq!(
        assess_project_engine_compatibility(
            Some(">0.1"),
            &ProjectEngineVersion::parse("0.1.99").unwrap(),
        )
        .unwrap()
        .disposition(),
        ProjectEngineCompatibilityDisposition::ProjectRequiresNewerEngine
    );
}
