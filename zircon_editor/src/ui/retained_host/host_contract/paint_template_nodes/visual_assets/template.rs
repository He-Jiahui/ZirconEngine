use super::asset::{icon_source_is_vector, source_is_svg, vector_cache_target};
use super::candidates::{icon_candidates, image_candidates};
use super::keys::template_image_cache_key;
use super::loading::{
    load_pixels_from_candidates_with_status, missing_icon_pixels, CandidatePixelsLoad,
};
use super::pixels::HostPaintImagePixels;
use super::retained::retained_image_pixels;
use super::target::RasterTargetSize;
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_image_pixels(
    preview_image: &crate::ui::retained_host::primitives::Image,
    media_source: &str,
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    prefer_preview_image: bool,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    template_image_pixels_with_vector_hint(
        preview_image,
        media_source,
        icon_name,
        target_width,
        target_height,
        tint,
        prefer_preview_image,
        damage_frame,
        false,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_vector_image_pixels(
    preview_image: &crate::ui::retained_host::primitives::Image,
    media_source: &str,
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    prefer_preview_image: bool,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    template_image_pixels_with_vector_hint(
        preview_image,
        media_source,
        icon_name,
        target_width,
        target_height,
        tint,
        prefer_preview_image,
        damage_frame,
        true,
    )
}

fn template_image_pixels_with_vector_hint(
    preview_image: &crate::ui::retained_host::primitives::Image,
    media_source: &str,
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    prefer_preview_image: bool,
    damage_frame: Option<FrameRect>,
    explicit_vector: bool,
) -> Option<HostPaintImagePixels> {
    let requested_target = RasterTargetSize::new(target_width, target_height);
    let source_pixels = || {
        load_template_candidate_pixels(
            media_source,
            icon_name,
            requested_target,
            tint,
            damage_frame,
            explicit_vector,
        )
    };
    let preview_pixels = || retained_image_pixels(preview_image, tint);
    let pixels = if prefer_preview_image {
        preview_pixels().or_else(source_pixels)
    } else {
        source_pixels().or_else(preview_pixels)
    };
    let fallback_target = vector_cache_target(
        requested_target,
        template_source_is_vector("", icon_name, explicit_vector),
    );
    let fallback_key = template_image_cache_key("", icon_name);
    pixels.or_else(|| {
        (!icon_name.trim().is_empty())
            .then_some(())
            .and_then(|_| fallback_target)
            .and_then(|target| missing_icon_pixels(&fallback_key, target, tint))
    })
}

fn load_template_candidate_pixels(
    media_source: &str,
    icon_name: &str,
    requested_target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
    damage_frame: Option<FrameRect>,
    explicit_vector: bool,
) -> Option<HostPaintImagePixels> {
    if !media_source.trim().is_empty() {
        let target = vector_cache_target(
            requested_target,
            template_source_is_vector(media_source, "", explicit_vector),
        );
        let key = template_image_cache_key(media_source, "");
        match load_pixels_from_candidates_with_status(
            || image_candidates(media_source),
            &key,
            target,
            tint,
            damage_frame.clone(),
        ) {
            CandidatePixelsLoad::Ready(pixels) => return Some(pixels),
            CandidatePixelsLoad::Deferred => return None,
            CandidatePixelsLoad::Missing => {}
        }
    }
    if icon_name.trim().is_empty() {
        return None;
    }
    let target = vector_cache_target(
        requested_target,
        template_source_is_vector("", icon_name, explicit_vector),
    );
    let key = template_image_cache_key("", icon_name);
    match load_pixels_from_candidates_with_status(
        || icon_candidates(icon_name),
        &key,
        target,
        tint,
        damage_frame,
    ) {
        CandidatePixelsLoad::Ready(pixels) => Some(pixels),
        CandidatePixelsLoad::Missing | CandidatePixelsLoad::Deferred => None,
    }
}

fn template_source_is_vector(media_source: &str, icon_name: &str, explicit_vector: bool) -> bool {
    if explicit_vector {
        return true;
    }
    if !media_source.trim().is_empty() {
        return source_is_svg(media_source);
    }
    icon_source_is_vector(icon_name)
}

#[cfg(test)]
mod tests {
    use super::template_source_is_vector;

    #[test]
    fn primary_media_source_wins_over_the_icon_fallback_kind() {
        assert!(!template_source_is_vector(
            "asset://previews/material.png",
            "folder-open-outline",
            false
        ));
        assert!(template_source_is_vector(
            "asset://previews/material.svg",
            "fallback.png",
            false
        ));
    }

    #[test]
    fn svg_role_hint_and_semantic_icon_names_preserve_vector_identity() {
        assert!(template_source_is_vector("", "save", false));
        assert!(template_source_is_vector("preview.png", "", true));
        assert!(!template_source_is_vector("", "preview.png", false));
    }
}
