#[path = "../../spatial/attenuation.rs"]
mod attenuation;
#[path = "cone.rs"]
mod cone;
#[path = "doppler.rs"]
mod doppler;
#[path = "pan.rs"]
mod pan;
#[path = "profile.rs"]
mod profile;

pub(super) use pan::apply_source_pan;
pub(super) use profile::spatial_profile;
