#[derive(Clone, Copy)]
pub struct ReflectResource {
    pub contains: fn(&crate::scene::World) -> bool,
    pub read_field: fn(
        &crate::scene::World,
        &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    pub read_fields: fn(
        &crate::scene::World,
    ) -> Result<
        Vec<zircon_runtime_interface::reflect::ReflectFieldValue>,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    pub write_field: fn(
        &mut crate::scene::World,
        &str,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
}

impl ReflectResource {
    pub fn contains(&self, world: &crate::scene::World) -> bool {
        (self.contains)(world)
    }

    pub fn read_field(
        &self,
        world: &crate::scene::World,
        field_name: &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field)(world, field_name)
    }

    pub fn read_fields(
        &self,
        world: &crate::scene::World,
    ) -> Result<
        Vec<zircon_runtime_interface::reflect::ReflectFieldValue>,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_fields)(world)
    }

    pub fn write_field(
        &self,
        world: &mut crate::scene::World,
        field_name: &str,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_field)(world, field_name, value)
    }
}
