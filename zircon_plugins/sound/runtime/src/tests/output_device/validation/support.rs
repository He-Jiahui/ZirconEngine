#[path = "../../validation/support/block_size.rs"]
mod block_size;
#[path = "../../validation/support/channel_layout.rs"]
mod channel_layout;
#[path = "../../validation/support/descriptor.rs"]
mod descriptor;
#[path = "../../validation/support/speaker_metadata.rs"]
mod speaker_metadata;

pub(super) use block_size::invalid_block_size_descriptor;
pub(super) use channel_layout::invalid_channel_layout_descriptor;
pub(super) use speaker_metadata::invalid_speaker_metadata_descriptor;
