#[path = "advanced_slots.rs"]
mod advanced_slots;
#[path = "../../builtin_render_feature/builtin_render_feature.rs"]
mod builtin_render_feature;
#[path = "../../builtin_render_feature/requires_explicit_opt_in.rs"]
mod requires_explicit_opt_in;

#[cfg(test)]
pub(crate) use advanced_slots::descriptor_only_advanced_slots;
pub(crate) use advanced_slots::{
    descriptor_only_advanced_slot, descriptor_only_advanced_slot_requires_capability_opt_in,
    AdvancedBuiltinFeatureSlot,
};
pub use builtin_render_feature::BuiltinRenderFeature;
