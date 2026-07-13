use std::path::Path;

use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) struct FramebufferProof<'a> {
    pub(super) rgba: &'a [u8],
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) background: [u8; 4],
}

impl FramebufferProof<'_> {
    pub(super) fn changed_pixels(&self, frame: UiFrame, threshold: u8) -> usize {
        self.pixels(frame)
            .filter(|pixel| max_channel_delta(pixel, &self.background) >= threshold)
            .count()
    }

    pub(super) fn soft_edge_pixels(&self) -> usize {
        self.soft_edge_pixels_in(UiFrame::new(
            0.0,
            0.0,
            self.width as f32,
            self.height as f32,
        ))
    }

    pub(super) fn soft_edge_pixels_in(&self, frame: UiFrame) -> usize {
        self.pixels(frame)
            .filter(|pixel| {
                let delta = max_channel_delta(pixel, &self.background);
                (8..=210).contains(&delta)
            })
            .count()
    }

    pub(super) fn changed_pixel_principal_axis_degrees(
        &self,
        frame: UiFrame,
        threshold: u8,
    ) -> f32 {
        let points = self
            .pixel_coordinates(frame)
            .filter(|(_, _, pixel)| max_channel_delta(pixel, &self.background) >= threshold)
            .map(|(x, y, _)| (x as f64, y as f64))
            .collect::<Vec<_>>();
        assert!(
            points.len() > 2,
            "principal-axis proof requires visible pixels"
        );
        let count = points.len() as f64;
        let mean_x = points.iter().map(|point| point.0).sum::<f64>() / count;
        let mean_y = points.iter().map(|point| point.1).sum::<f64>() / count;
        let covariance_xx = points
            .iter()
            .map(|point| (point.0 - mean_x).powi(2))
            .sum::<f64>();
        let covariance_yy = points
            .iter()
            .map(|point| (point.1 - mean_y).powi(2))
            .sum::<f64>();
        let covariance_xy = points
            .iter()
            .map(|point| (point.0 - mean_x) * (point.1 - mean_y))
            .sum::<f64>();
        (0.5 * (2.0 * covariance_xy).atan2(covariance_xx - covariance_yy))
            .to_degrees()
            .abs() as f32
    }

    pub(super) fn dominant_color_pixels(
        &self,
        frame: UiFrame,
        predicate: impl Fn(&[u8]) -> bool,
    ) -> usize {
        self.pixels(frame).filter(|pixel| predicate(pixel)).count()
    }

    fn pixels(&self, frame: UiFrame) -> impl Iterator<Item = &[u8]> {
        self.pixel_coordinates(frame).map(|(_, _, pixel)| pixel)
    }

    fn pixel_coordinates(&self, frame: UiFrame) -> impl Iterator<Item = (u32, u32, &[u8])> {
        let left = frame.x.max(0.0).floor().min(self.width as f32) as u32;
        let top = frame.y.max(0.0).floor().min(self.height as f32) as u32;
        let right = frame.right().max(0.0).ceil().min(self.width as f32) as u32;
        let bottom = frame.bottom().max(0.0).ceil().min(self.height as f32) as u32;
        (top..bottom).flat_map(move |y| {
            (left..right).map(move |x| {
                let offset = ((y * self.width + x) * 4) as usize;
                (x, y, &self.rgba[offset..offset + 4])
            })
        })
    }
}

pub(super) fn assert_no_named_file_under(root: &Path, file_name: &str) {
    if !root.is_dir() {
        return;
    }
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().is_some_and(|name| name == file_name) {
                panic!(
                    "runtime text framebuffer proof must not be copied under {}: {}",
                    root.display(),
                    path.display()
                );
            }
        }
    }
}

fn max_channel_delta(lhs: &[u8], rhs: &[u8; 4]) -> u8 {
    lhs.iter()
        .zip(rhs)
        .map(|(lhs, rhs)| lhs.abs_diff(*rhs))
        .max()
        .unwrap_or(0)
}
