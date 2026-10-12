#[path = "../../../support/parser/pending/dependency.rs"]
mod dependency;
#[path = "../../../support/parser/pending/feature.rs"]
mod feature;
#[path = "pending/module.rs"]
mod module;
#[path = "../../../support/parser/pending/parent_feature.rs"]
mod parent_feature;

pub(super) use self::dependency::push_optional_feature_dependency;
pub(super) use self::feature::push_optional_feature;
pub(super) use self::module::push_optional_feature_module;
