use glyphon::TextArea;

pub(crate) struct NativeBitmapAtlasTextArea<'a, 'text> {
    pub(super) text_area: &'a TextArea<'text>,
    pub(super) background_color: Option<[f32; 4]>,
}

impl<'a, 'text> NativeBitmapAtlasTextArea<'a, 'text> {
    pub(crate) fn new(text_area: &'a TextArea<'text>, background_color: Option<[f32; 4]>) -> Self {
        Self {
            text_area,
            background_color,
        }
    }
}
