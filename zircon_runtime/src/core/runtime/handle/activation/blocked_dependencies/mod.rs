mod five_service;
mod four_service;
mod single;
mod three_service;
mod two_service;

pub(super) use five_service::{FiveServiceDependencyMatch, first_blocked_five_service_dependency};
pub(super) use four_service::{FourServiceDependencyMatch, first_blocked_four_service_dependency};
pub(super) use single::dependency_slice_contains_service;
pub(super) use three_service::{
    ThreeServiceDependencyMatch, first_blocked_three_service_dependency,
};
pub(super) use two_service::{TwoServiceDependencyMatch, first_blocked_two_service_dependency};
