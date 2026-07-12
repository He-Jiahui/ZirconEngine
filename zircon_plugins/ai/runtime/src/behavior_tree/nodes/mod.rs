mod composite;
mod decorator;
mod service;
mod task;

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
