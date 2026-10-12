// 有身份字段的静态行才纳入比较；必填字段缺失时应使测试失败，防止不完整声明被接受。
use super::super::super::storage::PendingOptionManifest;
use super::super::{output, signature};

impl PendingOptionManifest {
    pub(in super::super::super::super) fn push_into(
        &mut self,
        options: &mut Vec<zircon_runtime::plugin::PluginOptionManifest>,
    ) {
        let Some(signature) = signature::take_option_manifest_signature(self) else {
            self.enum_values.clear();
            return;
        };
        output::push_option_manifest(
            options,
            signature,
            std::mem::take(&mut self.enum_values),
            self.required_capability.take(),
        );
    }
}
