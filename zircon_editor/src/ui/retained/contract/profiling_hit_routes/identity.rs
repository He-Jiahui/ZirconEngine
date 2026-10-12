pub(super) fn profile_control_id<'a>(id: &'a str, kind: &str, surface: &str) -> Option<&'a str> {
    id.strip_prefix(kind)?
        .strip_prefix('.')?
        .strip_prefix(surface)?
        .strip_prefix('.')
}
