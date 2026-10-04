use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// UI会话呈现模式；是否有world或活跃Play实例须从对应生命周期状态验证。
pub enum EditorSessionMode {
    #[default]
    Welcome,
    Project,
    Playing,
}
