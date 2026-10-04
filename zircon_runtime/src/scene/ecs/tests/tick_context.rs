use super::*;
use crate::core::framework::time::ClockDomainStamp;

fn context(domain: ClockDomainId) -> SystemTickContext {
    SystemTickContext::new(
        SystemStage::Update,
        ClockDomainStamp::initial(domain),
        7,
        None,
        Duration::from_millis(16),
        Duration::from_millis(80),
        3,
    )
}

#[test]
fn stage_contexts_select_the_declared_clock_without_reconstructing_it() {
    let contexts = SceneStageTickContexts::new(
        context(ClockDomainId::WorldVirtual),
        context(ClockDomainId::MonotonicReal),
        context(ClockDomainId::WorldFixed),
    );

    assert_eq!(
        contexts
            .for_domain(SceneSystemClockDomain::Virtual)
            .clock_domain(),
        ClockDomainId::WorldVirtual
    );
    assert_eq!(
        contexts
            .for_domain(SceneSystemClockDomain::MonotonicReal)
            .clock_domain(),
        ClockDomainId::MonotonicReal
    );
    assert_eq!(
        contexts
            .for_domain(SceneSystemClockDomain::Fixed)
            .clock_domain(),
        ClockDomainId::WorldFixed
    );
}
