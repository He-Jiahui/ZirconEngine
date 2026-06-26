pub(crate) const DOCUMENT_TAB_STRIP_X: f32 = 8.0;
pub(crate) const DOCUMENT_TAB_STRIP_Y: f32 = 1.0;
pub(crate) const DOCUMENT_TAB_MIN_WIDTH: f32 = 124.0;
pub(crate) const DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH: f32 = 156.0;
pub(crate) const DOCUMENT_TAB_MAX_WIDTH: f32 = 220.0;
pub(crate) const DOCUMENT_TAB_HEIGHT: f32 = 30.0;
pub(crate) const DOCUMENT_TAB_GAP: f32 = 4.0;
pub(crate) const DOCUMENT_TAB_CLOSE_EXTENT: f32 = 20.0;
pub(crate) const DOCUMENT_TAB_CLOSE_RIGHT_INSET: f32 = 8.0;
pub(crate) const DOCUMENT_TAB_CLOSE_TOP_INSET: f32 = 6.0;

const TITLE_WIDTH_PER_CHAR: f32 = 7.0;
const TITLE_CHROME_RESERVE: f32 = 42.0;
const CLOSEABLE_TITLE_CHROME_RESERVE: f32 = 70.0;

pub(crate) fn document_tab_preferred_width(title: &str, closeable: bool) -> f32 {
    let reserve = if closeable {
        CLOSEABLE_TITLE_CHROME_RESERVE
    } else {
        TITLE_CHROME_RESERVE
    };
    let minimum = if closeable {
        DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH
    } else {
        DOCUMENT_TAB_MIN_WIDTH
    };

    (title.chars().count() as f32 * TITLE_WIDTH_PER_CHAR + reserve)
        .clamp(minimum, DOCUMENT_TAB_MAX_WIDTH)
}

pub(crate) fn document_tab_close_x(tab_x: f32, tab_width: f32) -> f32 {
    tab_x + tab_width - DOCUMENT_TAB_CLOSE_RIGHT_INSET - DOCUMENT_TAB_CLOSE_EXTENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closeable_document_tab_width_keeps_asset_browser_title_readable() {
        let width = document_tab_preferred_width("Asset Browser", true);

        assert!(width >= 160.0);
        assert!(width <= DOCUMENT_TAB_MAX_WIDTH);
    }

    #[test]
    fn close_button_frame_uses_shared_right_inset_and_extent() {
        let tab_x = DOCUMENT_TAB_STRIP_X;
        let close_x = document_tab_close_x(tab_x, DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);

        assert_eq!(
            close_x + DOCUMENT_TAB_CLOSE_EXTENT + DOCUMENT_TAB_CLOSE_RIGHT_INSET,
            tab_x + DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH
        );
    }
}
