#[derive(Clone, Copy)]
pub struct ReflectResource {
    pub estimate_stage_clone_bytes: Option<
        fn(&crate::scene::World) -> Result<usize, zircon_runtime_interface::reflect::ReflectError>,
    >,
    pub stage_clone: Option<
        fn(
            &crate::scene::World,
            &mut crate::scene::World,
        ) -> Result<(), zircon_runtime_interface::reflect::ReflectError>,
    >,
    pub ensure: Option<
        fn(
            &mut crate::scene::World,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    >,
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
    pub fn estimate_stage_clone_bytes(
        &self,
        source: &crate::scene::World,
    ) -> Result<Option<usize>, zircon_runtime_interface::reflect::ReflectError> {
        self.estimate_stage_clone_bytes
            .map(|estimate| estimate(source))
            .transpose()
    }

    pub fn stage_clone(
        &self,
        source: &crate::scene::World,
        target: &mut crate::scene::World,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(stage_clone) = self.stage_clone else {
            return Ok(false);
        };
        stage_clone(source, target)?;
        Ok(true)
    }

    pub fn ensure(
        &self,
        world: &mut crate::scene::World,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(ensure) = self.ensure else {
            return Ok(false);
        };
        ensure(world)
    }

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
