use std::any::Any;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

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
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    pub(crate) fn from_rgba8(buffer: SharedPixelBuffer<Rgba8Pixel>) -> Self {
        Self {
            rgba: buffer.rgba,
            width: buffer.width,
            height: buffer.height,
        }
    }

    pub(crate) fn load_from_path(path: &Path) -> Result<Self, image::ImageError> {
        let image = image::open(path)?.to_rgba8();
        let (width, height) = image.dimensions();
        Ok(Self {
            rgba: image.into_raw(),
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
    values: Rc<Vec<T>>,
    metadata: Option<Rc<dyn Any>>,
}

impl<T> Default for ModelRc<T> {
    fn default() -> Self {
        Self {
            values: Rc::new(Vec::new()),
            metadata: None,
        }
    }
}

impl<T: Clone> ModelRc<T> {
    pub(crate) fn with_metadata<M: Any>(values: Vec<T>, metadata: M) -> Self {
        Self {
            values: Rc::new(values),
            metadata: Some(Rc::new(metadata)),
        }
    }

    pub(crate) fn row_count(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn row_data(&self, row: usize) -> Option<T> {
        self.values.get(row).cloned()
    }

    pub(crate) fn get(&self, row: usize) -> Option<&T> {
        self.values.get(row)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }

    #[cfg(test)]
    pub(crate) fn shares_values_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.values, &other.values)
    }

    pub(crate) fn map_preserving_metadata<U, F>(&self, map: F) -> ModelRc<U>
    where
        F: FnMut(&T) -> U,
    {
        ModelRc {
            values: Rc::new(self.values.iter().map(map).collect()),
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
        formatter
            .debug_struct("ModelRc")
            .field("values", &self.values)
            .field("has_metadata", &self.metadata.is_some())
            .finish()
    }
}

impl<T: PartialEq> PartialEq for ModelRc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<T: Clone> From<Rc<VecModel<T>>> for ModelRc<T> {
    fn from(model: Rc<VecModel<T>>) -> Self {
        let values = match Rc::try_unwrap(model) {
            Ok(model) => model.values,
            Err(model) => model.values.clone(),
        };
        Self {
            values: Rc::new(values),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
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
            values: Rc::new(vec![CloneProbe(Arc::clone(&clone_count))]),
            metadata: None,
        };

        assert!(model.get(0).is_some());
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
    }
}
