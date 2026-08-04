use crate::core::framework::scene::ComponentPropertyPath;
use crate::scene::world::World;

/// A schema-bound dynamic field resolved at the compile boundary.
///
/// The component type and field retain their exact schema keys. They are used
/// for direct map access during application; the raw property DTO remains only
/// with the enclosing writer for diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledDynamicProperty {
    component_id: Box<str>,
    property: Box<str>,
    schema_generation: u64,
    undeclared_catalog_generation: Option<u64>,
}

impl CompiledDynamicProperty {
    pub(super) fn compile(world: &World, property_path: &ComponentPropertyPath) -> Option<Self> {
        if World::is_runtime_property_component(property_path) {
            return None;
        }
        let (component_id, property) = property_path.as_str().rsplit_once('.')?;
        if component_id.is_empty() || property.is_empty() {
            return None;
        }
        let schema_generation = world.component_type_schema_generation(component_id);
        Some(Self {
            component_id: component_id.into(),
            property: property.into(),
            schema_generation,
            undeclared_catalog_generation: (schema_generation == 0)
                .then(|| world.component_type_schema_catalog_generation()),
        })
    }

    pub(super) fn is_current_for(&self, world: &World) -> bool {
        if self.schema_generation != 0 {
            return self.schema_generation
                == world.component_type_schema_generation(&self.component_id);
        }
        self.undeclared_catalog_generation == Some(world.component_type_schema_catalog_generation())
    }

    pub(super) fn component_id(&self) -> &str {
        &self.component_id
    }

    pub(super) fn property(&self) -> &str {
        &self.property
    }
}
