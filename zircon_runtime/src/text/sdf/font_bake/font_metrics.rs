use std::sync::Arc;

use ttf_parser::Face;

// The same parser serves shaping metadata, SDF metrics, and the fdsm outline
// adapter. Keep its borrowed face alive without a second font parsing stack.
self_cell::self_cell! {
    pub(super) struct SdfFontMetrics {
        owner: Arc<[u8]>,
        #[covariant]
        dependent: Face,
    }
}
