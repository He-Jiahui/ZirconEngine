//! 用不透明的 u64 包装区分 shape、body 与 constraint 句柄，并仅向公共 API 暴露原始值读取。

// TODO: [CR-PHYSICS-BACKEND-0006] 三种句柄只编码本地 index/generation，不含 backend/world 身份；独立实例可从同一 raw 值开始分配，跨实例误用可能命中另一实例对象。
// 需要定义实例归属合同或让句柄带 owner 身份。证据：backend/handle_pool.rs::Default/insert/get、lib.rs 公共导出；关联 PHY4-P1-016。
macro_rules! backend_handle {
    ($name:ident) => {
        /// 由创建它的物理 backend 管理的不透明对象句柄。
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(u64);

        impl $name {
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl ArenaHandle for $name {
            fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

pub(super) trait ArenaHandle: Copy {
    fn from_raw(raw: u64) -> Self;
    fn raw(self) -> u64;
}

backend_handle!(BodyHandle);
backend_handle!(ShapeHandle);
backend_handle!(ConstraintHandle);
