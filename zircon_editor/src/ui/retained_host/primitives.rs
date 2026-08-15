use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

#[path = "persistent_row_patch_map.rs"]
mod persistent_row_patch_map;

use persistent_row_patch_map::PersistentRowPatchMap;

pub(crate) type SharedString = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhysicalPosition {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPosition {
    pub(crate) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseRequestResponse {
    HideWindow,
    KeepWindowShown,
}

#[derive(Debug)]
pub(crate) enum PlatformError {
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for PlatformError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub(crate) const fn from_argb_u8(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    pub(crate) const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self { a: 255, r, g, b }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rgba8Pixel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SharedPixelBuffer<P> {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    _pixel: std::marker::PhantomData<P>,
}

impl<P> SharedPixelBuffer<P> {
    pub(crate) fn clone_from_slice(rgba: &[u8], width: u32, height: u32) -> Self {
        Self {
            rgba: rgba.to_vec(),
            width,
            height,
            _pixel: std::marker::PhantomData,
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.rgba
    }

    pub(crate) const fn width(&self) -> u32 {
        self.width
    }

    pub(crate) const fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Image {
    rgba: Arc<[u8]>,
    width: u32,
    height: u32,
}

impl Image {
    pub(crate) fn from_rgba8(buffer: SharedPixelBuffer<Rgba8Pixel>) -> Self {
        Self {
            rgba: buffer.rgba.into(),
            width: buffer.width,
            height: buffer.height,
        }
    }

    pub(crate) fn load_from_path(path: &Path) -> Result<Self, image::ImageError> {
        let image = image::open(path)?.to_rgba8();
        let (width, height) = image.dimensions();
        Ok(Self {
            rgba: image.into_raw().into(),
            width,
            height,
        })
    }

    pub(crate) fn to_rgba8(&self) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        self.is_valid()
            .then(|| SharedPixelBuffer::clone_from_slice(&self.rgba, self.width, self.height))
    }

    pub(crate) const fn size(&self) -> PhysicalSize {
        PhysicalSize::new(self.width, self.height)
    }

    fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }

    #[cfg(test)]
    pub(crate) fn shares_pixels_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rgba, &other.rgba)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VecModel<T> {
    values: Vec<T>,
}

impl<T> From<Vec<T>> for VecModel<T> {
    fn from(values: Vec<T>) -> Self {
        Self { values }
    }
}

#[derive(Clone)]
pub(crate) struct ModelRc<T> {
    values: ModelValues<T>,
    metadata: Option<Rc<dyn Any>>,
}

#[derive(Clone)]
enum ModelValues<T> {
    Contiguous(Rc<Vec<T>>),
    ContiguousOverlay {
        base: Rc<Vec<T>>,
        patches: PersistentRowPatchMap<T>,
    },
    SharedRows(Rc<Vec<Rc<T>>>),
    SharedRowsOverlay {
        base: Rc<Vec<Rc<T>>>,
        patches: PersistentRowPatchMap<T>,
    },
}

pub(crate) enum ModelIter<'a, T> {
    Contiguous(std::slice::Iter<'a, T>),
    ContiguousOverlay {
        base: &'a [T],
        patches: &'a PersistentRowPatchMap<T>,
        front: usize,
        back: usize,
    },
    SharedRows(std::slice::Iter<'a, Rc<T>>),
    SharedRowsOverlay {
        base: &'a [Rc<T>],
        patches: &'a PersistentRowPatchMap<T>,
        front: usize,
        back: usize,
    },
}

impl<'a, T> Iterator for ModelIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Contiguous(values) => values.next(),
            Self::ContiguousOverlay {
                base,
                patches,
                front,
                back,
            } => {
                if *front >= *back {
                    return None;
                }
                let row = *front;
                *front += 1;
                Some(patches.get(row).map(Rc::as_ref).unwrap_or(&base[row]))
            }
            Self::SharedRows(values) => values.next().map(Rc::as_ref),
            Self::SharedRowsOverlay {
                base,
                patches,
                front,
                back,
            } => {
                if *front >= *back {
                    return None;
                }
                let row = *front;
                *front += 1;
                Some(
                    patches
                        .get(row)
                        .or_else(|| base.get(row))
                        .expect("overlay row must resolve")
                        .as_ref(),
                )
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Contiguous(values) => values.size_hint(),
            Self::ContiguousOverlay { front, back, .. }
            | Self::SharedRowsOverlay { front, back, .. } => {
                let remaining = back.saturating_sub(*front);
                (remaining, Some(remaining))
            }
            Self::SharedRows(values) => values.size_hint(),
        }
    }
}

impl<T> DoubleEndedIterator for ModelIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Contiguous(values) => values.next_back(),
            Self::ContiguousOverlay {
                base,
                patches,
                front,
                back,
            } => {
                if *front >= *back {
                    return None;
                }
                *back -= 1;
                Some(patches.get(*back).map(Rc::as_ref).unwrap_or(&base[*back]))
            }
            Self::SharedRows(values) => values.next_back().map(Rc::as_ref),
            Self::SharedRowsOverlay {
                base,
                patches,
                front,
                back,
            } => {
                if *front >= *back {
                    return None;
                }
                *back -= 1;
                Some(
                    patches
                        .get(*back)
                        .or_else(|| base.get(*back))
                        .expect("overlay row must resolve")
                        .as_ref(),
                )
            }
        }
    }
}

impl<T> ExactSizeIterator for ModelIter<'_, T> {}

impl<T> Default for ModelRc<T> {
    fn default() -> Self {
        Self {
            values: ModelValues::Contiguous(Rc::new(Vec::new())),
            metadata: None,
        }
    }
}

impl<T> ModelRc<T> {
    pub(crate) fn with_metadata<M: Any>(values: Vec<T>, metadata: M) -> Self {
        Self {
            values: ModelValues::Contiguous(Rc::new(values)),
            metadata: Some(Rc::new(metadata)),
        }
    }

    pub(crate) fn from_shared_rows(values: Rc<Vec<Rc<T>>>) -> Self {
        Self {
            values: ModelValues::SharedRows(values),
            metadata: None,
        }
    }

    pub(crate) fn from_shared_rows_overlay_with_metadata(
        base: Rc<Vec<Rc<T>>>,
        patches: Rc<BTreeMap<usize, Rc<T>>>,
        metadata: Rc<dyn Any>,
    ) -> Self {
        let patches = PersistentRowPatchMap::from_shared_rows(base.len(), patches.as_ref());
        Self {
            values: ModelValues::SharedRowsOverlay { base, patches },
            metadata: Some(metadata),
        }
    }

    pub(crate) fn from_shared_rows_overlay(
        base: Rc<Vec<Rc<T>>>,
        patches: Rc<BTreeMap<usize, Rc<T>>>,
    ) -> Self {
        let patches = PersistentRowPatchMap::from_shared_rows(base.len(), patches.as_ref());
        Self {
            values: ModelValues::SharedRowsOverlay { base, patches },
            metadata: None,
        }
    }

    pub(crate) fn row_count(&self) -> usize {
        match &self.values {
            ModelValues::Contiguous(values) => values.len(),
            ModelValues::ContiguousOverlay { base, .. } => base.len(),
            ModelValues::SharedRows(values) => values.len(),
            ModelValues::SharedRowsOverlay { base, .. } => base.len(),
        }
    }

    pub(crate) fn row_data(&self, row: usize) -> Option<T>
    where
        T: Clone,
    {
        self.get(row).cloned()
    }

    pub(crate) fn get(&self, row: usize) -> Option<&T> {
        match &self.values {
            ModelValues::Contiguous(values) => values.get(row),
            ModelValues::ContiguousOverlay { base, patches } => {
                patches.get(row).map(Rc::as_ref).or_else(|| base.get(row))
            }
            ModelValues::SharedRows(values) => values.get(row).map(Rc::as_ref),
            ModelValues::SharedRowsOverlay { base, patches } => {
                patches.get(row).or_else(|| base.get(row)).map(Rc::as_ref)
            }
        }
    }

    pub(crate) fn iter(&self) -> ModelIter<'_, T> {
        match &self.values {
            ModelValues::Contiguous(values) => ModelIter::Contiguous(values.iter()),
            ModelValues::ContiguousOverlay { base, patches } => ModelIter::ContiguousOverlay {
                base,
                patches,
                front: 0,
                back: base.len(),
            },
            ModelValues::SharedRows(values) => ModelIter::SharedRows(values.iter()),
            ModelValues::SharedRowsOverlay { base, patches } => ModelIter::SharedRowsOverlay {
                base,
                patches,
                front: 0,
                back: base.len(),
            },
        }
    }

    pub(crate) fn shares_values_with(&self, other: &Self) -> bool {
        match (&self.values, &other.values) {
            (ModelValues::Contiguous(left), ModelValues::Contiguous(right)) => {
                Rc::ptr_eq(left, right)
            }
            (ModelValues::SharedRows(left), ModelValues::SharedRows(right)) => {
                Rc::ptr_eq(left, right)
            }
            (
                ModelValues::ContiguousOverlay {
                    base: left_base,
                    patches: left_patches,
                },
                ModelValues::ContiguousOverlay {
                    base: right_base,
                    patches: right_patches,
                },
            ) => {
                Rc::ptr_eq(left_base, right_base) && left_patches.shares_storage_with(right_patches)
            }
            (
                ModelValues::SharedRowsOverlay {
                    base: left_base,
                    patches: left_patches,
                },
                ModelValues::SharedRowsOverlay {
                    base: right_base,
                    patches: right_patches,
                },
            ) => {
                Rc::ptr_eq(left_base, right_base) && left_patches.shares_storage_with(right_patches)
            }
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_row_with(&self, other: &Self, row: usize) -> bool {
        match (&self.values, &other.values) {
            (ModelValues::Contiguous(left), ModelValues::Contiguous(right)) => {
                row < left.len() && row < right.len() && Rc::ptr_eq(left, right)
            }
            (
                ModelValues::Contiguous(left),
                ModelValues::ContiguousOverlay {
                    base: right,
                    patches,
                },
            )
            | (
                ModelValues::ContiguousOverlay {
                    base: right,
                    patches,
                },
                ModelValues::Contiguous(left),
            ) => row < left.len() && Rc::ptr_eq(left, right) && patches.get(row).is_none(),
            (
                ModelValues::ContiguousOverlay {
                    base: left_base,
                    patches: left_patches,
                },
                ModelValues::ContiguousOverlay {
                    base: right_base,
                    patches: right_patches,
                },
            ) if Rc::ptr_eq(left_base, right_base) => {
                match (left_patches.get(row), right_patches.get(row)) {
                    (None, None) => row < left_base.len(),
                    (Some(left), Some(right)) => Rc::ptr_eq(left, right),
                    _ => false,
                }
            }
            (ModelValues::SharedRows(left), ModelValues::SharedRows(right)) => left
                .get(row)
                .zip(right.get(row))
                .is_some_and(|(left, right)| Rc::ptr_eq(left, right)),
            _ => self
                .row_rc(row)
                .zip(other.row_rc(row))
                .is_some_and(|(left, right)| Rc::ptr_eq(left, right)),
        }
    }

    #[cfg(test)]
    fn row_rc(&self, row: usize) -> Option<&Rc<T>> {
        match &self.values {
            ModelValues::Contiguous(_) => None,
            ModelValues::ContiguousOverlay { patches, .. } => patches.get(row),
            ModelValues::SharedRows(values) => values.get(row),
            ModelValues::SharedRowsOverlay { base, patches } => {
                patches.get(row).or_else(|| base.get(row))
            }
        }
    }

    pub(crate) fn map_preserving_metadata<U, F>(&self, mut map: F) -> ModelRc<U>
    where
        F: FnMut(&T) -> U,
    {
        ModelRc {
            values: ModelValues::Contiguous(Rc::new(self.iter().map(&mut map).collect())),
            metadata: self.metadata.clone(),
        }
    }

    pub(crate) fn with_row_patches(&self, row_patches: BTreeMap<usize, T>) -> Self
    where
        T: Clone,
    {
        if row_patches.is_empty() {
            return self.clone();
        }
        let row_count = self.row_count();
        let row_patches = row_patches
            .into_iter()
            .filter(|(row, _)| *row < row_count)
            .map(|(row, value)| (row, Rc::new(value)))
            .collect::<BTreeMap<_, _>>();
        if row_patches.is_empty() {
            return self.clone();
        }

        let values = match &self.values {
            ModelValues::Contiguous(base) => ModelValues::ContiguousOverlay {
                base: Rc::clone(base),
                patches: PersistentRowPatchMap::empty(row_count).with_updates(row_patches),
            },
            ModelValues::ContiguousOverlay { base, patches } => ModelValues::ContiguousOverlay {
                base: Rc::clone(base),
                patches: patches.with_updates(row_patches),
            },
            ModelValues::SharedRows(base) => ModelValues::SharedRowsOverlay {
                base: Rc::clone(base),
                patches: PersistentRowPatchMap::empty(row_count).with_updates(row_patches),
            },
            ModelValues::SharedRowsOverlay { base, patches } => ModelValues::SharedRowsOverlay {
                base: Rc::clone(base),
                patches: patches.with_updates(row_patches),
            },
        };
        Self {
            values,
            metadata: self.metadata.clone(),
        }
    }

    pub(crate) fn metadata<M: Any>(&self) -> Option<&M> {
        self.metadata.as_deref()?.downcast_ref()
    }

    pub(crate) fn metadata_rc<M: Any>(&self) -> Option<Rc<M>> {
        Rc::clone(self.metadata.as_ref()?).downcast().ok()
    }
}

impl<T: fmt::Debug> fmt::Debug for ModelRc<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("ModelRc");
        match &self.values {
            ModelValues::Contiguous(values) => debug.field("values", values),
            ModelValues::ContiguousOverlay { base, patches } => debug
                .field("base_rows", &base.len())
                .field("overlay_rows", &patches.len()),
            ModelValues::SharedRows(values) => debug.field("values", values),
            ModelValues::SharedRowsOverlay { base, patches } => debug
                .field("base_rows", &base.len())
                .field("overlay_rows", &patches.len()),
        };
        debug
            .field("has_metadata", &self.metadata.is_some())
            .finish()
    }
}

impl<T: PartialEq> PartialEq for ModelRc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.row_count() == other.row_count() && self.iter().eq(other.iter())
    }
}

impl<T: Clone> From<Rc<VecModel<T>>> for ModelRc<T> {
    fn from(model: Rc<VecModel<T>>) -> Self {
        let values = match Rc::try_unwrap(model) {
            Ok(model) => model.values,
            Err(model) => model.values.clone(),
        };
        Self {
            values: ModelValues::Contiguous(Rc::new(values)),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::*;

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
}
