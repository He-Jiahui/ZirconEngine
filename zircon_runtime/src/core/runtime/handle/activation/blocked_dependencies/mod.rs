//! 卸载预检查使用定长匹配器寻找首个仍被其他实例依赖的槽位。
//! 结果按卸载列表的顺序选择，避免依赖遍历顺序改变阻塞诊断。

mod five_service;
mod four_service;
mod single;
mod three_service;
mod two_service;

pub(super) use five_service::{first_blocked_five_service_dependency, FiveServiceDependencyMatch};
pub(super) use four_service::{first_blocked_four_service_dependency, FourServiceDependencyMatch};
pub(super) use single::dependency_slice_contains_service;
pub(super) use three_service::{
    first_blocked_three_service_dependency, ThreeServiceDependencyMatch,
};
pub(super) use two_service::{first_blocked_two_service_dependency, TwoServiceDependencyMatch};
