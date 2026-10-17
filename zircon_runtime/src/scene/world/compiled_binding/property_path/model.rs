//! 场景属性的运行时身份、已解析目标与字段分派缓存。
//!
//! 返回对象拥有所需值，不持有 World 借用；驻留编号及修订随编译用 World 解释。
//! 保留缓存不会阻止 World 变更，消费入口仍须使用新鲜度检查。

use crate::core::framework::scene::ComponentPropertyPath;
use crate::scene::{EntityId, World};

use super::super::compiled_scene_animation_fields::CompiledAnimationRuntimeProperty;
use super::super::compiled_scene_camera_light_fields::{
    CompiledCameraProperty, CompiledLightProperty,
};
use super::super::compiled_scene_dynamic_field::CompiledDynamicProperty;

/// 所属 World 驻留表中的实体路径编号；复制编号不复制或延长驻留表的生命周期。
///
/// Runtime-owned identity for canonical entity-path text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathId(pub(in crate::scene::world::compiled_binding) u64);

/// 所属 World 驻留表中的字段键编号；内建键已规范化，动态模式键保留原始大小写。
///
/// Runtime-owned identity for canonical component-property text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentFieldId(pub(in crate::scene::world::compiled_binding) u64);

/// 拥有实体、层级根与编译时根修订的目标快照，不持有 World 借用。
///
/// 新鲜度检查比较传入 World 中的实体存在性和根修订，不校验字段模式或访问权限，
/// 也不比较跨 World 的身份令牌。替换流程通过续接旧修订令原缓存失效，见
/// `LevelSystem::replace_world_and_reset_runtime_state` 与 `World::commit_staged_scene_state`。
///
/// A generation-validated entity/property target compiled by a [`World`].
///
/// The raw DTOs remain the serde and diagnostic owners. Runtime owns their
/// identity and the resolved entity so consumers can retain this target without
/// rebuilding path strings every frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledScenePropertyTarget {
    entity: EntityId,
    root: EntityId,
    path_id: PathId,
    component_field_id: ComponentFieldId,
    /// 编译时根的拓扑绑定修订，与整体 World 修订和动态模式修订分别计数。
    generation: u64,
}

/// 内建变换字段；轴变体由下方键匹配限定到相应向量或四元数的有效轴。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledTransformProperty {
    Translation,
    TranslationAxis(usize),
    Rotation,
    RotationAxis(usize),
    Scale,
    ScaleAxis(usize),
}

/// 内建网格字段；形态权重索引在此仅作为编号保存，实际数组读取或扩容由消费入口处理。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompiledMeshRendererProperty {
    RenderQueue,
    MaterialQueue,
    OrderInLayer,
    DepthBias,
    MorphWeight(usize),
    Tint,
}

/// 编译时选择的字段分派；动态分支另外拥有其精确模式键与失效快照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CompiledScenePropertyWriterKind {
    Transform(CompiledTransformProperty),
    MeshRenderer(CompiledMeshRendererProperty),
    AnimationRuntime(CompiledAnimationRuntimeProperty),
    Camera(CompiledCameraProperty),
    Light(CompiledLightProperty),
    Dynamic(CompiledDynamicProperty),
}

impl CompiledTransformProperty {
    /// 匹配预先规范化的内建键与轴别名，成功时只返回固定字段及轴编号。
    fn from_canonical_key(key: &str) -> Option<Self> {
        match key {
            "transform.translation" => Some(Self::Translation),
            "transform.translation.x" | "transform.translation.0" => Some(Self::TranslationAxis(0)),
            "transform.translation.y" | "transform.translation.1" => Some(Self::TranslationAxis(1)),
            "transform.translation.z" | "transform.translation.2" => Some(Self::TranslationAxis(2)),
            "transform.rotation" => Some(Self::Rotation),
            "transform.rotation.x" | "transform.rotation.0" => Some(Self::RotationAxis(0)),
            "transform.rotation.y" | "transform.rotation.1" => Some(Self::RotationAxis(1)),
            "transform.rotation.z" | "transform.rotation.2" => Some(Self::RotationAxis(2)),
            "transform.rotation.w" | "transform.rotation.3" => Some(Self::RotationAxis(3)),
            "transform.scale" => Some(Self::Scale),
            "transform.scale.x" | "transform.scale.0" => Some(Self::ScaleAxis(0)),
            "transform.scale.y" | "transform.scale.1" => Some(Self::ScaleAxis(1)),
            "transform.scale.z" | "transform.scale.2" => Some(Self::ScaleAxis(2)),
            _ => None,
        }
    }
}

impl CompiledScenePropertyWriterKind {
    /// 按既定顺序解析内建字段及形态权重索引；不在此解析动态字段或访问组件实例。
    /// 动态字段的后备编译由 `World::compile_scene_property_writer_kind` 处理。
    pub(super) fn from_canonical_key(key: &str) -> Option<Self> {
        if let Some(property) = CompiledTransformProperty::from_canonical_key(key) {
            return Some(Self::Transform(property));
        }

        match key {
            "meshrenderer.renderqueue" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::RenderQueue,
                ));
            }
            "meshrenderer.materialqueue" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::MaterialQueue,
                ));
            }
            "meshrenderer.orderinlayer" => {
                return Some(Self::MeshRenderer(
                    CompiledMeshRendererProperty::OrderInLayer,
                ));
            }
            "meshrenderer.depthbias" => {
                return Some(Self::MeshRenderer(CompiledMeshRendererProperty::DepthBias));
            }
            "meshrenderer.tint" => {
                return Some(Self::MeshRenderer(CompiledMeshRendererProperty::Tint));
            }
            _ => {}
        }

        if let Some(property) = CompiledCameraProperty::from_canonical_key(key) {
            return Some(Self::Camera(property));
        }
        if let Some(property) = CompiledLightProperty::from_canonical_key(key) {
            return Some(Self::Light(property));
        }
        if let Some(property) = CompiledAnimationRuntimeProperty::from_canonical_key(key) {
            return Some(Self::AnimationRuntime(property));
        }

        let index = key
            .strip_prefix("meshrenderer.morphweights.")?
            .parse()
            .ok()?;
        Some(Self::MeshRenderer(
            CompiledMeshRendererProperty::MorphWeight(index),
        ))
    }

    /// 内建分支在此直接视为当前，动态分支委托其模式或目录修订检查。
    /// 实体与拓扑条件由外层目标检查，二者在公开 writer 检查中合并。
    fn is_current_for(&self, world: &World) -> bool {
        match self {
            Self::Dynamic(property) => property.is_current_for(world),
            Self::Transform(_)
            | Self::MeshRenderer(_)
            | Self::AnimationRuntime(_)
            | Self::Camera(_)
            | Self::Light(_) => true,
        }
    }
}

/// 拥有字段分派、目标快照及用于错误诊断的原始路径；克隆后仍指向同一组运行时身份。
///
/// [`World::read_compiled_scene_property`] 与 [`World::write_compiled_scene_property`] 在使用前检查
/// 缓存条件；发生过期时由消费方在编译边界重建，缓存本身不持有 World 借用。
///
/// A typed scene-property writer compiled at an import or edit boundary.
///
/// The writer retains the raw DTO solely for typed errors. Its steady-state
/// read/write path uses a resolved entity plus a precompiled field variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledScenePropertyWriter {
    target: CompiledScenePropertyTarget,
    pub(super) property_path: ComponentPropertyPath,
    pub(super) property: CompiledScenePropertyWriterKind,
}

impl CompiledScenePropertyWriter {
    /// 接收已准备的目标、路径和字段分派并取得其所有权；不再执行查找或准入验证。
    pub(super) fn new(
        target: CompiledScenePropertyTarget,
        property_path: ComponentPropertyPath,
        property: CompiledScenePropertyWriterKind,
    ) -> Self {
        Self {
            target,
            property_path,
            property,
        }
    }

    /// 返回编译时缓存的实体编号，不重新确认实体当前是否存在。
    pub const fn entity(&self) -> EntityId {
        self.target.entity()
    }

    pub(in crate::scene::world::compiled_binding) fn property_path(
        &self,
    ) -> &ComponentPropertyPath {
        &self.property_path
    }

    /// 合并目标存在性、根修订，以及动态分支的模式或目录修订检查。
    /// 返回 `true` 只表示这些缓存条件匹配；它不读取字段值，也不替代写入准入。
    pub fn is_current_for(&self, world: &World) -> bool {
        self.target.is_current_for(world) && self.property.is_current_for(world)
    }
}

impl CompiledScenePropertyTarget {
    pub(in crate::scene::world::compiled_binding) const fn new(
        entity: EntityId,
        root: EntityId,
        path_id: PathId,
        component_field_id: ComponentFieldId,
        generation: u64,
    ) -> Self {
        Self {
            entity,
            root,
            path_id,
            component_field_id,
            generation,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn path_id(&self) -> PathId {
        self.path_id
    }

    pub const fn component_field_id(&self) -> ComponentFieldId {
        self.component_field_id
    }

    /// 检查缓存实体与根仍存在，且根修订与快照一致；不检查字段定义或跨 World 身份。
    /// 动态字段的模式或目录修订由外层 [`CompiledScenePropertyWriter::is_current_for`] 追加检查。
    pub fn is_current_for(&self, world: &World) -> bool {
        world.contains_entity(self.entity)
            && world.contains_entity(self.root)
            && self.generation == world.scene_binding_generation(self.root)
    }
}
