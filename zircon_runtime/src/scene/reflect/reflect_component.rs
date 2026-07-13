pub type ReflectComponentReadFieldBySlot = fn(
    &crate::scene::World,
    crate::scene::EntityId,
    &str,
    u32,
) -> Result<
    zircon_runtime_interface::reflect::ReflectedValue,
    zircon_runtime_interface::reflect::ReflectError,
>;

pub type ReflectComponentWriteFieldBySlot =
    fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
        u32,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

#[derive(Clone)]
pub struct ReflectComponent {
    pub type_path: String,
    pub contains: fn(&crate::scene::World, crate::scene::EntityId, &str) -> bool,
    pub read_field: fn(
        &crate::scene::World,
        crate::scene::EntityId,
        &str,
        &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    pub read_fields: fn(
        &crate::scene::World,
        crate::scene::EntityId,
        &str,
    ) -> Result<
        Vec<zircon_runtime_interface::reflect::ReflectFieldValue>,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    pub write_field: fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
        &str,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    pub read_field_by_slot: Option<ReflectComponentReadFieldBySlot>,
    pub write_field_by_slot: Option<ReflectComponentWriteFieldBySlot>,
    pub remove: fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
}

impl ReflectComponent {
    pub fn new(
        type_path: impl Into<String>,
        contains: fn(&crate::scene::World, crate::scene::EntityId, &str) -> bool,
        read_field: fn(
            &crate::scene::World,
            crate::scene::EntityId,
            &str,
            &str,
        ) -> Result<
            zircon_runtime_interface::reflect::ReflectedValue,
            zircon_runtime_interface::reflect::ReflectError,
        >,
        read_fields: fn(
            &crate::scene::World,
            crate::scene::EntityId,
            &str,
        ) -> Result<
            Vec<zircon_runtime_interface::reflect::ReflectFieldValue>,
            zircon_runtime_interface::reflect::ReflectError,
        >,
        write_field: fn(
            &mut crate::scene::World,
            crate::scene::EntityId,
            &str,
            &str,
            zircon_runtime_interface::reflect::ReflectedValue,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
        remove: fn(
            &mut crate::scene::World,
            crate::scene::EntityId,
            &str,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            contains,
            read_field,
            read_fields,
            write_field,
            read_field_by_slot: None,
            write_field_by_slot: None,
            remove,
        }
    }

    pub fn with_dense_field_slots(
        mut self,
        read_field_by_slot: ReflectComponentReadFieldBySlot,
        write_field_by_slot: ReflectComponentWriteFieldBySlot,
    ) -> Self {
        self.read_field_by_slot = Some(read_field_by_slot);
        self.write_field_by_slot = Some(write_field_by_slot);
        self
    }

    pub fn contains(&self, world: &crate::scene::World, entity: crate::scene::EntityId) -> bool {
        (self.contains)(world, entity, &self.type_path)
    }

    pub fn read_field(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        field_name: &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field)(world, entity, &self.type_path, field_name)
    }

    pub fn read_fields(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
    ) -> Result<
        Vec<zircon_runtime_interface::reflect::ReflectFieldValue>,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_fields)(world, entity, &self.type_path)
    }

    pub fn write_field(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        field_name: &str,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_field)(world, entity, &self.type_path, field_name, value)
    }

    pub fn read_field_by_slot(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        field_slot: u32,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        let Some(read_field_by_slot) = self.read_field_by_slot else {
            return Err(missing_dense_slot_adapter(&self.type_path, "read"));
        };
        read_field_by_slot(world, entity, &self.type_path, field_slot)
    }

    pub fn write_field_by_slot(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        field_slot: u32,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(write_field_by_slot) = self.write_field_by_slot else {
            return Err(missing_dense_slot_adapter(&self.type_path, "write"));
        };
        write_field_by_slot(world, entity, &self.type_path, field_slot, value)
    }

    pub fn remove(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.remove)(world, entity, &self.type_path)
    }
}

fn missing_dense_slot_adapter(
    type_path: &str,
    operation: &str,
) -> zircon_runtime_interface::reflect::ReflectError {
    zircon_runtime_interface::reflect::ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: format!("component reflection has no dense field-slot {operation} adapter"),
    }
}
