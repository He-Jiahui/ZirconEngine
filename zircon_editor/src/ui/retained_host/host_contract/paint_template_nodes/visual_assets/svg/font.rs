use std::sync::{Arc, OnceLock};

use resvg::usvg;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cached_svg_font_db(
) -> Arc<usvg::fontdb::Database> {
    static SVG_FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    SVG_FONT_DB
        .get_or_init(|| {
            zircon_runtime::profile_scope!(
                "editor",
                "host_painter",
                "visual_assets_init_system_font_db"
            );
            let mut database = usvg::fontdb::Database::new();
            database.load_system_fonts();
            Arc::new(database)
        })
        .clone()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn svg_may_need_fonts(
    svg: &[u8],
) -> bool {
    let Ok(svg) = std::str::from_utf8(svg) else {
        return false;
    };
    let svg = svg.as_bytes();
    let mut index = 0;
    while index < svg.len() {
        let remaining = &svg[index..];
        match svg[index] {
            b'<' if remaining.starts_with(b"<text") || remaining.starts_with(b"<tspan") => {
                return true;
            }
            b'f' if remaining.starts_with(b"font-family") => return true,
            _ => index += 1,
        }
    }
    false
}

#[cfg(test)]
#[path = "font/single_scan_tests.rs"]
mod single_scan_tests;
