//! 项目工作区pane实例的注册表恢复入口；宿主先解码版本化文档，再逐项导入，最后规范化布局。
use super::{ViewInstance, ViewRegistry};

impl ViewRegistry {
    /// 只接受已注册且能力满足的种类；恢复身份还必须与既有实例和单例索引保持一致。
    pub fn restore_instance(&mut self, instance: ViewInstance) -> Result<ViewInstance, String> {
        let Some(descriptor) = self.descriptors.get(&instance.descriptor_id) else {
            return Err(format!(
                "cannot restore missing descriptor {}",
                instance.descriptor_id.0
            ));
        };
        if let Some(error) = self.descriptor_capability_error(descriptor) {
            return Err(error);
        }
        let multi_instance = descriptor.multi_instance;
        self.update_counter(&instance);
        if !multi_instance {
            self.single_instance_index
                .insert(instance.descriptor_id.clone(), instance.instance_id.clone());
        }
        // BUG: [CR-EDITOR-WORKBENCH-0001] 相同实例ID可被另一描述符覆盖，旧单例索引仍指向该ID；
        // 后续打开旧描述符可能返回新描述符的实例。证据：项目快照逐项恢复与open_descriptor索引复用。
        self.instances
            .insert(instance.instance_id.clone(), instance.clone());
        Ok(instance)
    }
}
