# Frameworks 05 M1 跨域契约签名基线

## 1. 扫描口径

- 机器基线：[`2026-07-10-runtime-domain-dependencies.json`](2026-07-10-runtime-domain-dependencies.json)。
- 扫描 owner：`tools/runtime_domain_dependency_audit.py`。
- 仅扫描 `zircon_runtime/src/<domain>/**/*.rs` 生产 owner；排除 `tests/`、`tests.rs`、`*_tests.rs`、`test_*.rs` 和 crate-root facade。
- 2026-07-14 独立复审发现旧扫描器漏掉 bare/grouped root imports，并把注释与字符串计为引用；原 2399/80 与 2401/79 总数均已作废，不再作为验收证据。
- 修复后的扫描器以 baseline 首次提交 `f7a320904d681fb30dede6d5b222fc943cdeb3a7` 的精确源码树重算为 2001 references / 86 edges。该快照中 asset → ui = 0、graphics → ui = 4、ui → graphics = 21、graphics → scene = 1；完整矩阵与逐条 `{path,line,source}` 由 JSON 持有。

## 2. S1 共享文本契约草案

目标 owner：`core/framework/text/`；只持有请求、结果和服务 trait，不引用 `ui`、`graphics`、wgpu 或字体数据库实现。

```rust
pub struct TextShapeRequest<'a> {
    pub text: &'a str,
    pub language: Option<&'a str>,
    pub direction: TextDirection,
    pub writing_mode: TextWritingMode,
    pub font: TextFontRequest<'a>,
}

pub struct TextShapeResult {
    pub runs: Vec<TextShapeRun>,
    pub metrics: TextLayoutMetrics,
    pub resolved_direction: TextDirection,
}

pub trait TextLayoutService: Send + Sync {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode;
    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection;
    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError>;
}
```

切换要求：graphics 与 ui 都消费该 trait/DTO；`graphics::text` 不再作为 UI 的共享底座，`ui::text` 也不再被 graphics 反向调用。

## 3. S2 AssetLoaderRegistry 草案

目标 owner：asset 域。实现复核发现现有 `AssetImporterHandler` + `AssetImporterRegistry` 已完整覆盖该签名，不再新增一套同义 `AssetDocumentLoader` / `AssetLoaderRegistry`。

```rust
pub trait AssetImporterHandler: Debug + Send + Sync {
    fn descriptor(&self) -> &AssetImporterDescriptor;
    fn import(&self, context: &AssetImportContext)
        -> Result<AssetImportOutcome, AssetImportError>;
}

impl AssetImporterRegistry {
    pub fn register_arc(
        &mut self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), AssetImporterRegistryError>;
    pub fn select(&self, source_path: &Path)
        -> Result<Arc<dyn AssetImporterHandler>, AssetImportError>;
}
```

切换结果：`ui_document_importer` runtime plugin 通过 `RuntimeExtensionRegistry::register_asset_importer(...)` 注册 `.zui`；builtin runtime module 组装把该 registry 注入 `AssetModule`。asset 内旧 `.zui` backend/转换 owner 已删除，三处 `crate::ui` 生产引用与 `lib.rs` 声明顺序注释已清零。

## 4. S3 RenderSceneExtract 契约草案

目标 owner：`core/framework/render/extract/`；snapshot 只含稳定 DTO/资源句柄，不暴露 ECS world、scene storage 或具体 manager。

```rust
pub struct RenderSceneExtractRequest {
    pub frame_index: u64,
    pub viewport: RenderViewportDescriptor,
}

pub struct RenderSceneExtractSnapshot {
    pub entities: Vec<RenderEntitySnapshot>,
    pub lights: Vec<RenderLightSnapshot>,
    pub cameras: Vec<RenderCameraSnapshot>,
    pub resources: Vec<RenderResourceHandle>,
}

pub trait RenderSceneExtractSource: Send + Sync {
    fn extract_render_scene(
        &self,
        request: &RenderSceneExtractRequest,
    ) -> Result<RenderSceneExtractSnapshot, RenderExtractError>;
}
```

切换要求：graphics 只消费 snapshot；scene 负责从 ECS 批量生成，禁止 graphics 触达 scene ECS query/world owner。

## 5. S4 版本化 manager handle 草案

现有 `*ManagerHandle` 内部直接持有 `Arc<dyn Trait>`，不满足 index+version 悬垂检测。目标句柄只持身份，resolver 在使用点解析。

```rust
pub struct ManagerServiceHandle<T: ?Sized> {
    pub index: u32,
    pub generation: u32,
    pub service: RegistryName,
    marker: PhantomData<fn() -> T>,
}

pub trait ManagerServiceResolver {
    fn resolve<T: ?Sized + Send + Sync + 'static>(
        &self,
        handle: ManagerServiceHandle<T>,
    ) -> Result<Arc<T>, CoreError>;
}
```

切换要求：跨域状态只保存 typed handle；`Arc<具体实现>` 与 `Arc<dyn Trait>` 都不跨帧长期保存在邻域对象内。generation 不匹配必须返回 typed stale-handle error。

## 6. M2–M4 验收归属

- M2：asset → ui 从 3 降到 0；crate-root 声明顺序注释删除。
- M3：graphics → ui 与 ui → graphics 都降到 0；共享文本实现成为独立 owner。
- M4：graphics → scene 只允许明确批准的 framework handle/snapshot 路径；manager 裸 Arc 跨域持有清零。
- 每个切片更新机器 JSON 基线并在输出记录中写前后计数，禁止只改文档状态。
