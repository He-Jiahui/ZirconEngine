# 性能审查未验收模块

基线：2026-07-17 当前 Git 索引，共 **17,706** 个受跟踪 `.rs` 文件（排除 `dev/**` 参考引擎与构建输出）。本文件按模块文件夹压缩记录；审查时会把大目录继续拆成可完整验收的子目录，完成后整项移入 `review.md`。

## P0 · MVP 启动、运行时和基础编辑器

| module folder | files | acceptance focus |
|---|---:|---|
| `zircon_app/src/bin` | 11 | F0/F2 产品入口、诊断 viewer |
| `zircon_app/src/entry` | 136 | profile 选择、启动/退出、事件循环 |
| `zircon_app/src/plugins` + root files | 8 | 插件装配、presenter、root wiring |
| `zircon_runtime/src/core` | 784 | 生命周期、manager/framework、任务与诊断；验收时按子目录拆分 |
| `zircon_runtime/src/foundation` + root files | 12 | 基础契约与 root wiring |
| `zircon_runtime/src/scene` | 970 | F2 场景/ECS/调度；验收时按子目录拆分 |
| `zircon_runtime/src/input` | 22 | F2 输入事件与 action mapping |
| `zircon_runtime/src/graphics` | 1,386 | F2 最小渲染可达路径优先；按子目录拆分 |
| `zircon_runtime/src/render_graph` | 13 | pass/依赖/资源调度 |
| `zircon_runtime/src/rhi` + `rhi_wgpu` | 55 | 提交、同步、资源与 GPU marker/timestamp |
| `zircon_runtime/src/platform` | 39 | window/headless/event loop |
| `zircon_runtime/src/asset` | 486 | F1/F3 最小项目/资产路径优先 |
| `zircon_runtime/src/plugin` + `engine_module` | 586 | 插件发现、加载、注册、每帧回调 |
| `zircon_editor/src/core` | 178 | gateway、commands、jobs、message |
| `zircon_editor/src/scene` | 125 | F4 选择/修改/保存 |
| `zircon_editor/src/ui` | 4,193 | `ui/host` 250/250、`template_runtime` 44/44、`workbench` 327/327、`retained_host` root 19/19、`retained_host/app` 451/451、`retained_host/callback_dispatch` 135/135、`retained_host/host_contract/data` 77/77、`retained_host/host_contract/presenter` 31/31、`retained_host/host_contract/chrome_command_stream` 40/40、`retained_host/host_contract/globals` 16/16、`retained_host/host_contract/diagnostics` 9/9、`retained_host/host_contract/redraw` 7/7、`retained_host/host_contract/window` 38/38、`retained_host/host_contract/profiling_artifacts` 35/35、`retained_host/host_contract/profiling_hit_routes` 18/18、`retained_host/host_contract/native_keyboard` 13/13、`retained_host/host_contract/native_popup_dismiss` 3/3、`retained_host/host_contract/workbench_context_menu` 6/6、`host_page_overflow_menu` 1/1、`menu_popup_metrics` 1/1、`retained_host/host_contract/native_pointer/move_dispatch` 16/16、`retained_host/host_contract/native_pointer/scroll_dispatch` 18/18、`retained_host/host_contract/native_pointer/drag_resize` 21/21、`retained_host/host_contract/native_pointer/routing` 48/48、`retained_host/host_contract/native_pointer/menu_geometry` 27/27、`retained_host/host_contract/native_pointer/button_dispatch` 104/104、`retained_host/host_contract/paint_frame` 15/15、`retained_host/host_contract/paint_recording` 3/3、`retained_host/host_contract/paint_primitives` 26/26、`retained_host/host_contract/paint_geometry` 4/4、`retained_host/host_contract/paint_text` 30/30、`retained_host/host_contract/paint_theme` 6/6、`retained_host/host_contract/paint_workbench` 3/3、`retained_host/host_contract/paint_close_prompt` 5/5、`retained_host/host_contract/paint_debug_reflector_overlay` 4/4、`retained_host/host_contract/paint_diagnostics` 6/6、`retained_host/host_contract/surface_hit_test` 16/16、`template_popup_layout` 5/5、`template_geometry` 2/2、`template_component_family` 6/6、`frame_geometry` 4/4、`template_input_semantics` 2/2、`template_activation_semantics` 4/4、`retained_host/ui` root extras 6/6、`retained_host/ui/apply_presentation` 3/3、`retained_host/ui/workbench_window_projection` 5/5、`retained_host/ui/template_node_conversion` 1/1、`retained_host/ui/pane_data_conversion` 211/211、`retained_host/ui/tests` 10/10、`retained_host/asset_pointer` 23/23、`retained_host/hierarchy_pointer` 20/20、`retained_host/detail_pointer` 27/27、`retained_host/document_tab_pointer` 19/19、`retained_host/menu_pointer` 26/26、`retained_host/viewport_toolbar_pointer` 31/31、`retained_host/activity_rail_pointer` 23/23、`retained_host/drawer_header_pointer` 21/21、`retained_host/host_page_pointer` 20/20、`retained_host/welcome_recent_pointer` 20/20、`retained_host/shell_pointer` 8/8、`retained_host/tab_drag` 8/8、`retained_host/viewport` 26/26、`retained_host/route_intent` 2/2、`retained_host/workbench_preview_actions` 2/2 已静态读完但动态未验收；其余 retained host/MVP pane 继续按子目录推进 |
| `zircon_editor` root files | 2 | build/root wiring |

静态已读但动态未验收补充：`zircon_editor/src/ui/retained_host/host_contract/native_pointer/{chrome_damage,close_prompt_damage,pane_button_damage,redraw_result,tab_drag_damage,viewport_toolbar_damage}` + `resize_damage.rs` + `template_hover_damage.rs` **57/57**。

静态已读但动态未验收补充：`zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs` + `native_pointer/{constants.rs,state.rs}` **3/3**。

静态已读但动态未验收补充：`zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer.rs` + `paint_workbench_renderer/**` **102/102**。

静态已读但动态未验收补充：`zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/{mod.rs,template_nodes*,template_node_pipeline*,render_commands*,render_command_conversion*}` **61/61**，以及`zircon_runtime_interface/src/ui/surface/render/{command.rs,list.rs}` **2/2**；PERF-MVP-178的current-source Cargo、1/100/10,000-node build/hash/probe/sort/allocation counter和产品paint trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_viewport_scene_structure.rs,template_icon_assets.rs,template_property_axis_values.rs,template_dropdown_metrics.rs,template_icon_button_glyph_segments.rs,template_row_metrics.rs}` **6/6**；热点回链PERF-MVP-150/161/178，current-source Cargo、theme-lock/icon-cache/property-command allocation counter与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_asset_placeholder_visuals*,template_icon_button_glyph_kind*,template_icon_button_glyphs.rs,template_node_surface*,template_style_color*}` **12/12**；热点回链PERF-MVP-150/161/174/178，current-source Cargo、stable-generation glyph classification/icon raster/command build counter与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_icon_button_glyph_segments.rs,template_icon_button_glyph_shapes/**}` **35/35**（其中segment leaf已在上一切片读过，本切片完成34个shape/dispatch文件）；PERF-MVP-179的current-source Cargo、MVP asset-resolve/fallback counter、1/1k/10k command/draw/raster trace与pixel parity未完成。

静态已读但动态未验收补充：`paint_template_nodes/{sprite_atlas.rs,sprite_atlas/**,sprite_atlas_tests/**}` **9/9**；PERF-MVP-180的current-source Cargo、paint-thread filesystem/decode counters、multi-manifest/hot-reload/cache-bound与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{visual_assets.rs,visual_assets/**,visual_assets_tests/**}` **41/41**；PERF-MVP-181的current-source Cargo、candidate/filesystem/lock/RGBA-copy/cache-bound counter、worker miss/hot-reload与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_node_images.rs,template_node_images/**}` **4/4**；consumer热点回链PERF-MVP-178/181，current-source Cargo、stable-node tint/resource/command build与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_node_labels.rs,template_node_labels/**,template_node_labels_tests/**,template_node_text.rs,template_node_text/**}` **14/14**；热点回链PERF-MVP-156/161/174/178，current-source Cargo、per-node label allocation/theme-lock/text-command counter与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_style.rs,template_style/**,template_style_tests/**}` **18/18**；热点回链PERF-MVP-161/178，current-source Cargo、per-node state-resolution/theme-access/compiled-style counter与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{style_selector.rs,style_selector/**}` **157/157**；PERF-MVP-182/183的current-source Cargo、1/100/10,000-node theme-lock/role-allocation counter、theme-generation与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{material_primitives.rs,material_primitives/**}` **150/150**；PERF-MVP-184/185/186的current-source Cargo、handler/variant/theme/text/RGBA-mask/command规模counter、MVP产品命中与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{mui_x_primitives.rs,mui_x_primitives/**}` **53/53**；PERF-MVP-187/188的current-source Cargo、chart RGBA/raster/resource-generation、component theme-lock/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{material_state_layer.rs,material_state_layer/**,material_state_layer_tests/**}` **9/9**；PERF-MVP-189的idle零theme-read TDD、current-source Cargo、1/100/10,000-node counter与pixel trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_button_glyphs.rs,template_button_glyphs/**,template_buttons.rs,template_buttons/**,template_buttons_tests/**}` **29/29**；PERF-MVP-190的current-source Cargo、button key/style/theme-metrics/text/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_fields.rs,template_fields/**,template_fields_tests/**,template_field_stepper.rs,template_field_stepper/**}` **21/21**；PERF-MVP-191的current-source Cargo、field role/lowercase/label/theme-metrics/text/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_icon_buttons.rs,template_icon_buttons/**,template_icon_buttons_tests/**}` **18/18**；回链PERF-MVP-178/179/181/182/183，current-source Cargo、icon-button identity/context/theme-metrics/resource/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_axis_labels.rs,template_axis_labels/**,template_axis_labels_tests/**,template_axis_value_field_style.rs,template_axis_value_field_style/**,template_axis_value_fields.rs,template_axis_value_fields/**,template_axis_value_fields_tests/**}` **43/43**；PERF-MVP-192的current-source Cargo、Transform axis theme/metrics/palette/value-copy/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_inspector_row_geometry.rs,template_inspector_row_geometry/**,template_inspector_row_glyphs.rs,template_inspector_row_glyphs/**,template_inspector_row_kind.rs,template_inspector_row_kind/**,template_inspector_rows.rs,template_inspector_rows/**,template_inspector_rows_tests/**}` **38/38**；PERF-MVP-193的无分配bool修复/current-source Cargo，以及PERF-MVP-174/178/179/181的inspector text/resource/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_property_axis_values.rs,template_property_rows.rs,template_property_rows/**,template_property_rows_tests/**,template_row_metrics.rs}` **21/21**；PERF-MVP-194的current-source Cargo、property parse/Vec-String/theme-metrics/text/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_selection_control_geometry.rs,template_selection_control_geometry/**,template_selection_controls.rs,template_selection_controls/**,template_selection_controls_tests/**}` **26/26**；PERF-MVP-195的current-source Cargo、selection selector/theme-metrics/label/resource/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_slider_geometry.rs,template_slider_geometry/**,template_sliders.rs,template_sliders/**,template_sliders_tests/**}` **35/35**；PERF-MVP-196/197的current-source Cargo、editor/runtime tick预算修复、slider theme-metrics/text/static-dynamic command counter与GPU/Softbuffer trace未完成；runtime `ui/surface/render/sliders.rs`仅聚焦审查tick路径，整文件仍待验收。

静态已读但动态未验收补充：`paint_template_nodes/{template_segmented_control_geometry.rs,template_segmented_control_geometry/**,template_segmented_controls.rs,template_segmented_controls/**,template_segmented_controls_tests/**}` **28/28**；PERF-MVP-198的option二次copy/selected lowercase局部修复、current-source Cargo、option/style/theme-metrics/text/command规模counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_alert_glyphs.rs,template_alert_glyphs/**,template_alerts.rs,template_alerts/**,template_alerts_tests/**}` **29/29**；PERF-MVP-199的current-source Cargo、identity format/lowercase/text bytes、alert/toast theme与glyph/total command规模counter、fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_chip_glyphs.rs,template_chip_glyphs/**,template_chips.rs,template_chips/**,template_chips_tests/**}` **18/18**；PERF-MVP-200的current-source Cargo、chip identity/label/palette/metrics/command规模counter、chevron fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_status_control_geometry.rs,template_status_control_geometry/**,template_status_controls.rs,template_status_controls/**,template_status_controls_tests/**,template_status_glyphs.rs,template_status_glyphs/**}` **34/34**；PERF-MVP-201的current-source Cargo、30s idle与status split/format/measure/theme-metrics/static-dynamic command counter、glyph fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_list_row_glyphs.rs,template_list_row_glyphs/**,template_list_rows.rs,template_list_rows/**,template_list_rows_tests/**}` **20/20**；PERF-MVP-202的current-source Cargo、visible/offscreen visited、row selector/theme-metrics/label/resource/command规模counter、adornment fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_table_rows.rs,template_table_rows/**,template_table_rows_tests/**}` **28/28**；PERF-MVP-203的current-source Cargo、visible/offscreen visited、row-column layout/fixed+actual measurement/row_data-String/theme-metrics-resource/command规模counter、action fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_tree_row_geometry.rs,template_tree_row_geometry/**,template_tree_row_glyphs.rs,template_tree_row_glyphs/**,template_tree_rows.rs,template_tree_rows/**,template_tree_rows_tests/**}` **32/32**；PERF-MVP-204的current-source Cargo、visible/offscreen/depth guide visited、selector/theme-metrics/label/resource/static-dynamic command counter、four-glyph fallback命中及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_popup_row_adornments.rs,template_popup_row_adornments/**,template_popup_rows.rs,template_popup_rows/**,template_popup_rows_tests/**}` **45/45**；PERF-MVP-205的clip-before-clone/单次无分配adornment局部修复、current-source Cargo、visible/offscreen row_data/flag/String/theme-metrics-text-command counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_section_title_glyphs.rs,template_section_title_glyphs/**,template_section_titles.rs,template_section_titles/**,template_section_titles_tests/**}` **23/23**；PERF-MVP-206的current-source Cargo、title identity/theme-metrics/label-String/surface-icon-text command规模counter、glyph产品预算及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_tooltip_glyphs.rs,template_tooltip_glyphs/**,template_tooltips.rs,template_tooltips/**,template_tooltips_tests/**}` **19/19**；PERF-MVP-207的current-source Cargo、1k hover target generation、tooltip theme-metrics/text/surface-glyph command counter、arrow mask/fallback预算及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_notification_center.rs,template_notification_center/**}` **11/11**；PERF-MVP-208的current-source Cargo、0/1/100/10k notification retention/generation/unread/visible-offscreen row_data/String/command/frame-memory counter、EditorLayout09回传及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_shell_panels.rs,template_shell_panels/**,template_shell_panels_tests/**}` **16/16**；无新增独立热点，PERF-MVP-161/178的current-source Cargo、stable shell selector/theme-metrics/command/damage counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_dialogs.rs,template_dialogs/**}` **22/22**；PERF-MVP-209的current-source Cargo、closed/stable-open dialog metrics-palette/variant-severity/action row_data-String-measurement/title-body/command counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_drag_overlay.rs,template_drag_overlay/**}` **9/9**；PERF-MVP-210的专用测试、current-source Cargo、inactive/1k same-payload move label bytes/static-dynamic build/geometry-command/damage/frame counter与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_command_palette.rs,template_command_palette/**}` **39/39**，并聚焦回查`core/commands/registry.rs`与`retained_host/app/command_palette_actions.rs`；PERF-MVP-211的current-source Cargo、catalog generation/when/search/owned bytes、visible-offscreen row_data/theme-metrics/text-command/keystroke p95 counter、Editor08回传及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_material_feedback.rs,template_material_feedback/**,template_material_feedback_tests/**}` **21/21**；PERF-MVP-212的current-source Cargo、typed arc/ring或有界完整identity raster cache、stable/indeterminate raster-allocation-pixel-op-key-format/upload counter、Softbuffer像素与current-source RenderDoc trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_dropdowns.rs,template_dropdowns/**,template_dropdowns_tests/**,template_dropdown_glyphs.rs,template_dropdown_glyphs/**,template_dropdown_metrics.rs}` **19/19**；PERF-MVP-213的一次label/metrics局部合并、current-source Cargo、changed/stable label-theme-metrics-resource-command counter、scale与GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_sample_grid.rs,template_sample_grid/**,template_sample_grid_tests/**}` **11/11**；PERF-MVP-214的Editor07 grid generation回传、batched dashed-line/marker primitive、current-source Cargo、tick-point-format-static-dynamic-command/CPU counter及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_timeline_strip.rs,template_timeline_strip/**,template_timeline_strip_tests/**}` **11/11**；PERF-MVP-215的bounded single-generation tick直接修复、Editor07 timeline generation回传、current-source Cargo、stable/scrub tick-format-key-command/CPU counter及GPU/Softbuffer trace未完成。

静态已读但动态未验收补充：`paint_template_nodes/{template_weight_heatmap.rs,template_weight_heatmap/**,template_weight_heatmap_tests/**}` **10/10**；PERF-MVP-216的single-source-projection+bounded-grid直接修复、Editor07 worker heat generation/Render13 texture-compute回传、current-source Cargo、C×R×S/command/upload/CPU counter及GPU/Softbuffer trace未完成。

## P1 · MVP 直接支撑与回归测试

| module folder | files | acceptance focus |
|---|---:|---|
| `zircon_runtime/src/dynamic_api` | 56 | editor/runtime session 与 render bridge |
| `zircon_runtime/src/ui` | 783 | F4 可达 UI、布局、paint |
| `zircon_runtime/src/text` | 197 | MVP 字体/文本最小路径 |
| `zircon_runtime/src/diagnostic_log` | 7 | 高频日志与同步 sink |
| `zircon_runtime/src/operation` | 7 | 操作队列/错误路径 |
| `zircon_runtime/src/tests` | 3,466 | 对应生产模块随批次验收 |
| `zircon_editor/src/tests` | 487 | 对应生产模块随批次验收 |
| `zircon_app/src/tests` | 3 | 产品入口契约 |
| `zircon_runtime_interface` | 377 | 动态接口、DTO 与调用开销 |
| `zircon_plugins/first_party_runtime_catalog` | 2 | MVP plugin 集合 |
| `zircon_plugins/first_party_editor_catalog` | 3 | MVP editor plugin 集合 |
| `zircon_plugins/plugin_sdk` + examples | 23 | 插件 ABI/调用边界 |
| `zircon_plugins/native_window_hosting` | 6 | F0/F4 host 路径 |
| `zircon_plugins/runtime_diagnostics` | 6 | profiling/diagnostics 插件 |

## P2 · 非 MVP 运行时、编辑器工具与插件

| module folder | files | acceptance focus |
|---|---:|---|
| `zircon_runtime/src/animation` | 28 | evaluator、graph 与 per-frame allocation |
| `zircon_runtime/src/script` | 96 | VM/binding/callback 频率 |
| `zircon_runtime/src/navigation` | 12 | path/query/update 调度 |
| `zircon_runtime/src/builtin` | 27 | builtin 调用热区 |
| `zircon_runtime/src/bin` | 40 | 工具/fixture/benchmark 入口 |
| `zircon_runtime/src/reflection_macros` | 8 | 生成与运行时访问开销 |
| `zircon_plugins/sound` | 1,533 | realtime callback、DSP、锁/分配、worker |
| `zircon_plugins/hybrid_gi` | 192 | 高级渲染 CPU/GPU |
| `zircon_plugins/virtual_geometry` | 235 | streaming/culling/GPU-driven |
| `zircon_plugins/net` | 171 | I/O、复制、tick 频率 |
| `zircon_plugins/animation` + `animation_graph` | 174 | 动画更新与 graph |
| `zircon_plugins/rendering` | 113 | render plugin lifecycle/pass |
| `zircon_plugins/navigation` | 101 | navigation plugin |
| `zircon_plugins/ai` | 63 | AI tick/scheduling |
| `zircon_plugins/physics` | 85 | simulation step/parallelism |
| `zircon_plugins/particles` | 47 | simulation/upload/draw |
| `zircon_plugins/texture_importer` | 33 | decode/cache/import |
| `zircon_plugins/asset_importers` | 26 | import I/O/cache |
| `zircon_plugins/texture` + `terrain` + `tilemap_2d` | 34 | resource/update/render paths |
| 其余首方插件目录 | 106 | 小型 importer/editor/tool plugins，逐目录验收 |
| `zircon_hub` | 133 | 项目管理 UI/I/O（非 F0-F5 editor host） |
| `zircon_reflect_derive` | 5 | proc-macro 构建性能 |
| `tools` Rust | 15 | 辅助工具性能/正确性 |

## 守恒检查

上表是面向执行的压缩视图；P2 的“其余首方插件目录”由基线清单中未单列的小目录汇总。每次里程碑验收必须用 Git 索引重新计算精确集合，并验证：

```text
review 精确文件集合 ∩ pending 精确文件集合 = ∅
review 精确文件集合 ∪ pending 精确文件集合 = 当前受跟踪 Rust 文件集合
```
