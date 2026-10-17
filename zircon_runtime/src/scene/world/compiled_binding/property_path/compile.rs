//! 在导入或编辑边界解析场景属性目标及字段分派。
//!
//! 编译入口准备运行时身份和失效快照；稳定帧读写复用缓存，不在这里接纳待写入的值。
//! 驻留编号耗尽会 panic，不转换为 SceneResult；编号与修订只供运行时缓存解释。

use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::scene::{EntityId, SceneError, SceneResult, World};

use super::super::compiled_scene_dynamic_field::CompiledDynamicProperty;
use super::model::{
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, CompiledScenePropertyWriterKind,
};

impl World {
    /// 解析实体路径并创建拥有值的目标快照，同时驻留路径身份与字段身份。
    ///
    /// # 参数与返回
    /// `entity_path` 用于查找实体，`property_path` 仅用于准备字段身份；找不到实体时返回 `None`。
    /// 此入口不验证字段是否受 writer 支持。返回值不保留两个输入或 World 的借用，
    /// 其有效性通过 [`Self::scene_binding_generation`] 及目标的新鲜度检查解释。
    ///
    /// Compiles a scene-path target at an import or edit boundary.
    ///
    /// A retained target remains valid until the resolved entity's hierarchy or
    /// a name in its hierarchy changes. The typed field dispatch is added by the
    /// property-access cutover; this boundary intentionally does not fall back to
    /// per-frame path resolution.
    pub fn compile_scene_property_target(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> Option<CompiledScenePropertyTarget> {
        let entity = self.get_entity_by_path(entity_path)?;
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        Some(
            self.compile_scene_property_target_for_entity_with_canonical_field(
                entity,
                entity_path.as_str(),
                &component_field_key,
            ),
        )
    }

    /// 为已解析实体记录根与修订，并驻留调用方给出的路径身份及已准备字段键。
    /// 本函数不重查路径是否对应 `entity`；调用方负责提供一致的实体与身份文本。
    fn compile_scene_property_target_for_entity_with_canonical_field(
        &mut self,
        entity: EntityId,
        entity_identity: &str,
        component_field_key: &str,
    ) -> CompiledScenePropertyTarget {
        let root = self.scene_binding_root(entity);
        let generation = self.scene_binding_generation(root);
        let path_id = self.scene_binding_generations.intern_path(entity_identity);
        let component_field_id = self
            .scene_binding_generations
            .intern_component_field(component_field_key);

        self.record_scene_property_target_compilation();
        CompiledScenePropertyTarget::new(entity, root, path_id, component_field_id, generation)
    }

    /// 在编译边界准备可复用的字段 writer；返回对象拥有目标、分派及路径值。
    ///
    /// # 参数与返回
    /// 两个路径分别描述目标实体和字段。先解析字段分派，不支持的字段返回 `UnknownProperty`；
    /// 字段可编译后才查实体，实体缺失返回 `Ok(None)`，否则返回 `Ok(Some(...))`。
    /// 具体值转换与组件存在性在消费时检查；动态字段的可编辑性与按登记条件执行的模式值验证由动态写入入口判断。
    ///
    /// Compiles a typed scene-property writer at an import or edit boundary.
    ///
    /// Missing entities remain an optional lookup. Unsupported fields fail at
    /// the compile boundary and never fall back to the generic string visitor
    /// during steady-state application.
    pub fn compile_scene_property_writer(
        &mut self,
        entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        let property =
            self.compile_scene_property_writer_kind(property_path, &component_field_key)?;
        let Some(entity) = self.get_entity_by_path(entity_path) else {
            return Ok(None);
        };
        let target = self.compile_scene_property_target_for_entity_with_canonical_field(
            entity,
            entity_path.as_str(),
            &component_field_key,
        );
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    /// 复用调用方已解析的 `entity` 与其 `canonical_entity_path`；二者对应关系由调用方维护。
    /// 字段分派失败先返回错误，之后实体存在性检查才可能返回 `Ok(None)`；成功返回拥有值的 writer。
    /// 动画序列编译先取得实体的实际路径再进入此入口，见 `compile_sequence_for_world`。
    ///
    /// Compiles a writer after an import boundary has already resolved its
    /// target entity and canonicalized its path. The path is interned once for
    /// stable runtime identity and never parsed during writer application.
    pub(crate) fn compile_scene_property_writer_for_entity(
        &mut self,
        entity: EntityId,
        canonical_entity_path: &EntityPath,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<Option<CompiledScenePropertyWriter>> {
        let component_field_key = Self::canonical_component_field_key(property_path);
        self.record_scene_property_canonicalization(component_field_key.len());
        let property =
            self.compile_scene_property_writer_kind(property_path, &component_field_key)?;
        if !self.contains_entity(entity) {
            return Ok(None);
        }
        let target = self.compile_scene_property_target_for_entity_with_canonical_field(
            entity,
            canonical_entity_path.as_str(),
            &component_field_key,
        );
        Ok(Some(CompiledScenePropertyWriter::new(
            target,
            property_path.clone(),
            property,
        )))
    }

    /// 先匹配规范化内建键，再用原始路径尝试动态字段；均不成立时构造 `UnknownProperty`。
    /// 这里只选择分派并记录诊断，不验证实例中是否已有对应组件或字段。
    fn compile_scene_property_writer_kind(
        &self,
        property_path: &ComponentPropertyPath,
        component_field_key: &str,
    ) -> SceneResult<CompiledScenePropertyWriterKind> {
        self.record_scene_property_field_dispatch_compilation();
        CompiledScenePropertyWriterKind::from_canonical_key(component_field_key)
            .or_else(|| {
                CompiledDynamicProperty::compile(self, property_path)
                    .map(CompiledScenePropertyWriterKind::Dynamic)
            })
            .ok_or_else(|| SceneError::UnknownProperty {
                property_path: property_path.to_string(),
            })
    }

    /// 沿父链解析编译目标的根；遍历次数受实体数量约束以避免损坏链导致无限循环。
    /// 此界限不构成层级合法性检查；正常返回顶层根依赖 World 的层级不变量。
    fn scene_binding_root(&self, entity: EntityId) -> EntityId {
        let mut root = entity;
        let mut remaining = self.entities.len().saturating_add(1);
        while remaining > 0 {
            let Some(parent) = self.parent_of(root) else {
                break;
            };
            root = parent;
            remaining -= 1;
        }
        root
    }
}
