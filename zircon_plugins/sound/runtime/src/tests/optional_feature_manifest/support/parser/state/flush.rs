mod dependency;
mod feature;
mod module;
mod scope;

pub(super) use self::dependency::flush_pending_dependency;
pub(super) use self::module::flush_pending_module;
pub(super) use self::scope::close_optional_feature_scope;
