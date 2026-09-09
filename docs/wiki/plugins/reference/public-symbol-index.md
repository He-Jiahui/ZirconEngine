---
related_code:
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/declaration.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/test.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src
plan_sources:
  - user: 2026-09-09 插件公开接口完整参考
tests:
  - zircon_plugins/plugin_sdk/src
doc_type: api-reference
title: Plugin SDK 公开符号索引
status: source-audited
---

# Plugin SDK 公开符号索引

本页用于从 IDE 或 rustdoc 反查接口。方法名按模块列出；“错误/前置条件”比签名更重要，因为很多 builder 方法本身不返回 `Result`，错误会延迟到 `build`、registry 或宿主选择阶段。

## crate feature 与 re-export

|feature|模块|主要 re-export|
|---|---|---|
|`declaration`|`declaration`|PluginDeclaration、PluginTarget、PluginPlatform、PluginMaturityLevel、PluginPackaging、PluginCapabilityRole、PluginPackageRole、NativePluginEntryDeclaration|
|`runtime`|runtime、manifest、registration、prelude、test|RuntimePluginDeclaration、PluginManifestBuilder、PluginModuleBuilder、ImporterRuntimeManifestBuilder、RuntimePluginRegistrationBuilder、TestRuntime、PluginInterface、BridgeImport、WeakBridge|
|`editor`|editor|EditorPluginDeclaration、zircon_editor、zircon_runtime|
|`editor_contribution`|editor_contribution|EditorContributionBuilder|
|`native`|native、dist|ABI structs、status helpers、manifest codecs、dist macros|

关闭不需要的 feature 可避免 editor/native 代码进入最终制品。SDK 默认 feature 是 `runtime`。

## declaration.rs

|符号|签名/返回|说明|
|---|---|---|
|`PluginDeclaration::new`|`const fn(...)->Self`|构造静态 package declaration|
|`id`|`const fn(self)->&'static str`|canonical package ID|
|`display_name`|`const fn`|用户显示名|
|`category`|`const fn`|catalog 分类|
|`module_name`|`const fn`|主模块名|
|`declared_targets`|`const fn`|声明 target 切片|
|`declared_platforms`|`const fn`|声明 platform 切片|
|`capabilities`|`const fn`|能力名称切片|
|`capability_roles`|`const fn`|与能力按 index 对齐|
|`declared_maturity`|`const fn`|成熟度标签|
|`declared_packaging`|`const fn`|默认制品策略|
|`package_role`|`const fn`|生产/工具/样例/fixture|
|`with_package_role`|`const fn(self,...)->Self`|复制并替换 package role|
|`target_modes`|`fn(self)->Vec<RuntimeTargetMode>`|runtime feature 投影|
|`supported_platforms`|`fn(self)->Vec<ExportTargetPlatform>`|runtime feature 投影|
|`maturity`|`fn(self)->RuntimePluginMaturity`|枚举映射|
|`default_packaging`|`fn(self)->Vec<ExportPackagingStrategy>`|枚举映射|
|`module_descriptor`|`fn(self)->ModuleDescriptor`|用 module name/description 构造|
|`runtime_declaration`|`fn(self, crate_name)->RuntimePluginDeclaration`|canonical ID 与 role 长度校验|
|`runtime_descriptor`|`fn(self, crate_name)->RuntimePluginDescriptor`|直接完成 descriptor|

`NativePluginEntryDeclaration::new(name, cstr)`、`name()`、`cstr()` 用于把原生入口名和 NUL 结尾字节绑定。`cstr` 不得包含内部 NUL。

## runtime.rs

|方法|作用|典型错误|
|---|---|---|
|`new`|创建 descriptor builder|runtime ID 与 package ID 不一致|
|`with_category`|设置分类|空分类导致 catalog 难检索|
|`with_enabled_by_default`|默认选中|工具插件误启用|
|`with_required_by_default`|默认必需|缺失时阻塞整组|
|`with_target_modes`|覆盖 target|与 module targets 不一致|
|`with_init_level`|设置 InitLevel|过早访问未就绪 manager|
|`with_module_descriptor`|绑定核心模块|缺 descriptor 无法激活|
|`with_module_dependency`|追加依赖|形成环或引用缺失|
|`with_capability`|追加 runtime capability|命名空间不稳定|
|`with_system_sets`|批量写集合|集合名重复|
|`with_system_anchors`|批量写排序锚点|anchor 不存在|
|`with_maturity`|设置成熟度|把 stub 标 Stable|
|`with_capability_status`|写 capability 状态|required 与 optional 混淆|
|`with_optional_feature`|挂 feature bundle|primary dependency 冲突|
|`with_provided_interface`|写接口 manifest|interface ID 重复|
|`with_provided_interface_id`|快捷接口 ID|缺 schema/版本语义|
|`with_default_packaging`|设置包装策略|native 无 dist crate|
|`with_package_role`|设置包角色|fixture 进入生产 catalog|
|`descriptor`|clone builder 构造 descriptor|频繁调用会复制 vectors|
|`package_manifest`|descriptor 后投影 manifest|字段应与 descriptor 一致|
|`into_descriptor`|消费 builder|之后不能再修改 declaration|

## manifest builders

`PluginManifestBuilder` 方法完整集合：`new`、`with_category`、`with_package_role`、`with_description`、`with_maturity`、`with_supported_targets`、`with_supported_platforms`、`with_capability`、`with_capabilities`、`with_default_packaging`、`with_asset_root`、`with_content_root`、`with_module`、`build`。

`PluginModuleBuilder` 方法：`runtime`、`editor`、`native`、`vm`、`new`、`with_description`、`with_init_level`、`with_module_dependency`、`with_module_dependencies`、`with_target_modes`、`with_capabilities`、`with_system_sets`、`with_system_anchors`、`build`。

`PluginFeatureBundleBuilder` 方法：`new`、`with_dependency`、`with_primary_dependency`、`with_required_dependency`、`with_capability`、`with_capabilities`、`with_runtime_capability_module`、`with_editor_capability_module`、`with_runtime_module`、`with_runtime_module_from_builder`、`with_editor_module`、`with_editor_module_from_builder`、`with_default_packaging`、`enabled_by_default`、`build`。

## registration.rs

|类型/方法|约束|
|---|---|
|`RuntimePluginRegistrationBuilder::new(&mut RuntimeExtensionRegistry)`|独占 registry 借用|
|`module(name)`|intern owner，返回 `Result<RuntimePluginModuleRegistration>`|
|`module_name` / `owner`|读取当前模块身份|
|`runtime_scene_system`|factory 为 `Send + Sync + 'static`|
|`resource<T>`|T 必须实现 `Resource`|
|`component`|提交 `ComponentTypeDescriptor`|
|`event<E>`|E 必须实现 `Event`|
|`plugin_option`|写选项 manifest|
|`plugin_event_catalog`|写事件 catalog|
|`export_interface<T>`|T 为 `PluginInterface + ?Sized`，实现使用 `Arc`|
|`import_interface<T>`|返回 `BridgeImport<T>`|
|`owner_revocation_listener`|注册撤销回调|
|system builder `in_set`|收集字符串，register 时 intern|
|`with_order`|整数排序键|
|`with_tick_policy`|覆盖 stage 默认策略|
|`before` / `after`|追加 `SystemOrderingConstraint`|
|`register`|提交系统并返回 registry error|

## editor.rs/editor_contribution.rs

`EditorPluginDeclaration`：`new`、`with_category`、`with_package_role`、`with_description`、`with_maturity`、`with_capability`、`with_capabilities`、`with_runtime_event_consumer_registration`、`mirrors_runtime`、`mirrors_runtime_manifest`、`with_asset_root`、`with_content_root`、`descriptor`、`base_manifest`、`package_manifest`、`capabilities`、`mirrored_runtime_package_id`、`runtime_event_consumers`、`registration_report`。

`EditorContributionBuilder`：`new`、`view`、`drawer`、`menu`、`command`、`command_with_execution_contract`、`asset_type`、`settings_page`、`localization_bundle`、`tool_resource_kind`、`build`。所有贡献 ID 必须在 package 命名空间内唯一；`build` 失败时不得注册部分 batch。

## native.rs helpers

|函数|行为|
|---|---|
|`callback_status`|构造 code + diagnostics 指针|
|`owned_bytes`|Vec 转移为 owned ABI buffer|
|`command_manifest_v4_to_toml/from_toml`|命令 manifest 编解码|
|`command_manifest_v4_is_current_and_dense`|校验 schema/slot/name/上限|
|`registration_manifest_v3_to_toml/from_toml`|注册 manifest 编解码|
|`registration_manifest_v3_schema_is_current`|检查 registration schema|
|`free_owned_bytes_v3`|校验 shape/token 后释放|
|`bytes_from_slice`|null/zero 安全地生成 borrowed slice|
|`catch_native_callback_panic`|将 unwind 转 PANIC status|
|`host_supports_all_capabilities_v3`|全部能力满足；空列表 true|
|`host_supports_any_capability_v3`|任一满足；空列表 false|
|`host_supports_capability_v3`|单能力版本与授权检查|
|`capability_list_contains`|解析换行/逗号/分号列表|

## test.rs

`TestRuntimeBuilder` 以 `with_fixed_timestep`、`without_fixed_timestep`、`with_max_fixed_steps`、`with_base_modules`、`without_base_modules`、`with_runtime_plugin(s)`、`with_registration_report`、`with_feature_registration_report`、`without_scene_runtime_extension_plan`、`without_base_module_activation`、`without_plugin_module_activation` 配置，`build` 返回 `Result<TestRuntime, TestRuntimeError>`。

`TestRuntime` 的 `runtime`、`into_runtime`、`handle`、`extension_report`、`activated_modules`、`resolve_manager`、`create_default_level`、`advance_time_by`、`advance_time_by_seconds`、`tick_level_seconds` 覆盖 manager、时间和 scene-system 验证。

## 符号验收清单

- [ ] 每个公开方法在至少一个本目录页面有调用形状。
- [ ] 每个 `Result` 的错误类型与恢复策略已记录。
- [ ] 每个 unsafe/native 函数都有 null、长度、所有权说明。
- [ ] 每个 builder 的默认值和 consuming 行为已说明。
- [ ] runtime/editor/native 三种 feature 的边界通过 Cargo feature 测试。
- [ ] API 改动同步更新 manifest schema、ABI 版本或迁移说明。
