mod burst;
mod codec;
mod curve;
mod lower;
mod model;
mod overview;
mod parameter;
mod seed;
mod validation;

pub use burst::{parse_particle_burst_count, ParticleSourceBurstEdit, ParticleSourceBurstValue};
pub use codec::{ParticleSourceError, ParticleSourceRead};
pub use curve::{
    ParticleCurveChannel, ParticleSourceCurveAdapter, ParticleSourceCurveEdit,
    ParticleSourceCurveId,
};
pub use model::{
    ParticleSourceAnimationBinding, ParticleSourceBackend, ParticleSourceBurst,
    ParticleSourceColorKey, ParticleSourceCoordinateSpace, ParticleSourceEmitter,
    ParticleSourcePhysics, ParticleSourceScalarKey, ParticleSourceScalarRange, ParticleSourceShape,
    ParticleSourceSystem, ParticleSourceVec3Range, ParticleSystemSource,
};
pub use overview::{
    ParticleOverviewConnectionEligibility, ParticleOverviewNodeId, ParticleOverviewProjection,
};
pub use parameter::ParticleSourceParameterEdit;
pub use seed::parse_particle_seed;
