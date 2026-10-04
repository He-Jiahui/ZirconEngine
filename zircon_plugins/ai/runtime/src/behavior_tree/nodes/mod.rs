//! 内置节点表只声明 ID、类别与语义；目录负责将其注册为可编译的实现槽位。

mod composite;
mod decorator;
mod integration;
mod service;
mod task;

#[cfg(test)]
pub(crate) use integration::IntegrationTaskResult;
pub(crate) use integration::{
    BehaviorIntegrationHost, BehaviorIntegrationTaskContext, RuntimeBehaviorIntegrationHost,
};

use super::{BehaviorNodeCategory, BehaviorNodeSemantics};

pub(super) type StandardNodeDescriptor = (
    &'static str,
    &'static str,
    BehaviorNodeCategory,
    BehaviorNodeSemantics,
);

pub(super) fn standard_node_descriptors() -> impl Iterator<Item = &'static StandardNodeDescriptor> {
    composite::DESCRIPTORS
        .iter()
        .chain(decorator::DESCRIPTORS.iter())
        .chain(service::DESCRIPTORS.iter())
        .chain(task::DESCRIPTORS.iter())
}
