use super::{ShaderEntryPointReflection, ShaderStageIoReflection, ShaderTemplateReflection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ShaderStageIoNumericType {
    dimension: ShaderStageIoNumericDimension,
    scalar: naga::Scalar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShaderStageIoNumericDimension {
    Scalar,
    Vector(naga::VectorSize),
    Matrix(naga::VectorSize, naga::VectorSize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderVertexInputScalarKind {
    Sint,
    Uint,
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderFragmentOutputScalarKind {
    Sint,
    Uint,
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShaderFragmentOutputNumericType {
    component_count: u8,
    scalar_kind: ShaderFragmentOutputScalarKind,
    scalar_width: u8,
}

impl ShaderFragmentOutputNumericType {
    pub(crate) fn new(
        scalar_kind: ShaderFragmentOutputScalarKind,
        scalar_width: u8,
        component_count: u8,
    ) -> Option<Self> {
        (scalar_width > 0 && (1..=4).contains(&component_count)).then_some(Self {
            component_count,
            scalar_kind,
            scalar_width,
        })
    }

    fn numeric_type(self) -> ShaderStageIoNumericType {
        let dimension = match self.component_count {
            1 => ShaderStageIoNumericDimension::Scalar,
            2 => ShaderStageIoNumericDimension::Vector(naga::VectorSize::Bi),
            3 => ShaderStageIoNumericDimension::Vector(naga::VectorSize::Tri),
            4 => ShaderStageIoNumericDimension::Vector(naga::VectorSize::Quad),
            _ => unreachable!("fragment target component count is validated at construction"),
        };
        let kind = match self.scalar_kind {
            ShaderFragmentOutputScalarKind::Sint => naga::ScalarKind::Sint,
            ShaderFragmentOutputScalarKind::Uint => naga::ScalarKind::Uint,
            ShaderFragmentOutputScalarKind::Float => naga::ScalarKind::Float,
        };
        ShaderStageIoNumericType {
            dimension,
            scalar: naga::Scalar {
                kind,
                width: self.scalar_width,
            },
        }
    }
}

impl ShaderTemplateReflection {
    pub(crate) fn validate_vertex_input_stage_interface(
        &self,
        vertex_entry_name: &str,
        mut provided_scalar_kind: impl FnMut(u32) -> Option<ShaderVertexInputScalarKind>,
    ) -> Result<(), String> {
        let vertex_entry = self
            .entry_points
            .iter()
            .find(|entry| {
                entry.name == vertex_entry_name && entry.stage == naga::ShaderStage::Vertex
            })
            .ok_or_else(|| {
                format!("shader is missing required @vertex entry `{vertex_entry_name}`")
            })?;

        for input in vertex_entry.inputs.iter().filter_map(stage_location) {
            let location = input.location;
            let Some(provided) = provided_scalar_kind(location) else {
                return Err(format!(
                    "shader vertex input interface mismatch for @vertex `{vertex_entry_name}`: \
                     @location({location}) is required by the shader but is not provided by the \
                     vertex layout"
                ));
            };
            let Some(required) = input
                .numeric_type
                .and_then(|ty| ty.vertex_input_scalar_kind())
            else {
                return Err(format!(
                    "shader vertex input interface mismatch for @vertex `{vertex_entry_name}`: \
                     @location({location}) has an unsupported scalar kind"
                ));
            };
            if required != provided {
                return Err(format!(
                    "shader vertex input interface mismatch for @vertex `{vertex_entry_name}`: \
                     @location({location}) requires {required:?} but the vertex layout provides \
                     {provided:?}"
                ));
            }
            if input.per_primitive {
                return Err(format!(
                    "shader vertex input interface mismatch for @vertex `{vertex_entry_name}`: \
                     @location({location}) cannot be per-primitive for a vertex-stage input"
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn validate_vertex_fragment_stage_interface(
        &self,
        vertex_entry_name: &str,
        fragment_entry_name: &str,
    ) -> Result<(), String> {
        let vertex_entry = self
            .entry_points
            .iter()
            .find(|entry| {
                entry.name == vertex_entry_name && entry.stage == naga::ShaderStage::Vertex
            })
            .ok_or_else(|| {
                format!("shader is missing required @vertex entry `{vertex_entry_name}`")
            })?;
        let fragment_entry = self
            .entry_points
            .iter()
            .find(|entry| {
                entry.name == fragment_entry_name && entry.stage == naga::ShaderStage::Fragment
            })
            .ok_or_else(|| {
                format!("shader is missing required @fragment entry `{fragment_entry_name}`")
            })?;

        validate_stage_locations(vertex_entry, fragment_entry).map_err(|message| {
            format!(
                "shader stage interface mismatch between @vertex `{vertex_entry_name}` and \
                 @fragment `{fragment_entry_name}`: {message}"
            )
        })
    }

    pub(crate) fn validate_fragment_output_stage_interface(
        &self,
        fragment_entry_name: &str,
        mut target_numeric_type: impl FnMut(u32) -> Option<ShaderFragmentOutputNumericType>,
    ) -> Result<(), String> {
        let fragment_entry = self
            .entry_points
            .iter()
            .find(|entry| {
                entry.name == fragment_entry_name && entry.stage == naga::ShaderStage::Fragment
            })
            .ok_or_else(|| {
                format!("shader is missing required @fragment entry `{fragment_entry_name}`")
            })?;

        for output in fragment_entry.outputs.iter().filter_map(stage_location) {
            let Some(target_type) = target_numeric_type(output.location) else {
                continue;
            };
            let Some(shader_type) = output.numeric_type else {
                return Err(format!(
                    "shader fragment output interface mismatch for @fragment \
                     `{fragment_entry_name}`: @location({}) has an unsupported numeric type",
                    output.location
                ));
            };
            if !target_type.numeric_type().is_subtype_of(shader_type) {
                return Err(format!(
                    "shader fragment output interface mismatch for @fragment \
                     `{fragment_entry_name}`: @location({}) target type {target_type:?} is not \
                     covered by shader output {shader_type:?}",
                    output.location
                ));
            }
        }

        Ok(())
    }
}

pub(super) fn reflect_stage_io_numeric_type(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
) -> Option<ShaderStageIoNumericType> {
    let (dimension, scalar) = match module.types[ty].inner {
        naga::TypeInner::Scalar(scalar) => (ShaderStageIoNumericDimension::Scalar, scalar),
        naga::TypeInner::Vector { size, scalar } => {
            (ShaderStageIoNumericDimension::Vector(size), scalar)
        }
        naga::TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => (ShaderStageIoNumericDimension::Matrix(columns, rows), scalar),
        _ => return None,
    };
    Some(ShaderStageIoNumericType { dimension, scalar })
}

fn validate_stage_locations(
    vertex_entry: &ShaderEntryPointReflection,
    fragment_entry: &ShaderEntryPointReflection,
) -> Result<(), String> {
    let mut vertex_outputs = vertex_entry.outputs.iter().filter_map(stage_location);
    let mut vertex_output = vertex_outputs.next();

    for fragment_input in fragment_entry.inputs.iter().filter_map(stage_location) {
        while vertex_output.is_some_and(|output| output.location < fragment_input.location) {
            vertex_output = vertex_outputs.next();
        }
        let Some(output) =
            vertex_output.filter(|output| output.location == fragment_input.location)
        else {
            return Err(format!(
                "@location({}) is required by the fragment entry but is not produced by the \
                 vertex entry",
                fragment_input.location
            ));
        };
        validate_stage_location(output, fragment_input)?;
    }

    Ok(())
}

fn validate_stage_location(
    vertex_output: ShaderStageLocation,
    fragment_input: ShaderStageLocation,
) -> Result<(), String> {
    let location = fragment_input.location;
    if fragment_input.interpolation != vertex_output.interpolation {
        return Err(format!(
            "@location({location}) interpolation {:?} does not match vertex output {:?}",
            fragment_input.interpolation, vertex_output.interpolation
        ));
    }
    if fragment_input.sampling != vertex_output.sampling {
        return Err(format!(
            "@location({location}) sampling {:?} does not match vertex output {:?}",
            fragment_input.sampling, vertex_output.sampling
        ));
    }
    let Some(fragment_type) = fragment_input.numeric_type else {
        return Err(format!(
            "@location({location}) fragment input has an unsupported numeric type"
        ));
    };
    let Some(vertex_type) = vertex_output.numeric_type else {
        return Err(format!(
            "@location({location}) vertex output has an unsupported numeric type"
        ));
    };
    if !fragment_type.is_subtype_of(vertex_type) {
        return Err(format!(
            "@location({location}) fragment numeric type {fragment_type:?} is not compatible \
             with vertex output {vertex_type:?}"
        ));
    }
    if fragment_input.per_primitive != vertex_output.per_primitive {
        return Err(format!(
            "@location({location}) per_primitive={} does not match vertex output {}",
            fragment_input.per_primitive, vertex_output.per_primitive
        ));
    }
    Ok(())
}

impl ShaderStageIoNumericType {
    fn vertex_input_scalar_kind(self) -> Option<ShaderVertexInputScalarKind> {
        match self.scalar.kind {
            naga::ScalarKind::Sint => Some(ShaderVertexInputScalarKind::Sint),
            naga::ScalarKind::Uint => Some(ShaderVertexInputScalarKind::Uint),
            naga::ScalarKind::Float => Some(ShaderVertexInputScalarKind::Float),
            naga::ScalarKind::Bool
            | naga::ScalarKind::AbstractInt
            | naga::ScalarKind::AbstractFloat => None,
        }
    }

    fn is_subtype_of(self, provided: Self) -> bool {
        if self.scalar.width > provided.scalar.width || self.scalar.kind != provided.scalar.kind {
            return false;
        }
        match (self.dimension, provided.dimension) {
            (ShaderStageIoNumericDimension::Scalar, ShaderStageIoNumericDimension::Scalar)
            | (ShaderStageIoNumericDimension::Scalar, ShaderStageIoNumericDimension::Vector(_)) => {
                true
            }
            (
                ShaderStageIoNumericDimension::Vector(input),
                ShaderStageIoNumericDimension::Vector(output),
            ) => input <= output,
            (
                ShaderStageIoNumericDimension::Matrix(input_columns, input_rows),
                ShaderStageIoNumericDimension::Matrix(output_columns, output_rows),
            ) => input_columns == output_columns && input_rows == output_rows,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
struct ShaderStageLocation {
    location: u32,
    interpolation: Option<naga::Interpolation>,
    sampling: Option<naga::Sampling>,
    per_primitive: bool,
    numeric_type: Option<ShaderStageIoNumericType>,
}

fn stage_location(value: &ShaderStageIoReflection) -> Option<ShaderStageLocation> {
    let naga::Binding::Location {
        location,
        interpolation,
        sampling,
        per_primitive,
        blend_src: _,
    } = value.binding.as_ref()?
    else {
        return None;
    };
    Some(ShaderStageLocation {
        location: *location,
        interpolation: *interpolation,
        sampling: *sampling,
        per_primitive: *per_primitive,
        numeric_type: value.numeric_type,
    })
}
