/// 宿主观察到的应用激活状态，不代表生命周期已进入运行态或存在可用表面。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationActivationState {
    #[default]
    Unknown,
    Active,
    Inactive,
}
