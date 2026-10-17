use std::collections::{BTreeMap, HashMap};

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::error::{SceneError, SceneResult};

#[cfg(test)]
#[path = "tests/schema_generation_hash.rs"]
mod schema_generation_hash_tests;

/// 描述符按类型路径有序存放，代数索引用于判断缓存是否需要重新绑定。
///
/// 克隆会复制目录与代数；相等比较只看描述符内容，不比较运行时缓存代数。
/// 组件编号及反射字段槽位分别属于其它目录，本目录不分配这两类编号。
#[derive(Clone, Debug, Default)]
/// 插件动态组件描述符的 World 本地目录；每个类型及总目录的代数用于缓存失效与反射模式同步。
pub struct ComponentTypeRegistry {
    /// 以完整类型路径为键的描述符；迭代顺序由键的字典序决定。
    descriptors: BTreeMap<String, ComponentTypeDescriptor>,
    /// 全目录共用的饱和递增代数，每次实际模式变更用其新值标记对应路径。
    next_schema_generation: u64,
    /// 每个路径最近一次模式变更的代数；移除描述符后仍保留该记录。
    schema_generations: HashMap<String, u64>,
}

// Schema revisions are runtime cache metadata and do not change persistent
// component-descriptor equality.
impl PartialEq for ComponentTypeRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.descriptors == other.descriptors
    }
}

impl Eq for ComponentTypeRegistry {}

impl ComponentTypeRegistry {
    /// 只读检查插件名前缀加点分隔符和路径重名，不验证属性内容或发布目录。
    /// World 将此预检与反射、ECS 参与方的检查组合后再提交。
    pub(super) fn validate_new_descriptor(
        &self,
        descriptor: &ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
            return Err(SceneError::ComponentTypePluginPrefixMismatch {
                type_id: descriptor.type_id.clone(),
                plugin_id: descriptor.plugin_id.clone(),
            });
        }
        if self.descriptors.contains_key(&descriptor.type_id) {
            return Err(SceneError::DuplicateComponentType {
                type_id: descriptor.type_id.clone(),
            });
        }
        Ok(())
    }

    /// 接管并登记新描述符；前缀错误或路径已存在时返回错误，目录和代数不变。
    /// 成功后推进该路径及整个目录的模式代数；字段声明的合法性由更高层入口检查。
    pub fn register(&mut self, descriptor: ComponentTypeDescriptor) -> SceneResult<()> {
        self.validate_new_descriptor(&descriptor)?;
        let type_id = descriptor.type_id.clone();
        self.descriptors.insert(type_id.clone(), descriptor);
        self.advance_schema_generation(&type_id);
        Ok(())
    }

    /// 接管已通过新描述符检查的候选，供 World 的整体预检提交阶段使用。
    /// 调用方须确保目录自预检后未出现同名项；调试断言不是可恢复的错误处理。
    pub(super) fn publish_prevalidated(&mut self, descriptor: ComponentTypeDescriptor) {
        let type_id = descriptor.type_id.clone();
        debug_assert!(self.validate_new_descriptor(&descriptor).is_ok());
        let previous = self.descriptors.insert(type_id.clone(), descriptor);
        debug_assert!(previous.is_none());
        self.advance_schema_generation(&type_id);
    }

    /// 按完整路径借用描述符；未登记时返回 `None`，引用受目录共享借用期约束。
    pub fn descriptor(&self, type_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(type_id)
    }

    /// Returns the revision for the declared dynamic component schema.
    ///
    /// A compiled dynamic-field writer binds this revision at compile time so a
    /// registration refresh invalidates only writers for that component type.
    /// 未产生该路径的模式代数时返回零；移除后的路径仍返回最后一次变更代数。
    pub fn schema_generation(&self, type_id: &str) -> u64 {
        self.schema_generations
            .get(type_id)
            .copied()
            .unwrap_or_default()
    }

    /// Returns the revision for the schema catalog as a whole.
    ///
    /// This is only needed by a compiled field whose type was undeclared at
    /// compile time. Once a catalog appears, that writer must rebind rather
    /// than retain the former permissive empty-registry semantics.
    /// 返回全目录模式代数；实例值变化不会推进它，代数达到 u64 上限后保持饱和。
    pub fn schema_catalog_generation(&self) -> u64 {
        self.next_schema_generation
    }

    /// 在 VM 候选目录中新增或刷新描述符，同一完整内容重复提交时不推进代数。
    ///
    /// 插件前缀不符或既有同名项属于其它插件时返回错误；变化时推进总目录和该路径代数。
    /// 其它路径的代数保持不变；属性及 VM 字段约束由上层候选登记负责。
    pub(super) fn upsert_vm_descriptor(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        if !component_type_belongs_to_plugin(&descriptor.type_id, &descriptor.plugin_id) {
            return Err(SceneError::ComponentTypePluginPrefixMismatch {
                type_id: descriptor.type_id,
                plugin_id: descriptor.plugin_id,
            });
        }
        if let Some(existing) = self.descriptors.get(&descriptor.type_id) {
            if existing.plugin_id != descriptor.plugin_id {
                return Err(SceneError::DuplicateComponentType {
                    type_id: descriptor.type_id,
                });
            }
        }
        let type_id = descriptor.type_id.clone();
        if self.descriptors.get(&type_id) == Some(&descriptor) {
            return Ok(());
        }
        self.descriptors.insert(type_id.clone(), descriptor);
        self.advance_schema_generation(&type_id);
        Ok(())
    }

    /// 移除存在的 VM 描述符并留下新的路径代数；路径缺席时不推进任何代数。
    pub(super) fn remove_vm_descriptor(&mut self, type_id: &str) {
        if self.descriptors.remove(type_id).is_some() {
            self.advance_schema_generation(type_id);
        }
    }

    /// 按完整类型路径的字典序迭代借用描述符，迭代器不能跨目录可变借用保留。
    pub fn descriptors(&self) -> impl Iterator<Item = &ComponentTypeDescriptor> {
        self.descriptors.values()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn contains(&self, type_id: &str) -> bool {
        self.descriptors.contains_key(type_id)
    }

    /// 推进全目录饱和计数，并把同一新代数写入指定路径的变更记录。
    fn advance_schema_generation(&mut self, type_id: &str) {
        self.next_schema_generation = self.next_schema_generation.saturating_add(1);
        self.schema_generations
            .insert(type_id.to_string(), self.next_schema_generation);
    }
}

/// 只检查插件前缀和紧随的点分隔符，不解析插件标识语法或组件类型的其它段。
fn component_type_belongs_to_plugin(type_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = type_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}

#[cfg(test)]
#[path = "tests/component_type_registry.rs"]
mod tests;
