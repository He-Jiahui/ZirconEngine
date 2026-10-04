//! 栅格请求、结果和错误是框架中立契约；Runtime 的字形栅格服务提供实现，UI 侧负责校验字体代际。
mod bitmap_format;
mod error;
mod hinting;
mod mode;
mod receipt;
mod request;
mod smoothing;
mod synthetic_style;

pub use bitmap_format::TextGlyphBitmapFormat;
pub use error::TextGlyphRasterError;
pub use hinting::TextGlyphRasterHinting;
pub use mode::TextGlyphRasterMode;
pub use receipt::TextGlyphRasterReceipt;
pub use request::TextGlyphRasterRequest;
pub use smoothing::TextGlyphRasterSmoothing;
pub use synthetic_style::TextGlyphSyntheticStyle;
