//! 栅格请求、结果和错误是框架中立契约；Runtime 的字形栅格服务提供实现，UI 侧负责校验字体代际。
#[path = "bitmap_format.rs"]
mod bitmap_format;
#[path = "error.rs"]
mod error;
#[path = "hinting.rs"]
mod hinting;
#[path = "mode.rs"]
mod mode;
#[path = "receipt.rs"]
mod receipt;
#[path = "request.rs"]
mod request;
#[path = "smoothing.rs"]
mod smoothing;
#[path = "synthetic_style.rs"]
mod synthetic_style;

pub use bitmap_format::TextGlyphBitmapFormat;
pub use error::TextGlyphRasterError;
pub use hinting::TextGlyphRasterHinting;
pub use mode::TextGlyphRasterMode;
pub use receipt::TextGlyphRasterReceipt;
pub use request::TextGlyphRasterRequest;
pub use smoothing::TextGlyphRasterSmoothing;
pub use synthetic_style::TextGlyphSyntheticStyle;
