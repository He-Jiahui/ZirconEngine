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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ModelRc<T> {
    values: Rc<Vec<T>>,
}

impl<T> Default for ModelRc<T> {
    fn default() -> Self {
        Self {
            values: Rc::new(Vec::new()),
        }
    }
}

impl<T: Clone> ModelRc<T> {
    pub(crate) fn row_count(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn row_data(&self, row: usize) -> Option<T> {
        self.values.get(row).cloned()
    }

    #[cfg(test)]
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }
}

impl<T: Clone> From<Rc<VecModel<T>>> for ModelRc<T> {
    fn from(model: Rc<VecModel<T>>) -> Self {
        Self {
            values: Rc::new(model.values.clone()),
        }
    }
}
