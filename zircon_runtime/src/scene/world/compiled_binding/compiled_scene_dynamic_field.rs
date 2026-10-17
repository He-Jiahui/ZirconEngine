//! 已编译动态字段的精确键与模式失效快照。
//!
//! 只记录编译时的失效条件；字段存在、可编辑性与值接纳由 World 消费入口判断。

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
    /// 从 `property_path` 的最后一处分隔点复制组件键与字段键，并记录当前模式修订。
    ///
    /// # 返回与生命周期
    /// Runtime 内建组件或空段返回 `None`；其他可分段键返回拥有字符串的缓存，不保留输入或 World 借用。
    /// 未声明类型也可形成缓存；仅模式修订为零时保存总目录修订，成功不表示字段已通过访问准入。
    /// 参考 `World::set_compiled_dynamic_component_property` 与外层 writer 的新鲜度检查。
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

    /// 模式修订非零时只比较本类型修订；为零时比较编译时保存的总目录修订。
    /// 因此无关模式登记不使前者过期，却会使后者重新经过编译边界；本函数不检查实体或字段权限。
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
