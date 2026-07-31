pub(in crate::graphics::runtime::render_framework) const ZR_RENDERDOC_CAPTURE_NEXT_ENV: &str =
    "ZR_RENDERDOC_CAPTURE_NEXT";
pub(in crate::graphics::runtime::render_framework) const ZR_RENDERDOC_CAPTURE_FRAME_COUNT_ENV:
    &str = "ZR_RENDERDOC_CAPTURE_FRAME_COUNT";

const MAX_RENDERDOC_CAPTURE_FRAME_COUNT: u32 = 8;

pub(in crate::graphics::runtime::render_framework) fn renderdoc_capture_frame_count_from_env() -> u32
{
    renderdoc_capture_frame_count_from_values(
        std::env::var(ZR_RENDERDOC_CAPTURE_FRAME_COUNT_ENV)
            .ok()
            .as_deref(),
        std::env::var(ZR_RENDERDOC_CAPTURE_NEXT_ENV).ok().as_deref(),
    )
}

fn renderdoc_capture_frame_count_from_values(
    frame_count: Option<&str>,
    capture_next: Option<&str>,
) -> u32 {
    frame_count
        .and_then(|value| value.parse::<u32>().ok())
        .map(|count| count.min(MAX_RENDERDOC_CAPTURE_FRAME_COUNT))
        .unwrap_or_else(|| u32::from(renderdoc_capture_next_from_value(capture_next)))
}

pub(crate) fn renderdoc_capture_next_from_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| value == "1")
}

#[cfg(test)]
mod tests {
    use super::{renderdoc_capture_frame_count_from_values, MAX_RENDERDOC_CAPTURE_FRAME_COUNT};

    #[test]
    fn renderdoc_capture_frame_count_preserves_the_legacy_next_frame_switch() {
        assert_eq!(
            renderdoc_capture_frame_count_from_values(None, Some("1")),
            1
        );
        assert_eq!(renderdoc_capture_frame_count_from_values(None, None), 0);
    }

    #[test]
    fn renderdoc_capture_frame_count_is_explicit_and_bounded() {
        assert_eq!(
            renderdoc_capture_frame_count_from_values(Some("2"), Some("1")),
            2
        );
        assert_eq!(
            renderdoc_capture_frame_count_from_values(Some("0"), Some("1")),
            0
        );
        assert_eq!(
            renderdoc_capture_frame_count_from_values(Some("999"), None),
            MAX_RENDERDOC_CAPTURE_FRAME_COUNT
        );
        assert_eq!(
            renderdoc_capture_frame_count_from_values(Some("invalid"), Some("1")),
            1
        );
    }
}
