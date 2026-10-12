//! 文本管线测试按夹具、布局请求、测量缓存与 surface 缓存分组，覆盖同一帧的预热和发布链。
#[path = "fixtures.rs"]
mod fixtures;
#[path = "inline_widgets.rs"]
mod inline_widgets;
#[path = "layout_request.rs"]
mod layout_request;
#[path = "measure_cache.rs"]
mod measure_cache;
#[path = "render_extract_prewarm.rs"]
mod render_extract_prewarm;
#[path = "surface_cache.rs"]
mod surface_cache;
