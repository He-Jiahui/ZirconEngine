use super::RuntimePluginCatalog;

// 聚合 diagnostics 为空才表示 catalog 构建成功；update.publish 对候选也要求诊断为空才替换已发布数据。
impl RuntimePluginCatalog {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
