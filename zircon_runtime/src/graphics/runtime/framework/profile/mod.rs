#[path = "../../profile/apply_disabled_profile_features.rs"]
mod apply_disabled_profile_features;
#[path = "../../profile/apply_flagship_profile_features.rs"]
mod apply_flagship_profile_features;
#[path = "profile.rs"]
mod compile_options_for_profile;
#[path = "new_compile_options.rs"]
mod new_compile_options;

pub(in crate::graphics::runtime::render_framework) use compile_options_for_profile::compile_options_for_profile;
