use std::collections::HashMap;

use super::super::RegistryName;
use crate::core::CoreError;
use crate::core::ServiceKind;

#[test]
fn registry_name_accepts_only_exact_three_segment_names() {
    let valid = RegistryName::new("TestModule.Manager.ClockManager").unwrap();
    assert_eq!(valid.as_str(), "TestModule.Manager.ClockManager");

    for invalid in [
        "",
        "TestModule",
        "TestModule.Manager",
        ".Manager.ClockManager",
        "TestModule.Service.ClockManager",
        "TestModule..ClockManager",
        "TestModule.Manager.",
        " TestModule.Manager.ClockManager",
        "TestModule .Manager.ClockManager",
        "TestModule.Manager. ClockManager",
        "TestModule.Manager.ClockManager ",
        "TestModule.Manager.ClockManager.Extra",
    ] {
        let error = RegistryName::new(invalid).unwrap_err();
        assert!(matches!(
            error,
            CoreError::InvalidRegistryName(value) if value == invalid
        ));
    }
}

#[test]
fn registry_name_from_parts_builds_canonical_service_names() {
    let name = RegistryName::from_parts("TestModule", ServiceKind::Driver, "ClockDriver");

    assert_eq!(name.as_str(), "TestModule.Driver.ClockDriver");
    assert_eq!(name.module_name(), "TestModule");
    assert_eq!(name.service_kind(), ServiceKind::Driver);
    assert_eq!(name.service_name(), "ClockDriver");
}

#[test]
fn registry_name_from_parts_rejects_invalid_segments() {
    for invalid_module in ["", " TestModule", "TestModule ", "Test.Module"] {
        assert!(std::panic::catch_unwind(|| {
            RegistryName::from_parts(invalid_module, ServiceKind::Driver, "ClockDriver");
        })
        .is_err());
    }

    for invalid_service in ["", " ClockDriver", "ClockDriver ", "Clock.Driver"] {
        assert!(std::panic::catch_unwind(|| {
            RegistryName::from_parts("TestModule", ServiceKind::Driver, invalid_service);
        })
        .is_err());
    }
}

#[test]
fn registry_name_caches_segments_without_changing_string_contract() {
    let name = RegistryName::from_parts("TestModule", ServiceKind::Manager, "ClockManager");

    let mut services = HashMap::new();
    services.insert(name.clone(), 1usize);
    assert_eq!(services.get(name.as_str()), Some(&1));

    let encoded = serde_json::to_string(&name).unwrap();
    assert_eq!(encoded, "\"TestModule.Manager.ClockManager\"");
    let decoded: RegistryName = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, name);
    assert_eq!(decoded.module_name(), "TestModule");
    assert_eq!(decoded.service_kind(), ServiceKind::Manager);
    assert_eq!(decoded.service_name(), "ClockManager");

    let registry_name_source = include_str!("../descriptors/registry_name.rs");
    assert!(registry_name_source.contains("module_end: usize"));
    assert!(registry_name_source.contains("service_start: usize"));
    assert!(registry_name_source.contains("kind: ServiceKind"));
    assert!(registry_name_source.contains("module_end: module.len()"));
    assert!(
        registry_name_source.contains("let service_start = module.len() + kind_segment.len() + 2;")
    );
    assert!(registry_name_source.contains("impl Hash for RegistryName"));
    assert!(registry_name_source.contains("self.as_str().hash(state)"));
    assert!(registry_name_source.contains("fn registry_separator_offsets(value: &str)"));
    assert!(registry_name_source.contains("let bytes = value.as_bytes();"));
    assert!(registry_name_source.contains("let mut separator_count = 0;"));
    assert!(registry_name_source.contains("while index < bytes.len()"));
    assert!(registry_name_source.contains("first_separator = index;"));
    assert!(registry_name_source.contains("second_separator = index;"));
    assert!(registry_name_source.contains("separator_count += 1;"));
    assert!(registry_name_source.contains("if separator_count == 2"));
    assert!(registry_name_source.contains("Some((first_separator, second_separator))"));
    assert!(registry_name_source.contains("return None;"));
    assert!(registry_name_source.contains("fn is_canonical_segment(value: &str) -> bool"));
    assert!(registry_name_source.contains("fn is_canonical_dot_free_segment(value: &str) -> bool"));
    assert!(registry_name_source.contains("first.is_whitespace()"));
    assert!(registry_name_source.contains("let last = match chars.next_back()"));
    assert!(registry_name_source.contains("Some(last) => last"));
    assert!(registry_name_source.contains("None => first"));
    assert!(registry_name_source.contains("!last.is_whitespace()"));
    assert!(registry_name_source.contains("ch == '.'"));
    assert!(registry_name_source.contains("!last.is_whitespace()"));
    assert!(registry_name_source.contains("is_canonical_dot_free_segment(module)"));
    assert!(registry_name_source.contains("is_canonical_dot_free_segment(service)"));
    assert!(registry_name_source
        .contains("let kind_segment = &value.as_bytes()[kind_start..kind_end];"));
    assert!(registry_name_source.contains("ServiceKind::from_registry_segment_bytes(kind_segment)"));
    let new_start = registry_name_source.find("pub fn new(").unwrap();
    let from_parts_start = registry_name_source.find("pub fn from_parts(").unwrap();
    let new_source = &registry_name_source[new_start..from_parts_start];
    let from_parts_source = &registry_name_source
        [from_parts_start..registry_name_source.find("pub fn as_str(").unwrap()];
    assert!(!registry_name_source.contains("Self::new(value).expect"));
    assert!(!new_source.contains(".find('.')"));
    assert!(!new_source.contains(".trim()"));
    assert!(!new_source.contains("service.contains('.')"));
    assert!(
        !new_source.contains("ServiceKind::from_registry_segment(&value[kind_start..kind_end])")
    );
    assert!(!from_parts_source.contains(".trim()"));
    assert!(!from_parts_source.contains(".contains('.')"));
    assert!(!registry_name_source.contains(".split("));
    assert!(!registry_name_source.contains("split_once"));
    assert!(!registry_name_source.contains("rsplit_once"));
    assert!(!registry_name_source.contains("first_separator?"));
    assert!(!registry_name_source.contains("second_separator?"));
    assert!(!registry_name_source.contains("value.bytes().enumerate()"));
    assert!(!registry_name_source.contains(".unwrap_or("));
    let deserialize_start = registry_name_source
        .find("impl<'de> Deserialize<'de> for RegistryName")
        .expect("RegistryName serde implementation should stay visible");
    let display_start = registry_name_source
        .find("impl fmt::Display for RegistryName")
        .expect("Display impl should delimit serde source");
    let deserialize_source = &registry_name_source[deserialize_start..display_start];
    assert!(deserialize_source.contains("match Self::new(value)"));
    assert!(deserialize_source.contains("Ok(name) => Ok(name)"));
    assert!(deserialize_source.contains("Err(error) => Err(serde::de::Error::custom(error))"));
    assert!(!deserialize_source.contains(".map_err("));
}

#[test]
fn service_kind_registry_segments_are_canonical() {
    for kind in [
        ServiceKind::Driver,
        ServiceKind::Manager,
        ServiceKind::Plugin,
    ] {
        assert_eq!(
            ServiceKind::from_registry_segment(kind.as_str()),
            Some(kind)
        );
    }
    assert_eq!(ServiceKind::from_registry_segment("Service"), None);
    assert_eq!(ServiceKind::from_registry_segment("manager"), None);
}

#[test]
fn service_kind_registry_segments_use_direct_byte_match() {
    let lifecycle_source = include_str!("../lifecycle.rs");

    assert!(lifecycle_source.contains("pub fn from_registry_segment(value: &str) -> Option<Self>"));
    assert!(lifecycle_source.contains("Self::from_registry_segment_bytes(value.as_bytes())"));
    assert!(lifecycle_source
        .contains("pub(crate) fn from_registry_segment_bytes(value: &[u8]) -> Option<Self>"));
    assert!(lifecycle_source.contains("b\"Driver\" => Some(Self::Driver)"));
    assert!(lifecycle_source.contains("b\"Manager\" => Some(Self::Manager)"));
    assert!(lifecycle_source.contains("b\"Plugin\" => Some(Self::Plugin)"));

    let parser_start = lifecycle_source
        .find("pub fn from_registry_segment(value: &str) -> Option<Self>")
        .expect("ServiceKind parser should stay visible to the registry-name source guard");
    let parser_end = lifecycle_source[parser_start..]
        .find("pub const fn as_str(self) -> &'static str")
        .map(|offset| parser_start + offset)
        .expect("ServiceKind parser guard should end before as_str");
    let parser_source = &lifecycle_source[parser_start..parser_end];

    assert!(!parser_source.contains("match value {"));
    assert!(!parser_source.contains("\n            \"Driver\" => Some(Self::Driver)"));
    assert!(!parser_source.contains("\n            \"Manager\" => Some(Self::Manager)"));
    assert!(!parser_source.contains("\n            \"Plugin\" => Some(Self::Plugin)"));
    assert!(!parser_source.contains(".trim()"));
    assert!(!parser_source.contains(".to_ascii"));
}

#[test]
fn registry_name_new_uses_service_kind_byte_slice_parser() {
    let registry_name_source = include_str!("../descriptors/registry_name.rs");
    let lifecycle_source = include_str!("../lifecycle.rs");

    assert!(lifecycle_source
        .contains("pub(crate) fn from_registry_segment_bytes(value: &[u8]) -> Option<Self>"));
    assert!(lifecycle_source.contains("Self::from_registry_segment_bytes(value.as_bytes())"));
    assert!(registry_name_source
        .contains("let kind_segment = &value.as_bytes()[kind_start..kind_end];"));
    assert!(registry_name_source.contains("ServiceKind::from_registry_segment_bytes(kind_segment)"));

    let new_start = registry_name_source
        .find("pub fn new(")
        .expect("RegistryName::new should stay visible to the source guard");
    let from_parts_start = registry_name_source
        .find("pub fn from_parts(")
        .expect("RegistryName::from_parts should delimit the new() body");
    let new_source = &registry_name_source[new_start..from_parts_start];

    assert!(
        !new_source.contains("ServiceKind::from_registry_segment(&value[kind_start..kind_end])")
    );
    assert!(!new_source.contains("ServiceKind::from_registry_segment("));
    assert!(!new_source.contains("let kind = &value[kind_start..kind_end]"));
}
