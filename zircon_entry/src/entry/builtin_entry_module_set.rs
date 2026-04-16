use zircon_core::ModuleDescriptor;

use super::entry_profile::EntryProfile;

#[derive(Clone, Debug)]
pub struct BuiltinEntryModuleSet {
    descriptors: Vec<ModuleDescriptor>,
}

impl BuiltinEntryModuleSet {
    pub fn for_profile(profile: EntryProfile) -> Self {
        let mut descriptors = vec![
            zircon_manager::module_descriptor(),
            zircon_platform::module_descriptor(),
            zircon_input::module_descriptor(),
            zircon_asset::module_descriptor(),
            zircon_graphics::module_descriptor(),
            zircon_scene::module_descriptor(),
            zircon_script::module_descriptor(),
            zircon_physics::module_descriptor(),
            zircon_sound::module_descriptor(),
            zircon_texture::module_descriptor(),
            zircon_ui::module_descriptor(),
            zircon_net::module_descriptor(),
            zircon_navigation::module_descriptor(),
            zircon_particles::module_descriptor(),
            zircon_animation::module_descriptor(),
        ];

        if matches!(profile, EntryProfile::Editor) {
            descriptors.push(zircon_editor::module_descriptor());
        }

        Self { descriptors }
    }

    pub fn descriptors(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }
}
