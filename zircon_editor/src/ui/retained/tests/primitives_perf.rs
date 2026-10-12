use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::*;

#[test]
fn replacing_model_metadata_preserves_the_retained_value_storage() {
    let original = ModelRc::with_metadata(vec!["first", "second"], 1_u64);

    let replaced = original.replacing_metadata(2_u64);

    assert!(original.shares_values_with(&replaced));
    assert_eq!(replaced.metadata::<u64>(), Some(&2));
}

#[test]
fn raster_pixel_product_downscales_to_fit_without_upscaling() {
    let source = image::DynamicImage::ImageRgba8(image::RgbaImage::new(400, 200));
    let downscaled = Image::from_dynamic_image_for_target(source, 40, 40);
    assert_eq!((downscaled.width, downscaled.height), (40, 20));

    let small = image::DynamicImage::ImageRgba8(image::RgbaImage::new(20, 10));
    let retained = Image::from_dynamic_image_for_target(small, 40, 40);
    assert_eq!((retained.width, retained.height), (20, 10));
}

#[derive(Debug, PartialEq)]
struct FixtureMetadata {
    generation: u64,
}

#[test]
fn model_metadata_is_shared_by_clones_without_changing_value_equality() {
    let model = ModelRc::with_metadata(vec![1_u32, 2_u32], FixtureMetadata { generation: 7 });
    let cloned = model.clone();

    assert_eq!(model, cloned);
    assert_eq!(
        cloned
            .metadata::<FixtureMetadata>()
            .map(|metadata| metadata.generation),
        Some(7)
    );
}

#[test]
fn model_mapping_preserves_the_shared_metadata_allocation() {
    let model = ModelRc::with_metadata(vec![1_u32, 2_u32], FixtureMetadata { generation: 7 });
    let source_metadata = model
        .metadata_rc::<FixtureMetadata>()
        .expect("source metadata");

    let mapped = model.map_preserving_metadata(|value| value.to_string());
    let mapped_metadata = mapped
        .metadata_rc::<FixtureMetadata>()
        .expect("mapped metadata");

    assert_eq!(mapped.row_data(0).as_deref(), Some("1"));
    assert!(Rc::ptr_eq(&source_metadata, &mapped_metadata));
}

struct CloneProbe(Arc<AtomicUsize>);

impl Clone for CloneProbe {
    fn clone(&self) -> Self {
        self.0.fetch_add(1, Ordering::Relaxed);
        Self(Arc::clone(&self.0))
    }
}

#[derive(Clone, Debug)]
struct EqualityProbe {
    comparisons: Arc<AtomicUsize>,
    value: usize,
}

impl PartialEq for EqualityProbe {
    fn eq(&self, other: &Self) -> bool {
        self.comparisons.fetch_add(1, Ordering::Relaxed);
        self.value == other.value
    }
}

#[test]
fn model_equality_does_not_visit_rows_when_storage_identity_proves_equality() {
    let comparisons = Arc::new(AtomicUsize::new(0));
    let model = ModelRc::with_metadata(
        (0..128)
            .map(|value| EqualityProbe {
                comparisons: Arc::clone(&comparisons),
                value,
            })
            .collect(),
        "fixture",
    );
    let shared = model.clone();

    assert_eq!(model, shared);
    assert_eq!(comparisons.load(Ordering::Relaxed), 0);

    let independent = ModelRc::with_metadata(
        (0..128)
            .map(|value| EqualityProbe {
                comparisons: Arc::clone(&comparisons),
                value,
            })
            .collect(),
        "fixture",
    );
    assert_eq!(model, independent);
    assert_eq!(comparisons.load(Ordering::Relaxed), 128);
}

#[test]
fn model_rc_takes_unique_vec_model_without_cloning_rows() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let source = Rc::new(VecModel::from(vec![CloneProbe(Arc::clone(&clone_count))]));

    let model = ModelRc::from(source);

    assert_eq!(model.row_count(), 1);
    assert_eq!(clone_count.load(Ordering::Relaxed), 0);
}

#[test]
fn model_rc_clones_rows_when_the_source_vec_model_is_shared() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let source = Rc::new(VecModel::from(vec![CloneProbe(Arc::clone(&clone_count))]));
    let shared = Rc::clone(&source);

    let model = ModelRc::from(source);

    assert_eq!(model.row_count(), 1);
    assert_eq!(shared.values.len(), 1);
    assert_eq!(clone_count.load(Ordering::Relaxed), 1);
}

#[test]
fn model_rc_borrowed_row_access_does_not_clone() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let model = ModelRc {
        values: ModelValues::Contiguous(Rc::new(vec![CloneProbe(Arc::clone(&clone_count))])),
        metadata: None,
    };

    assert!(model.get(0).is_some());
    assert_eq!(clone_count.load(Ordering::Relaxed), 0);
}

#[test]
fn model_rc_publishes_shared_rows_without_cloning_values() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    let rows = Rc::new(vec![Rc::new(CloneProbe(Arc::clone(&clone_count)))]);

    let first = ModelRc::from_shared_rows(Rc::clone(&rows));
    let second = ModelRc::from_shared_rows(rows);

    assert!(first.shares_row_with(&second, 0));
    assert_eq!(clone_count.load(Ordering::Relaxed), 0);
}

#[test]
fn model_rc_row_patch_reuses_unmodified_contiguous_storage() {
    let original = ModelRc::with_metadata(vec!["left", "middle", "right"], "fixture");
    let patched = original.with_row_patches(BTreeMap::from([(1, "changed")]));

    assert_eq!(
        patched.iter().copied().collect::<Vec<_>>(),
        vec!["left", "changed", "right"]
    );
    assert_eq!(patched.metadata::<&str>(), Some(&"fixture"));
    match &patched.values {
        ModelValues::ContiguousOverlay { base, patches } => {
            assert!(
                matches!(&original.values, ModelValues::Contiguous(original_base) if Rc::ptr_eq(base, original_base))
            );
            assert_eq!(patches.len(), 1);
        }
        _ => panic!("contiguous model should publish a sparse overlay"),
    }
}

#[test]
fn sparse_overlay_iterator_preserves_double_ended_interleaving() {
    let original = ModelRc::with_metadata((0usize..16).collect(), "fixture");
    let patched = original.with_row_patches(BTreeMap::from([(1, 101), (7, 107), (14, 114)]));
    let mut iter = patched.iter();

    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next_back(), Some(&15));
    assert_eq!(iter.next(), Some(&101));
    assert_eq!(iter.next_back(), Some(&114));
    assert_eq!(
        iter.collect::<Vec<_>>(),
        vec![&2, &3, &4, &5, &6, &107, &8, &9, &10, &11, &12, &13]
    );
}

#[test]
fn sparse_overlay_iterator_exhausts_when_both_ends_meet_on_one_patch() {
    let patched = ModelRc::with_metadata(vec![0usize], "fixture")
        .with_row_patches(BTreeMap::from([(0, 100)]));
    let mut forward_first = patched.iter();
    assert_eq!(forward_first.next(), Some(&100));
    assert_eq!(forward_first.next_back(), None);

    let mut reverse_first = patched.iter();
    assert_eq!(reverse_first.next_back(), Some(&100));
    assert_eq!(reverse_first.next(), None);
}

#[test]
fn image_clone_shares_the_pixel_allocation() {
    let image = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &[1, 2, 3, 255],
        1,
        1,
    ));
    let cloned = image.clone();

    assert!(image.shares_pixels_with(&cloned));
    assert_eq!(image, cloned);
}
