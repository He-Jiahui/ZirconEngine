use serde::{Deserialize, Serialize};

/// 组件的注册期存储选择：Table 随原型行迁移，SparseSet 由内部实体句柄单独定位。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageType {
    #[default]
    Table,
    SparseSet,
}
