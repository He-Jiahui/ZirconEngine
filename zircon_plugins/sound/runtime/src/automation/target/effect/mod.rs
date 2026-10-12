#[path = "apply.rs"]
mod apply;
#[path = "base_parameters.rs"]
mod base_parameters;
#[path = "delay.rs"]
mod delay;
#[path = "dynamics.rs"]
mod dynamics;
#[path = "filter.rs"]
mod filter;
#[path = "gain.rs"]
mod gain;
#[path = "modulation.rs"]
mod modulation;
#[path = "reverb.rs"]
mod reverb;
#[path = "shaper.rs"]
mod shaper;
#[path = "stereo.rs"]
mod stereo;

pub(in crate::automation::target) use apply::apply_effect_parameter;
