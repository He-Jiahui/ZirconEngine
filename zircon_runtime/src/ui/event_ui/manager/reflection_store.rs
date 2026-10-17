use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{diff::compute_diff, UiEventManager};
use zircon_runtime_interface::ui::event_ui::{
    UiInvocationError, UiNodeDescriptor, UiNodePath, UiNotification, UiPropertyDescriptor,
    UiReflectionDiff, UiReflectionNodePatch, UiReflectionSnapshot, UiTreeId,
};

#[cfg(test)]
#[path = "reflection_store/tests/perf.rs"]
mod optimization_tests;

impl UiEventManager {
    /// 接管一棵树的完整反射快照，重建路径索引并返回本次差异。
    ///
    /// # 参数与返回
    /// `snapshot` 的树编号决定替换目标；首次登记时将其全部节点报告为变化节点。
    /// 替换已有快照时按节点描述比较；非空差异另行广播，返回值由调用者拥有。
    ///
    /// 此处只保存反射数据，不创建真实控件；路径冲突按索引遍历的最后写入者覆盖。
    pub fn replace_tree(&mut self, snapshot: UiReflectionSnapshot) -> UiReflectionDiff {
        let tree_id = snapshot.tree_id.clone();
        let diff = if let Some(previous) = self.trees.insert(tree_id.clone(), snapshot) {
            compute_diff(&previous, self.trees.get(&tree_id).expect("replaced tree"))
        } else {
            UiReflectionDiff {
                tree_id: tree_id.clone(),
                changed_nodes: self
                    .trees
                    .get(&tree_id)
                    .map(|tree| tree.nodes.keys().copied().collect())
                    .unwrap_or_default(),
                removed_nodes: Vec::new(),
            }
        };
        self.rebuild_node_index();
        if !diff.is_empty() {
            self.broadcast(UiNotification::ReflectionDiff(diff.clone()));
        }
        diff
    }

    /// 按树编号返回当前反射快照的独立副本，缺失时返回 `None`。
    ///
    /// 返回值不借用管理器；修改副本不会更新已保存的快照。
    pub fn query_tree(&self, tree_id: &UiTreeId) -> Option<UiReflectionSnapshot> {
        self.trees.get(tree_id).cloned()
    }

    /// 按节点路径索引返回节点描述的独立副本，定位不到时返回 `None`。
    ///
    /// 修改返回值不会回写快照；本次查询不重新扫描其他树寻找同名路径。
    pub fn query_node(&self, node_path: &UiNodePath) -> Option<UiNodeDescriptor> {
        let (tree_id, node_id) = self.node_index.get(node_path)?;
        self.trees
            .get(tree_id)
            .and_then(|tree| tree.nodes.get(node_id))
            .cloned()
    }

    /// 按节点路径和属性名查询单个属性，任一级不存在时返回 `None`。
    ///
    /// 仅克隆命中的属性描述；返回副本不借用节点，也不会回写管理器。
    pub fn query_property(
        &self,
        node_path: &UiNodePath,
        property_name: &str,
    ) -> Option<UiPropertyDescriptor> {
        let (tree_id, node_id) = self.node_index.get(node_path)?;
        self.trees
            .get(tree_id)
            .and_then(|tree| tree.nodes.get(node_id))
            .and_then(|node| node.properties.get(property_name))
            .cloned()
    }

    /// 更新管理器快照中的可写属性，并发送该节点的差异通知。
    ///
    /// # 参数与返回
    /// `node_path` 与 `property_name` 定位当前索引中的属性；`value` 移入快照。
    /// 目标缺失或属性不可写时返回相应错误，成功返回 `Ok(())`。
    ///
    /// # 约束
    /// 此处不按属性声明类型验证 JSON 值，也不向真实 UI 控件执行写入。
    /// 即使新旧值相等也会替换并广播；批量内部更新参见 [`Self::apply_reflection_patches`]。
    pub fn set_property(
        &mut self,
        node_path: UiNodePath,
        property_name: String,
        value: Value,
    ) -> Result<(), UiInvocationError> {
        let (tree_id, node_id) = self
            .node_index
            .get(&node_path)
            .cloned()
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let tree = self
            .trees
            .get_mut(&tree_id)
            .ok_or_else(|| UiInvocationError::UnknownTree(tree_id.0.clone()))?;
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let property = node.properties.get_mut(&property_name).ok_or_else(|| {
            UiInvocationError::UnknownProperty {
                node_path: node_path.0.clone(),
                property_name: property_name.clone(),
            }
        })?;
        if !property.writable {
            return Err(UiInvocationError::PropertyNotWritable {
                node_path: node_path.0.clone(),
                property_name,
            });
        }
        property.reflected_value = value;
        let diff = UiReflectionDiff {
            tree_id,
            changed_nodes: vec![node_id],
            removed_nodes: Vec::new(),
        };
        self.broadcast(UiNotification::ReflectionDiff(diff));
        Ok(())
    }

    /// 将内部反射补丁应用到已保存快照，返回并广播实际有变化的节点集合。
    ///
    /// # 参数与返回
    /// `patches` 只在调用期间借用；属性值在写入快照时克隆。
    /// 成功返回按树编号分组、节点编号去重的差异，未变化的补丁不产生差异。
    ///
    /// # 约束与顺序
    /// 先验证全部节点定位及属性名，再开始修改；返回定位错误时不会提交本批属性或按压状态。
    /// 本同步路径不检查属性可写标志或 JSON 类型；`pressed` 仅在补丁提供该值时更新。
    /// 补丁只更新反射数据，不执行真实控件动作或注册路由。
    pub fn apply_reflection_patches(
        &mut self,
        patches: &[UiReflectionNodePatch],
    ) -> Result<Vec<UiReflectionDiff>, UiInvocationError> {
        // 解析阶段借用索引中的树编号，完整验证后才进入可变访问，避免逐补丁先写后报错。
        let mut resolved = Vec::with_capacity(patches.len());
        for (patch_index, patch) in patches.iter().enumerate() {
            let (tree_id, node_id) = self
                .node_index
                .get(&patch.node_path)
                .ok_or_else(|| UiInvocationError::UnknownNode(patch.node_path.0.clone()))?;
            let tree = self
                .trees
                .get(tree_id)
                .ok_or_else(|| UiInvocationError::UnknownTree(tree_id.0.clone()))?;
            let node = tree
                .nodes
                .get(node_id)
                .ok_or_else(|| UiInvocationError::UnknownNode(patch.node_path.0.clone()))?;
            for property_name in patch.properties.keys() {
                if !node.properties.contains_key(property_name) {
                    return Err(UiInvocationError::UnknownProperty {
                        node_path: patch.node_path.0.clone(),
                        property_name: property_name.clone(),
                    });
                }
            }
            resolved.push((tree_id, *node_id, patch_index));
        }

        let mut changed_by_tree = BTreeMap::<UiTreeId, BTreeSet<_>>::new();
        for (tree_id, node_id, patch_index) in resolved {
            let patch = &patches[patch_index];
            let node = self
                .trees
                .get_mut(tree_id)
                .and_then(|tree| tree.nodes.get_mut(&node_id))
                .expect("reflection patch targets were validated before mutation");
            let mut changed = false;
            for (property_name, value) in &patch.properties {
                let property = node
                    .properties
                    .get_mut(property_name)
                    .expect("reflection patch properties were validated before mutation");
                if property.reflected_value != *value {
                    property.reflected_value = value.clone();
                    changed = true;
                }
            }
            if let Some(pressed) = patch.pressed {
                if node.state_flags.pressed != pressed {
                    node.state_flags.pressed = pressed;
                    changed = true;
                }
            }
            if changed {
                changed_by_tree
                    .entry(tree_id.clone())
                    .or_default()
                    .insert(node_id);
            }
        }

        // 有序集合将同一节点的多次变化合并，通知只要求调用者重查最终快照。
        let diffs = changed_by_tree
            .into_iter()
            .map(|(tree_id, changed_nodes)| UiReflectionDiff {
                tree_id,
                changed_nodes: changed_nodes.into_iter().collect(),
                removed_nodes: Vec::new(),
            })
            .collect::<Vec<_>>();
        for diff in &diffs {
            self.broadcast(UiNotification::ReflectionDiff(diff.clone()));
        }
        Ok(diffs)
    }

    /// 整体替换快照后重建路径索引，并复用哈希表已有存储。
    ///
    /// 重复路径由按树编号、节点编号遍历时最后写入者覆盖；此处不检查路径唯一性。
    pub(crate) fn rebuild_node_index(&mut self) {
        let node_count = self
            .trees
            .values()
            .map(|tree| tree.nodes.len())
            .sum::<usize>();
        self.node_index.clear();
        let additional_capacity = node_count.saturating_sub(self.node_index.capacity());
        if additional_capacity > 0 {
            self.node_index.reserve(additional_capacity);
        }
        for (tree_id, tree) in &self.trees {
            for (node_id, node) in &tree.nodes {
                self.node_index
                    .insert(node.node_path.clone(), (tree_id.clone(), *node_id));
            }
        }
    }
}
