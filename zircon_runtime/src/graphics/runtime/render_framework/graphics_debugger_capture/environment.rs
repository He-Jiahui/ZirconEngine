pub(in crate::graphics::runtime::render_framework) const ZR_RENDERDOC_CAPTURE_NEXT_ENV: &str =
    "ZR_RENDERDOC_CAPTURE_NEXT";

pub(in crate::graphics::runtime::render_framework) fn renderdoc_capture_next_from_env() -> bool {
    std::env::var(ZR_RENDERDOC_CAPTURE_NEXT_ENV).is_ok_and(|value| value == "1")
}

#[cfg(test)]
pub(crate) fn renderdoc_capture_next_from_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| value == "1")
}
