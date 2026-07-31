//! ECS-facing perception components, budgeted scanning, stimulus aging, and event adapters.

mod adapter;
mod components;
mod scan;
mod stimuli;

pub use adapter::{
    HearingStimulusAdapter, AI_HEARING_ANIMATION_EVENT_NAME, AI_HEARING_INGEST_EVENT_LIMIT,
    AI_HEARING_PENDING_EVENT_CAPACITY, AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS,
};
pub use components::{
    ai_perception_component_descriptors, AiPerceptionChannels, AiPerceptionReceiver,
    AiPerceptionSource, AI_PERCEPTION_RECEIVER_COMPONENT_TYPE, AI_PERCEPTION_SOURCE_COMPONENT_TYPE,
};
pub use scan::{AiTickBudget, PerceptionTickReport, DEFAULT_AI_PERCEPTION_PAIR_BUDGET};
pub use stimuli::PerceivedStimuli;

pub(crate) use adapter::{hearing_event_from_animation, hearing_event_from_sound};
pub(crate) use components::perception_receiver;
pub(crate) use scan::tick_perception;
