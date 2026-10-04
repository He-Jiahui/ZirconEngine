use std::sync::Arc;

use super::{VmBackend, VmError};

/// 家族只负责选择器命名与后端解析；限定名前缀由注册表分派，裸名回退要求各家族自行决定是否接受。
pub trait VmBackendFamily: Send + Sync {
    fn family_name(&self) -> &str;

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError>;

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str));
}
