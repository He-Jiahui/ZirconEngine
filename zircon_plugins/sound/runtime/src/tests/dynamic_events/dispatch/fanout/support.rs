#[path = "../../../dispatch/fanout/support/fixture.rs"]
mod fixture;
#[path = "../../../dispatch/fanout/support/ids.rs"]
mod ids;
#[path = "../../../dispatch/fanout/support/invocation.rs"]
mod invocation;
#[path = "../../../dispatch/fanout/support/registration.rs"]
mod registration;

pub(super) use fixture::fanout_fixture;
