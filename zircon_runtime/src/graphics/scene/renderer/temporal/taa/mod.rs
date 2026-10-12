#[path = "execute_taa_resolve.rs"]
mod execute_taa_resolve;
#[path = "../../../temporal/taa/resolve_bind_group_cache.rs"]
pub(in crate::graphics::scene::scene_renderer) mod taa_resolve_bind_group_cache;
#[path = "taa_resolve_params.rs"]
pub(in crate::graphics::scene::scene_renderer) mod taa_resolve_params;
#[path = "../../../temporal/taa/temporal_history_store.rs"]
mod temporal_history_store;

pub(crate) use temporal_history_store::{
    TemporalHistoryKey, TemporalHistoryStore, TAA_SCENE_COLOR_HISTORY_FORMAT,
};
