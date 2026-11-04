use std::sync::Arc;
use std::time::Duration;

use resvg::{tiny_skia, usvg};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::{content::startup_text, EditorStartupStage};

/// The native overlay anchors one bounded card. Inside it, text and stage rows
/// flow from shared typography/gap tokens; resize/DPI uniformly fit that flow.
pub(super) fn render(
    width: u32,
    height: u32,
    scale: f32,
    stage: EditorStartupStage,
    project_requested: bool,
    elapsed: Duration,
    fonts: Arc<usvg::fontdb::Database>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let tokens = EditorDesignTokens::default();
    let palette = &tokens.palette;
    let metrics = super::super::host_contract::current_host_metrics();
    let gap = metrics.gap_l;
    let line = metrics.line_height(metrics.font_body);
    let row = line + metrics.gap_m * 2.0;
    let natural_height = gap * 6.0 + metrics.font_large * 2.0 + line * 3.0 + row * 4.0;
    let logical_width = (width as f32 / scale).max(1.0);
    let logical_height = (height as f32 / scale).max(1.0);
    let fit = (logical_height / natural_height).min(1.0);
    let offset_y = (logical_height - natural_height * fit).max(0.0) * 0.5;
    let card_width = logical_width / fit;
    let content_width = (card_width - gap * 4.0).max(1.0);
    let background = color(palette.surface[0].to_u8());
    let surface = color(palette.surface[1].to_u8());
    let accent = color(palette.accent.to_u8());
    let text = color(palette.text_primary.to_u8());
    let muted = color(palette.text_secondary.to_u8());
    let border = color(palette.border.to_u8());
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {logical_width} {logical_height}"><rect width="100%" height="100%" fill="{background}"/><g transform="translate(0 {offset_y}) scale({fit})" font-family="Fira Sans" fill="{text}"><rect x="{gap}" y="{gap}" width="{}" height="{}" rx="{}" fill="{surface}" stroke="{border}"/>"#,
        card_width - gap * 2.0,
        natural_height - gap * 2.0,
        metrics.radius_panel
    );
    let x = gap * 2.0;
    let mut y = gap * 2.0 + metrics.font_large;
    label(
        &mut svg,
        x,
        y,
        metrics.font_large,
        &text,
        &startup_text("editor.startup.brand"),
    );
    y += line + metrics.gap_s;
    label(
        &mut svg,
        x,
        y,
        metrics.font_small,
        &muted,
        &format!(
            "{} {}",
            startup_text("editor.startup.product"),
            env!("CARGO_PKG_VERSION")
        ),
    );
    y += gap + line;
    label(
        &mut svg,
        x,
        y,
        metrics.font_body,
        &text,
        &startup_text("editor.startup.title"),
    );
    y += gap;
    let marker = metrics.gap_m;
    for item in EditorStartupStage::ALL {
        let active = item == stage;
        let tone = if item <= stage { &accent } else { &muted };
        let center_y = y + row * 0.5;
        if active {
            svg.push_str(&format!(r#"<rect x="{x}" y="{y}" width="{content_width}" height="{row}" rx="{}" fill="{}"/>"#,
                metrics.radius_control, color(palette.surface[3].to_u8())));
        }
        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{center_y}" r="{}" fill="{tone}"/>"#,
            x + marker * 1.5,
            marker * 0.4
        ));
        label(
            &mut svg,
            x + marker * 3.0,
            center_y + metrics.font_body * 0.35,
            metrics.font_body,
            if active { &text } else { &muted },
            &startup_text(item.text_key(project_requested)),
        );
        y += row;
    }
    y += gap;
    let footer = format!(
        "{}  {:.0} s",
        startup_text("editor.startup.elapsed"),
        elapsed.as_secs_f64().floor()
    );
    label(&mut svg, x, y, metrics.font_small, &muted, &footer);
    y += line;
    label(
        &mut svg,
        x,
        y,
        metrics.font_small,
        &muted,
        &startup_text("editor.startup.dismiss"),
    );
    svg.push_str("</g></svg>");
    let options = usvg::Options {
        fontdb: fonts,
        ..Default::default()
    };
    let tree = usvg::Tree::from_data(svg.as_bytes(), &options)?;
    let mut pixels =
        tiny_skia::Pixmap::new(width, height).ok_or("invalid splash surface extent")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixels.as_mut(),
    );
    Ok(pixels.take())
}

fn color(rgba: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
}

fn label(svg: &mut String, x: f32, y: f32, size: f32, color: &str, value: &str) {
    let escaped = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    svg.push_str(&format!(
        r#"<text x="{x}" y="{y}" font-size="{size}" fill="{color}">{escaped}</text>"#
    ));
}

#[cfg(test)]
#[path = "tests/paint.rs"]
mod tests;
