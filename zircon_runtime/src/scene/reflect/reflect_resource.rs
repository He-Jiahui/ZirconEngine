pub type ReflectResourceReadFieldBySlot = fn(
    &crate::scene::World,
    u32,
) -> Result<
    zircon_runtime_interface::reflect::ReflectedValue,
    zircon_runtime_interface::reflect::ReflectError,
>;

pub type ReflectResourceWriteFieldBySlot =
    fn(
        &mut crate::scene::World,
        u32,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

pub type ReflectResourceWriteFieldsBySlot =
    fn(
        &mut crate::scene::World,
        Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

pub type ReflectResourcePreflightTransfer =
    fn(
        &mut crate::scene::World,
        &mut crate::scene::World,
    ) -> Result<(), zircon_runtime_interface::reflect::ReflectError>;

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
    /// Moves the already validated preflight value into a dedicated artifact
    /// World before the live target begins publication.
    pub transfer_preflight: ReflectResourcePreflightTransfer,
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
    pub read_field_by_slot: ReflectResourceReadFieldBySlot,
    pub write_field_by_slot: ReflectResourceWriteFieldBySlot,
    pub write_fields_by_slot: ReflectResourceWriteFieldsBySlot,
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

    pub fn transfer_preflight(
        &self,
        source: &mut crate::scene::World,
        artifact: &mut crate::scene::World,
    ) -> Result<(), zircon_runtime_interface::reflect::ReflectError> {
        (self.transfer_preflight)(source, artifact)
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

    pub fn read_field_by_slot(
        &self,
        world: &crate::scene::World,
        field_slot: u32,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field_by_slot)(world, field_slot)
    }

    pub fn write_fields_by_slot(
        &self,
        world: &mut crate::scene::World,
        fields: Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_fields_by_slot)(world, fields)
    }

    pub fn write_field_by_slot(
        &self,
        world: &mut crate::scene::World,
        field_slot: u32,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_field_by_slot)(world, field_slot, value)
    }
}
