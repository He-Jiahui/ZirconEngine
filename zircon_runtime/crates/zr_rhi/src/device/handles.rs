use std::array;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{DeviceGeneration, DeviceId};

const RESOURCE_KIND_COUNT: usize = 9;
static NEXT_RESOURCE_NAMESPACE: AtomicU64 = AtomicU64::new(1);

/// The resource domain carried by an opaque device handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RenderResourceKind {
    Buffer,
    Texture,
    TextureView,
    Sampler,
    BindGroupLayout,
    BindGroup,
    ShaderModule,
    PipelineLayout,
    Pipeline,
}

impl RenderResourceKind {
    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ResourceHandleIdentity {
    namespace: u64,
    device_id: DeviceId,
    device_generation: DeviceGeneration,
    kind: RenderResourceKind,
    slot: u32,
    slot_generation: u32,
}

impl ResourceHandleIdentity {
    const fn diagnostic_id(self) -> u64 {
        let mut value = self.namespace ^ self.device_id.raw().rotate_left(11);
        value ^= self.device_generation.raw().rotate_left(23);
        value ^= (self.kind as u64) << 56;
        value ^= (self.slot as u64).rotate_left(37);
        value ^= (self.slot_generation as u64).rotate_left(49);
        value ^ (value >> 33)
    }
}

#[derive(Clone, Copy, Debug)]
struct SlotState {
    generation: u32,
    occupied: bool,
    retired: bool,
}

#[derive(Debug)]
struct ResourceHandleAllocatorState {
    slots: [Vec<SlotState>; RESOURCE_KIND_COUNT],
    reusable_slots: [Vec<u32>; RESOURCE_KIND_COUNT],
}

impl Default for ResourceHandleAllocatorState {
    fn default() -> Self {
        Self {
            slots: array::from_fn(|_| Vec::new()),
            reusable_slots: array::from_fn(|_| Vec::new()),
        }
    }
}

/// Allocation failure for a resource-kind slot table.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderResourceHandleAllocationError {
    #[error("resource handle slots exhausted for `{kind:?}`")]
    SlotExhausted { kind: RenderResourceKind },
}

/// Validation failure returned before a backend registry dereferences a handle.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderResourceHandleError {
    #[error("resource handle belongs to device `{actual:?}`, expected `{expected:?}`")]
    WrongDevice {
        expected: DeviceId,
        actual: DeviceId,
    },
    #[error("resource handle belongs to device generation `{actual:?}`, expected `{expected:?}`")]
    WrongGeneration {
        expected: DeviceGeneration,
        actual: DeviceGeneration,
    },
    #[error("resource handle diagnostic id `{diagnostic_id}` belongs to another device registry")]
    ForeignAllocator { diagnostic_id: u64 },
    #[error("resource handle kind `{actual:?}` cannot be used as `{expected:?}")]
    WrongKind {
        expected: RenderResourceKind,
        actual: RenderResourceKind,
    },
    #[error("resource handle diagnostic id `{diagnostic_id}` is stale or has been destroyed")]
    StaleHandle { diagnostic_id: u64 },
}

/// Device-owned allocator for non-forgeable, generational RHI resource handles.
///
/// Backends keep one allocator for each concrete device generation. Its private
/// namespace prevents a second registry from accepting an otherwise identical
/// device-id/slot tuple, while slot generations reject use-after-destroy.
#[derive(Clone, Debug)]
pub struct RenderResourceHandleAllocator {
    device_id: DeviceId,
    device_generation: DeviceGeneration,
    namespace: u64,
    state: Arc<Mutex<ResourceHandleAllocatorState>>,
}

impl RenderResourceHandleAllocator {
    pub fn new(device_id: DeviceId, device_generation: DeviceGeneration) -> Self {
        let namespace = NEXT_RESOURCE_NAMESPACE.fetch_add(1, Ordering::Relaxed);
        Self {
            device_id,
            device_generation,
            namespace,
            state: Arc::new(Mutex::new(ResourceHandleAllocatorState::default())),
        }
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn device_generation(&self) -> DeviceGeneration {
        self.device_generation
    }

    fn allocate(
        &self,
        kind: RenderResourceKind,
    ) -> Result<ResourceHandleIdentity, RenderResourceHandleAllocationError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let index = kind.index();

        let (slot, slot_generation) = if let Some(slot) = state.reusable_slots[index].pop() {
            let slot_state = &mut state.slots[index][slot as usize];
            debug_assert!(!slot_state.occupied && !slot_state.retired);
            slot_state.occupied = true;
            (slot, slot_state.generation)
        } else {
            let slot = u32::try_from(state.slots[index].len())
                .map_err(|_| RenderResourceHandleAllocationError::SlotExhausted { kind })?;
            state.slots[index].push(SlotState {
                generation: 1,
                occupied: true,
                retired: false,
            });
            (slot, 1)
        };

        Ok(ResourceHandleIdentity {
            namespace: self.namespace,
            device_id: self.device_id,
            device_generation: self.device_generation,
            kind,
            slot,
            slot_generation,
        })
    }

    fn validate(
        &self,
        identity: ResourceHandleIdentity,
        expected_kind: RenderResourceKind,
    ) -> Result<(), RenderResourceHandleError> {
        self.validate_owner(identity, expected_kind)?;

        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::validate_slot(&state, identity, expected_kind)
    }

    fn validate_owner(
        &self,
        identity: ResourceHandleIdentity,
        expected_kind: RenderResourceKind,
    ) -> Result<(), RenderResourceHandleError> {
        if identity.device_id != self.device_id {
            return Err(RenderResourceHandleError::WrongDevice {
                expected: self.device_id,
                actual: identity.device_id,
            });
        }
        if identity.device_generation != self.device_generation {
            return Err(RenderResourceHandleError::WrongGeneration {
                expected: self.device_generation,
                actual: identity.device_generation,
            });
        }
        if identity.namespace != self.namespace {
            return Err(RenderResourceHandleError::ForeignAllocator {
                diagnostic_id: identity.diagnostic_id(),
            });
        }
        if identity.kind != expected_kind {
            return Err(RenderResourceHandleError::WrongKind {
                expected: expected_kind,
                actual: identity.kind,
            });
        }
        Ok(())
    }

    fn validate_slot(
        state: &ResourceHandleAllocatorState,
        identity: ResourceHandleIdentity,
        expected_kind: RenderResourceKind,
    ) -> Result<(), RenderResourceHandleError> {
        let Some(slot_state) = state.slots[expected_kind.index()].get(identity.slot as usize)
        else {
            return Err(RenderResourceHandleError::StaleHandle {
                diagnostic_id: identity.diagnostic_id(),
            });
        };
        if !slot_state.occupied || slot_state.generation != identity.slot_generation {
            return Err(RenderResourceHandleError::StaleHandle {
                diagnostic_id: identity.diagnostic_id(),
            });
        }
        Ok(())
    }

    fn release(
        &self,
        identity: ResourceHandleIdentity,
        expected_kind: RenderResourceKind,
    ) -> Result<(), RenderResourceHandleError> {
        self.validate_owner(identity, expected_kind)?;

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::validate_slot(&state, identity, expected_kind)?;
        let next_generation = {
            let slot_state = &mut state.slots[expected_kind.index()][identity.slot as usize];
            slot_state.occupied = false;
            slot_state.generation.checked_add(1)
        };
        if let Some(next_generation) = next_generation {
            state.slots[expected_kind.index()][identity.slot as usize].generation = next_generation;
            state.reusable_slots[expected_kind.index()].push(identity.slot);
        } else {
            state.slots[expected_kind.index()][identity.slot as usize].retired = true;
        }
        Ok(())
    }
}

macro_rules! define_resource_handle {
    ($name:ident, $kind:ident, $allocate:ident, $release:ident, $validate:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(ResourceHandleIdentity);

        impl $name {
            pub const fn device_id(self) -> DeviceId {
                self.0.device_id
            }

            pub const fn device_generation(self) -> DeviceGeneration {
                self.0.device_generation
            }

            /// Opaque identity suitable for diagnostics only, never for lookup.
            pub const fn diagnostic_id(self) -> u64 {
                self.0.diagnostic_id()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("diagnostic_id", &self.diagnostic_id())
                    .finish()
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_u64(self.diagnostic_id())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let _ = <serde::de::IgnoredAny as Deserialize>::deserialize(deserializer)?;
                Err(<D::Error as serde::de::Error>::custom(concat!(
                    stringify!($name),
                    " cannot be deserialized because RHI resource handles are device-owned"
                )))
            }
        }

        impl RenderResourceHandleAllocator {
            pub fn $allocate(&self) -> Result<$name, RenderResourceHandleAllocationError> {
                self.allocate(RenderResourceKind::$kind).map($name)
            }

            pub fn $validate(&self, handle: $name) -> Result<(), RenderResourceHandleError> {
                self.validate(handle.0, RenderResourceKind::$kind)
            }

            pub fn $release(&self, handle: $name) -> Result<(), RenderResourceHandleError> {
                self.release(handle.0, RenderResourceKind::$kind)
            }
        }
    };
}

define_resource_handle!(
    BufferHandle,
    Buffer,
    allocate_buffer,
    release_buffer,
    validate_buffer
);
define_resource_handle!(
    TextureHandle,
    Texture,
    allocate_texture,
    release_texture,
    validate_texture
);
define_resource_handle!(
    TextureViewHandle,
    TextureView,
    allocate_texture_view,
    release_texture_view,
    validate_texture_view
);
define_resource_handle!(
    SamplerHandle,
    Sampler,
    allocate_sampler,
    release_sampler,
    validate_sampler
);
define_resource_handle!(
    BindGroupLayoutHandle,
    BindGroupLayout,
    allocate_bind_group_layout,
    release_bind_group_layout,
    validate_bind_group_layout
);
define_resource_handle!(
    BindGroupHandle,
    BindGroup,
    allocate_bind_group,
    release_bind_group,
    validate_bind_group
);
define_resource_handle!(
    ShaderModuleHandle,
    ShaderModule,
    allocate_shader_module,
    release_shader_module,
    validate_shader_module
);
define_resource_handle!(
    PipelineLayoutHandle,
    PipelineLayout,
    allocate_pipeline_layout,
    release_pipeline_layout,
    validate_pipeline_layout
);
define_resource_handle!(
    PipelineHandle,
    Pipeline,
    allocate_pipeline,
    release_pipeline,
    validate_pipeline
);
