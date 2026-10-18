# UI SDF 字体真实 Bake 收束计划

**Summary**
- 生产代码只落在 `zircon_runtime/src/graphics/scene/scene_renderer/ui/**`，不碰 RHI、render graph、VG/GI、render/plugin cutover，也不改 shared `UiRenderCommand` / template API。
- 选择 `fontsdf = "0.5.3"` 作为本轮第三方 SDF bake crate：它直接提供 `rasterize_sdf` / `rasterize_indexed_sdf` 输出单通道 `Vec<u8>`，能保留当前 `R8Unorm` atlas 和 `sdf_text.wgsl` 的 `.r` 采样路径。
- `fdsm` / RGB MSDF 暂不进入本轮；它适合后续 RGB MSDF milestone，但当前会额外牵动 atlas format、shader 语义和质量调参。

**Key Changes**
- 新增 renderer-local bake 层：`sdf_font_bake.rs`，只向 `sdf_render.rs` 暴露私有数据结构：
  - 加载 `font_asset.rs` 已解析出的字体 manifest/source path，读取字体 bytes 并构建 `fontsdf::Font`。
  - 按 `SdfAtlasGlyphKey` bake 非空白 glyph，输出 alpha bitmap、advance、bearing、bitmap 尺寸、missing/visible 标记。
  - whitespace 不 bake、不进 atlas，只通过字体 metrics 贡献 advance。
  - missing glyph 默认策略：优先使用字体 `.notdef`，若输出为空或 glyph index 为 0 且不可见，则该字符稳定跳过可见 quad，但保留一个保守 advance。
- `ScreenSpaceUiSdfAtlas` 继续负责 key、slot、run、rect；必要时只在 UI 私有层调整 slot 常量到更安全的固定尺寸，测试同步更新，不引入可变 atlas allocator。
- `ScreenSpaceUiSdfRenderer` 删除 rounded-rect placeholder 生成逻辑，改为：
  - 调用 bake 层生成 atlas alpha bytes。
  - 按真实 glyph metrics 计算 quad x/y、advance、alignment、clip。
  - 继续上传 `R8Unorm` atlas，shader 保持单通道 SDF path。
- 支持性改动仅限依赖/文档/测试：`zircon_runtime/Cargo.toml` 和 `Cargo.lock` 增加 `fontsdf`；更新 `docs/ui-and-layout/shared-ui-template-runtime.md` 说明 placeholder 已替换为真实字体轮廓 SDF。

**Test Plan**
- 保留并更新现有 UI 私有单元测试：
  - `sdf_atlas`: key、slot、whitespace、rect、run 稳定性继续通过。
  - `sdf_draw_plan`: glyph quad、clip、alignment、advance 改为基于真实 metrics 后继续通过。
- 新增 `sdf_font_bake` 单元测试：
  - `A`、`I`、`O` 的 alpha pattern 不同。
  - glyph mask 不等于旧 rounded-rect placeholder。
  - whitespace 不产生 atlas bitmap，但 advance 大于 0。
  - missing glyph 使用稳定 `.notdef` 或 skip 策略，不 panic。
- 扩展 capture test：
  - `UiTextRenderMode::Sdf` 文本与 background-only frame 做 pixel delta。
  - 验证 glyph footprint 稀疏，且 `AIO` 轮廓不是整块占位带。
- Milestone testing stage 命令：
  - `cargo test -p zircon_runtime --lib sdf_atlas --locked --jobs 1`
  - `cargo test -p zircon_runtime --lib sdf_draw_plan --locked --jobs 1`
  - `cargo test -p zircon_runtime --lib sdf_font_bake --locked --jobs 1`
  - `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 -- --test-threads=1 --nocapture`
  - `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime`

**Assumptions**
- “只改 ui/**”解释为生产 renderer 代码边界；引入第三方 crate 必须允许 `zircon_runtime/Cargo.toml` / `Cargo.lock` 的依赖更新，capture/docs 也按验证与仓库文档规则更新。
- 当前只做 ASCII/UI 常用文本的最小真实 SDF bake；复杂 shaping、bidi、完整 fallback chain、RGB MSDF 留到下一 milestone。
- crate 选择依据：[`fontsdf` API](https://docs.rs/fontsdf/0.5.3/fontsdf/struct.Font.html) 提供 SDF rasterize 和 metrics；[`fdsm`](https://docs.rs/fdsm/0.8.0/fdsm/) 是纯 Rust MSDF 但会自然导向 RGB atlas/shader 后续升级。
