use std::collections::BTreeMap;

use crate::graphics::shader::ShaderBindingResourceType;

#[path = "reflection/resource_binding_type.rs"]
mod resource_binding_type;
#[path = "reflection/sampling_pairs.rs"]
mod sampling_pairs;
#[cfg(test)]
#[path = "reflection/tests/sampling_pairs_tests.rs"]
mod sampling_pairs_tests;
#[path = "reflection/stage_interface.rs"]
mod stage_interface;
#[cfg(test)]
#[path = "reflection/tests/stage_interface_tests.rs"]
mod stage_interface_tests;
use resource_binding_type::{shader_binding_resource_type, shader_min_buffer_binding_size};
use sampling_pairs::reflect_entry_sampling_pairs;
use stage_interface::{reflect_stage_io_numeric_type, ShaderStageIoNumericType};
pub(crate) use stage_interface::{
    ShaderFragmentOutputNumericType, ShaderFragmentOutputScalarKind, ShaderVertexInputScalarKind,
};

const TYPE_LAYOUT_HASH_DOMAIN: &[u8] = b"zircon.shader.type-layout.v1";
const INTERFACE_LAYOUT_HASH_DOMAIN: &[u8] = b"zircon.shader.interface-layout.v1";
const ENTRY_RESOURCE_LAYOUT_HASH_DOMAIN: &[u8] = b"zircon.shader.entry-resource-layout.v2";
const RESOURCE_LAYOUT_HASH_DOMAIN: &[u8] = b"zircon.shader.resource-layout.v2";

const STAGE_VERTEX_BIT: u16 = 1 << 0;
const STAGE_TASK_BIT: u16 = 1 << 1;
const STAGE_MESH_BIT: u16 = 1 << 2;
const STAGE_FRAGMENT_BIT: u16 = 1 << 3;
const STAGE_COMPUTE_BIT: u16 = 1 << 4;
const STAGE_RAY_GENERATION_BIT: u16 = 1 << 5;
const STAGE_MISS_BIT: u16 = 1 << 6;
const STAGE_ANY_HIT_BIT: u16 = 1 << 7;
const STAGE_CLOSEST_HIT_BIT: u16 = 1 << 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderTemplateReflection {
    pub(crate) entry_points: Vec<ShaderEntryPointReflection>,
    pub(crate) resource_bindings: Vec<ShaderResourceBindingReflection>,
    pub(crate) pipeline_override_count: usize,
    // A dependent hash describes the unspecialized ABI. Exact pipeline identity must also
    // include the selected pipeline-constant values.
    pub(crate) interface_requires_specialization: bool,
    pub(crate) resource_layout_requires_specialization: bool,
    pub(crate) interface_layout_hash: [u8; 32],
    pub(crate) resource_layout_hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderEntryPointReflection {
    pub(crate) name: String,
    pub(crate) stage: naga::ShaderStage,
    pub(crate) workgroup_size: [u32; 3],
    pub(crate) workgroup_size_overrides: [bool; 3],
    pub(crate) inputs: Vec<ShaderStageIoReflection>,
    pub(crate) outputs: Vec<ShaderStageIoReflection>,
    pub(crate) resource_bindings: Vec<ShaderResourceBindingIdentity>,
    pub(crate) sampling_pairs: Vec<ShaderSamplingPairIdentity>,
    pub(crate) resource_layout_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ShaderSamplingPairIdentity {
    pub(crate) texture_group: u32,
    pub(crate) texture_binding: u32,
    pub(crate) sampler_group: u32,
    pub(crate) sampler_binding: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderStageIoReflection {
    pub(crate) name: Option<String>,
    pub(crate) binding: Option<naga::Binding>,
    pub(crate) type_layout_hash: [u8; 32],
    pub(crate) type_layout_requires_specialization: bool,
    numeric_type: Option<ShaderStageIoNumericType>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderResourceBindingReflection {
    pub(crate) identity: ShaderResourceBindingIdentity,
    pub(crate) name: Option<String>,
    pub(crate) visibility: ShaderStageVisibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ShaderResourceBindingIdentity {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) resource_type: ShaderBindingResourceType,
    pub(crate) min_binding_size: Option<u64>,
    pub(crate) address_space: naga::AddressSpace,
    pub(crate) memory_decorations: naga::MemoryDecorations,
    pub(crate) type_layout_hash: [u8; 32],
    pub(crate) type_layout_requires_specialization: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShaderTypeLayoutReflection {
    hash: [u8; 32],
    requires_specialization: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShaderStageVisibility(u16);

impl ShaderStageVisibility {
    pub(crate) fn contains(self, stage: naga::ShaderStage) -> bool {
        self.0 & shader_stage_bit(stage) != 0
    }

    fn insert(&mut self, stage: naga::ShaderStage) {
        self.0 |= shader_stage_bit(stage);
    }
}

pub(crate) fn reflect_validated_shader_module(
    module: &naga::Module,
    module_info: &naga::valid::ModuleInfo,
) -> ShaderTemplateReflection {
    let mut type_layout_hashes = vec![None; module.types.len()];
    let mut entry_points = module
        .entry_points
        .iter()
        .map(|entry_point| reflect_entry_point(module, entry_point, &mut type_layout_hashes))
        .collect::<Vec<_>>();
    let mut resource_bindings = BTreeMap::new();
    for (entry_index, entry_point) in module.entry_points.iter().enumerate() {
        let entry_info = module_info.get_entry_point(entry_index);
        entry_points[entry_index].sampling_pairs = reflect_entry_sampling_pairs(module, entry_info);
        for (handle, global) in module.global_variables.iter() {
            let Some(binding) = global.binding else {
                continue;
            };
            if entry_info[handle].is_empty() {
                continue;
            }

            let type_layout = shader_type_layout_hash(module, global.ty, &mut type_layout_hashes);
            let identity = ShaderResourceBindingIdentity {
                group: binding.group,
                binding: binding.binding,
                resource_type: shader_binding_resource_type(module, global),
                min_binding_size: shader_min_buffer_binding_size(module, global),
                address_space: global.space,
                memory_decorations: global.memory_decorations,
                type_layout_hash: type_layout.hash,
                type_layout_requires_specialization: type_layout.requires_specialization,
            };
            entry_points[entry_index].resource_bindings.push(identity);
            let reflected = resource_bindings.entry(identity).or_insert_with(|| {
                ShaderResourceBindingReflection {
                    identity,
                    name: global.name.clone(),
                    visibility: ShaderStageVisibility::default(),
                }
            });
            reflected.visibility.insert(entry_point.stage);
        }
        entry_points[entry_index].resource_bindings.sort_unstable();
        entry_points[entry_index].resource_bindings.dedup();
        entry_points[entry_index].resource_layout_hash =
            shader_entry_resource_layout_hash(&entry_points[entry_index]);
    }
    let resource_bindings = resource_bindings.into_values().collect::<Vec<_>>();
    entry_points.sort_by(|left, right| {
        shader_stage_bit(left.stage)
            .cmp(&shader_stage_bit(right.stage))
            .then_with(|| left.name.cmp(&right.name))
    });

    let interface_requires_specialization = entry_points.iter().any(|entry_point| {
        entry_point
            .workgroup_size_overrides
            .into_iter()
            .any(|value| value)
            || entry_point
                .inputs
                .iter()
                .chain(&entry_point.outputs)
                .any(|value| value.type_layout_requires_specialization)
    });
    let resource_layout_requires_specialization = resource_bindings
        .iter()
        .any(|resource| resource.identity.type_layout_requires_specialization);

    ShaderTemplateReflection {
        interface_layout_hash: shader_interface_layout_hash(&entry_points),
        resource_layout_hash: shader_resource_layout_hash(&entry_points, &resource_bindings),
        pipeline_override_count: module.overrides.len(),
        interface_requires_specialization,
        resource_layout_requires_specialization,
        entry_points,
        resource_bindings,
    }
}

/// Reflect every declared resource in a validated module, including resources
/// that are not reachable from one of the module's entry points yet. Generated
/// material helpers are validated before a render template wires them into an
/// entry point, so reachability-only reflection would hide an incorrect ABI.
pub(crate) fn reflect_declared_shader_resources(
    module: &naga::Module,
    visibility: &[naga::ShaderStage],
) -> Vec<ShaderResourceBindingReflection> {
    let mut type_layout_hashes = vec![None; module.types.len()];
    let mut reflected = BTreeMap::new();
    for (_, global) in module.global_variables.iter() {
        let Some(binding) = global.binding else {
            continue;
        };
        let type_layout = shader_type_layout_hash(module, global.ty, &mut type_layout_hashes);
        let identity = ShaderResourceBindingIdentity {
            group: binding.group,
            binding: binding.binding,
            resource_type: shader_binding_resource_type(module, global),
            min_binding_size: shader_min_buffer_binding_size(module, global),
            address_space: global.space,
            memory_decorations: global.memory_decorations,
            type_layout_hash: type_layout.hash,
            type_layout_requires_specialization: type_layout.requires_specialization,
        };
        let mut stage_visibility = ShaderStageVisibility::default();
        for &stage in visibility {
            stage_visibility.insert(stage);
        }
        reflected.insert(
            identity,
            ShaderResourceBindingReflection {
                identity,
                name: global.name.clone(),
                visibility: stage_visibility,
            },
        );
    }
    reflected.into_values().collect()
}

fn reflect_entry_point(
    module: &naga::Module,
    entry_point: &naga::EntryPoint,
    type_layout_hashes: &mut [Option<ShaderTypeLayoutReflection>],
) -> ShaderEntryPointReflection {
    let mut inputs = Vec::new();
    for argument in &entry_point.function.arguments {
        reflect_stage_io(
            module,
            argument.name.as_deref(),
            argument.ty,
            argument.binding.as_ref(),
            type_layout_hashes,
            &mut inputs,
        );
    }

    let mut outputs = Vec::new();
    if let Some(result) = &entry_point.function.result {
        reflect_stage_io(
            module,
            None,
            result.ty,
            result.binding.as_ref(),
            type_layout_hashes,
            &mut outputs,
        );
    }
    inputs.sort_by(compare_stage_io);
    outputs.sort_by(compare_stage_io);

    ShaderEntryPointReflection {
        name: entry_point.name.clone(),
        stage: entry_point.stage,
        workgroup_size: entry_point.workgroup_size,
        workgroup_size_overrides: entry_point
            .workgroup_size_overrides
            .map(|overrides| overrides.map(|value| value.is_some()))
            .unwrap_or([false; 3]),
        inputs,
        outputs,
        resource_bindings: Vec::new(),
        sampling_pairs: Vec::new(),
        resource_layout_hash: [0; 32],
    }
}

fn reflect_stage_io(
    module: &naga::Module,
    name: Option<&str>,
    ty: naga::Handle<naga::Type>,
    binding: Option<&naga::Binding>,
    type_layout_hashes: &mut [Option<ShaderTypeLayoutReflection>],
    reflected: &mut Vec<ShaderStageIoReflection>,
) {
    if binding.is_some() {
        let type_layout = shader_type_layout_hash(module, ty, type_layout_hashes);
        reflected.push(ShaderStageIoReflection {
            name: name.map(str::to_owned),
            binding: binding.cloned(),
            type_layout_hash: type_layout.hash,
            type_layout_requires_specialization: type_layout.requires_specialization,
            numeric_type: reflect_stage_io_numeric_type(module, ty),
        });
        return;
    }

    if let naga::TypeInner::Struct { members, .. } = &module.types[ty].inner {
        for member in members {
            reflect_stage_io(
                module,
                member.name.as_deref(),
                member.ty,
                member.binding.as_ref(),
                type_layout_hashes,
                reflected,
            );
        }
        return;
    }

    let type_layout = shader_type_layout_hash(module, ty, type_layout_hashes);
    reflected.push(ShaderStageIoReflection {
        name: name.map(str::to_owned),
        binding: None,
        type_layout_hash: type_layout.hash,
        type_layout_requires_specialization: type_layout.requires_specialization,
        numeric_type: reflect_stage_io_numeric_type(module, ty),
    });
}

fn compare_stage_io(
    left: &ShaderStageIoReflection,
    right: &ShaderStageIoReflection,
) -> std::cmp::Ordering {
    compare_bindings(left.binding.as_ref(), right.binding.as_ref())
        .then_with(|| left.type_layout_hash.cmp(&right.type_layout_hash))
}

fn compare_bindings(
    left: Option<&naga::Binding>,
    right: Option<&naga::Binding>,
) -> std::cmp::Ordering {
    use naga::Binding;
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(Binding::BuiltIn(left)), Some(Binding::BuiltIn(right))) => left.cmp(right),
        (Some(Binding::BuiltIn(_)), Some(Binding::Location { .. })) => std::cmp::Ordering::Less,
        (Some(Binding::Location { .. }), Some(Binding::BuiltIn(_))) => std::cmp::Ordering::Greater,
        (
            Some(Binding::Location {
                location: left_location,
                interpolation: left_interpolation,
                sampling: left_sampling,
                blend_src: left_blend_src,
                per_primitive: left_per_primitive,
            }),
            Some(Binding::Location {
                location: right_location,
                interpolation: right_interpolation,
                sampling: right_sampling,
                blend_src: right_blend_src,
                per_primitive: right_per_primitive,
            }),
        ) => (
            left_location,
            left_interpolation,
            left_sampling,
            left_blend_src,
            left_per_primitive,
        )
            .cmp(&(
                right_location,
                right_interpolation,
                right_sampling,
                right_blend_src,
                right_per_primitive,
            )),
    }
}

fn shader_interface_layout_hash(entry_points: &[ShaderEntryPointReflection]) -> [u8; 32] {
    let mut hasher = reflection_hasher(INTERFACE_LAYOUT_HASH_DOMAIN);
    hash_usize(&mut hasher, entry_points.len());
    for entry_point in entry_points {
        hash_bytes(&mut hasher, entry_point.name.as_bytes());
        hash_u16(&mut hasher, shader_stage_bit(entry_point.stage));
        for dimension in entry_point.workgroup_size {
            hash_u32(&mut hasher, dimension);
        }
        for has_override in entry_point.workgroup_size_overrides {
            hash_bool(&mut hasher, has_override);
        }
        hash_stage_io_slice(&mut hasher, &entry_point.inputs);
        hash_stage_io_slice(&mut hasher, &entry_point.outputs);
    }
    *hasher.finalize().as_bytes()
}

fn hash_stage_io_slice(hasher: &mut blake3::Hasher, values: &[ShaderStageIoReflection]) {
    hash_usize(hasher, values.len());
    for value in values {
        hash_binding(hasher, value.binding.as_ref());
        hasher.update(&value.type_layout_hash);
        hash_bool(hasher, value.type_layout_requires_specialization);
    }
}

fn shader_entry_resource_layout_hash(entry_point: &ShaderEntryPointReflection) -> [u8; 32] {
    let mut hasher = reflection_hasher(ENTRY_RESOURCE_LAYOUT_HASH_DOMAIN);
    hash_bytes(&mut hasher, entry_point.name.as_bytes());
    hash_u16(&mut hasher, shader_stage_bit(entry_point.stage));
    hash_usize(&mut hasher, entry_point.resource_bindings.len());
    for identity in &entry_point.resource_bindings {
        hash_resource_binding_identity(&mut hasher, *identity);
    }
    hash_usize(&mut hasher, entry_point.sampling_pairs.len());
    for pair in &entry_point.sampling_pairs {
        hash_u32(&mut hasher, pair.texture_group);
        hash_u32(&mut hasher, pair.texture_binding);
        hash_u32(&mut hasher, pair.sampler_group);
        hash_u32(&mut hasher, pair.sampler_binding);
    }
    *hasher.finalize().as_bytes()
}

fn shader_resource_layout_hash(
    entry_points: &[ShaderEntryPointReflection],
    resources: &[ShaderResourceBindingReflection],
) -> [u8; 32] {
    let mut hasher = reflection_hasher(RESOURCE_LAYOUT_HASH_DOMAIN);
    hash_usize(&mut hasher, entry_points.len());
    for entry_point in entry_points {
        hasher.update(&entry_point.resource_layout_hash);
    }
    hash_usize(&mut hasher, resources.len());
    for resource in resources {
        hash_resource_binding_identity(&mut hasher, resource.identity);
        hash_u16(&mut hasher, resource.visibility.0);
    }
    *hasher.finalize().as_bytes()
}

fn hash_resource_binding_identity(
    hasher: &mut blake3::Hasher,
    identity: ShaderResourceBindingIdentity,
) {
    hash_u32(hasher, identity.group);
    hash_u32(hasher, identity.binding);
    hash_address_space(hasher, identity.address_space);
    hash_u32(hasher, u32::from(identity.memory_decorations.bits()));
    hasher.update(&identity.type_layout_hash);
    hash_bool(hasher, identity.type_layout_requires_specialization);
}

fn shader_type_layout_hash(
    module: &naga::Module,
    handle: naga::Handle<naga::Type>,
    cached: &mut [Option<ShaderTypeLayoutReflection>],
) -> ShaderTypeLayoutReflection {
    if let Some(reflection) = cached[handle.index()] {
        return reflection;
    }

    let mut hasher = reflection_hasher(TYPE_LAYOUT_HASH_DOMAIN);
    let requires_specialization = match &module.types[handle].inner {
        naga::TypeInner::Scalar(scalar) => {
            hash_bytes(&mut hasher, b"scalar");
            hash_scalar(&mut hasher, *scalar);
            false
        }
        naga::TypeInner::Vector { size, scalar } => {
            hash_bytes(&mut hasher, b"vector");
            hasher.update(&[u8::from(*size)]);
            hash_scalar(&mut hasher, *scalar);
            false
        }
        naga::TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => {
            hash_bytes(&mut hasher, b"matrix");
            hasher.update(&[u8::from(*columns), u8::from(*rows)]);
            hash_scalar(&mut hasher, *scalar);
            false
        }
        naga::TypeInner::CooperativeMatrix {
            columns,
            rows,
            scalar,
            role,
        } => {
            hash_bytes(&mut hasher, b"cooperative-matrix");
            hash_debug(&mut hasher, columns);
            hash_debug(&mut hasher, rows);
            hash_scalar(&mut hasher, *scalar);
            hash_debug(&mut hasher, role);
            false
        }
        naga::TypeInner::Atomic(scalar) => {
            hash_bytes(&mut hasher, b"atomic");
            hash_scalar(&mut hasher, *scalar);
            false
        }
        naga::TypeInner::Pointer { base, space } => {
            hash_bytes(&mut hasher, b"pointer");
            let base = shader_type_layout_hash(module, *base, cached);
            hasher.update(&base.hash);
            hash_address_space(&mut hasher, *space);
            base.requires_specialization
        }
        naga::TypeInner::ValuePointer {
            size,
            scalar,
            space,
        } => {
            hash_bytes(&mut hasher, b"value-pointer");
            hash_debug(&mut hasher, size);
            hash_scalar(&mut hasher, *scalar);
            hash_address_space(&mut hasher, *space);
            false
        }
        naga::TypeInner::Array { base, size, stride } => {
            hash_bytes(&mut hasher, b"array");
            let base = shader_type_layout_hash(module, *base, cached);
            hasher.update(&base.hash);
            let size_requires_specialization = hash_array_size(&mut hasher, module, *size, cached);
            hash_u32(&mut hasher, *stride);
            base.requires_specialization || size_requires_specialization
        }
        naga::TypeInner::Struct { members, span } => {
            hash_bytes(&mut hasher, b"struct");
            hash_u32(&mut hasher, *span);
            hash_usize(&mut hasher, members.len());
            let mut requires_specialization = false;
            for member in members {
                hash_optional_bytes(&mut hasher, member.name.as_deref().map(str::as_bytes));
                hash_u32(&mut hasher, member.offset);
                hash_binding(&mut hasher, member.binding.as_ref());
                let member_type = shader_type_layout_hash(module, member.ty, cached);
                hasher.update(&member_type.hash);
                requires_specialization |= member_type.requires_specialization;
            }
            requires_specialization
        }
        naga::TypeInner::Image {
            dim,
            arrayed,
            class,
        } => {
            hash_bytes(&mut hasher, b"image");
            hash_debug(&mut hasher, dim);
            hash_bool(&mut hasher, *arrayed);
            hash_debug(&mut hasher, class);
            false
        }
        naga::TypeInner::Sampler { comparison } => {
            hash_bytes(&mut hasher, b"sampler");
            hash_bool(&mut hasher, *comparison);
            false
        }
        naga::TypeInner::AccelerationStructure { vertex_return } => {
            hash_bytes(&mut hasher, b"acceleration-structure");
            hash_bool(&mut hasher, *vertex_return);
            false
        }
        naga::TypeInner::RayQuery { vertex_return } => {
            hash_bytes(&mut hasher, b"ray-query");
            hash_bool(&mut hasher, *vertex_return);
            false
        }
        naga::TypeInner::BindingArray { base, size } => {
            hash_bytes(&mut hasher, b"binding-array");
            let base = shader_type_layout_hash(module, *base, cached);
            hasher.update(&base.hash);
            base.requires_specialization || hash_array_size(&mut hasher, module, *size, cached)
        }
    };
    let reflection = ShaderTypeLayoutReflection {
        hash: *hasher.finalize().as_bytes(),
        requires_specialization,
    };
    cached[handle.index()] = Some(reflection);
    reflection
}

fn hash_array_size(
    hasher: &mut blake3::Hasher,
    module: &naga::Module,
    size: naga::ArraySize,
    cached: &mut [Option<ShaderTypeLayoutReflection>],
) -> bool {
    match size {
        naga::ArraySize::Constant(size) => {
            hash_bytes(hasher, b"constant");
            hash_u32(hasher, size.get());
            false
        }
        naga::ArraySize::Pending(handle) => {
            hash_bytes(hasher, b"pending");
            let override_value = &module.overrides[handle];
            hash_optional_bytes(hasher, override_value.name.as_deref().map(str::as_bytes));
            hash_optional_u16(hasher, override_value.id);
            hasher.update(&shader_type_layout_hash(module, override_value.ty, cached).hash);
            hash_bool(hasher, override_value.init.is_some());
            true
        }
        naga::ArraySize::Dynamic => {
            hash_bytes(hasher, b"dynamic");
            false
        }
    }
}

fn hash_binding(hasher: &mut blake3::Hasher, binding: Option<&naga::Binding>) {
    match binding {
        None => hash_bytes(hasher, b"none"),
        Some(naga::Binding::BuiltIn(built_in)) => {
            hash_bytes(hasher, b"built-in");
            hash_debug(hasher, built_in);
        }
        Some(naga::Binding::Location {
            location,
            interpolation,
            sampling,
            blend_src,
            per_primitive,
        }) => {
            hash_bytes(hasher, b"location");
            hash_u32(hasher, *location);
            hash_debug(hasher, interpolation);
            hash_debug(hasher, sampling);
            hash_debug(hasher, blend_src);
            hash_bool(hasher, *per_primitive);
        }
    }
}

fn hash_address_space(hasher: &mut blake3::Hasher, address_space: naga::AddressSpace) {
    match address_space {
        naga::AddressSpace::Storage { access } => {
            hash_bytes(hasher, b"storage");
            hash_u32(hasher, access.bits());
        }
        other => hash_debug(hasher, &other),
    }
}

fn hash_scalar(hasher: &mut blake3::Hasher, scalar: naga::Scalar) {
    hash_debug(hasher, &scalar.kind);
    hasher.update(&[scalar.width]);
}

fn shader_stage_bit(stage: naga::ShaderStage) -> u16 {
    match stage {
        naga::ShaderStage::Vertex => STAGE_VERTEX_BIT,
        naga::ShaderStage::Task => STAGE_TASK_BIT,
        naga::ShaderStage::Mesh => STAGE_MESH_BIT,
        naga::ShaderStage::Fragment => STAGE_FRAGMENT_BIT,
        naga::ShaderStage::Compute => STAGE_COMPUTE_BIT,
        naga::ShaderStage::RayGeneration => STAGE_RAY_GENERATION_BIT,
        naga::ShaderStage::Miss => STAGE_MISS_BIT,
        naga::ShaderStage::AnyHit => STAGE_ANY_HIT_BIT,
        naga::ShaderStage::ClosestHit => STAGE_CLOSEST_HIT_BIT,
    }
}

fn reflection_hasher(domain: &[u8]) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, domain);
    hasher
}

fn hash_debug(hasher: &mut blake3::Hasher, value: &impl std::fmt::Debug) {
    hash_bytes(hasher, format!("{value:?}").as_bytes());
}

fn hash_optional_bytes(hasher: &mut blake3::Hasher, value: Option<&[u8]>) {
    match value {
        Some(value) => {
            hash_bool(hasher, true);
            hash_bytes(hasher, value);
        }
        None => hash_bool(hasher, false),
    }
}

fn hash_optional_u16(hasher: &mut blake3::Hasher, value: Option<u16>) {
    match value {
        Some(value) => {
            hash_bool(hasher, true);
            hash_u16(hasher, value);
        }
        None => hash_bool(hasher, false),
    }
}

fn hash_bytes(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn hash_bool(hasher: &mut blake3::Hasher, value: bool) {
    hasher.update(&[u8::from(value)]);
}

fn hash_u16(hasher: &mut blake3::Hasher, value: u16) {
    hasher.update(&value.to_le_bytes());
}

fn hash_u32(hasher: &mut blake3::Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

#[cfg(test)]
#[path = "tests/reflection.rs"]
mod tests;
