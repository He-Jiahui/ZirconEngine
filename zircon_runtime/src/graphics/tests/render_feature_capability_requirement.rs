use super::RenderFeatureCapabilityRequirement;
use crate::core::framework::render::RenderCapabilityKind;

#[test]
fn feature_requirements_round_trip_every_render_capability() {
    for capability in RenderCapabilityKind::ALL {
        let requirement = RenderFeatureCapabilityRequirement::from_capability_kind(capability);

        assert_eq!(requirement.capability_kind(), capability);
    }
}
