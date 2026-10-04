//! Native distribution entry helpers.
//!
//! This module keeps one-file `cdylib` exports out of the low-level ABI owner
//! in `native.rs`. A native plugin crate should define its manifests and
//! callbacks, then invoke one of these macros once at crate root.
//! 将插件声明、回调与静态 ABI 表组装成原生动态库入口。
//! 插件在 crate 根部调用一次发布宏；底层 ABI 载体和宿主协商规则由 `native` 模块定义。

#[doc(hidden)]
#[macro_export]
macro_rules! __zircon_native_dist_optional_cstr_ptr_v3 {
    (None) => {
        ::core::ptr::null()
    };
    (Some($value:expr)) => {
        ($value).as_ptr().cast()
    };
}

/// 生成同一动态库的 Runtime 与 Editor 描述符、入口及静态报告。
/// 所有传给 C 字符串字段的值须在库的存活期有效并以 NUL 结尾；宿主加载时仍会校验 ABI、能力与行为回调。
#[macro_export]
macro_rules! native_dist_plugin_v3 {
    (
        plugin_id: $plugin_id:expr,
        package_manifest: $package_manifest:expr,
        descriptor_abi_version: $descriptor_abi_version:expr,
        runtime_entry: $runtime_entry:ident,
        runtime_entry_name: $runtime_entry_name:expr,
        editor_entry: $editor_entry:ident,
        editor_entry_name: $editor_entry_name:expr,
        requested_capabilities: $requested_capabilities:expr,
        missing_host_diagnostics: $missing_host_diagnostics:expr,
        runtime: {
            required_capabilities: [$($runtime_required_capability:literal),* $(,)?],
            denied_capabilities: [$($runtime_denied_capability:literal),* $(,)?],
            negotiated_capabilities: $runtime_negotiated_capabilities:expr,
            diagnostics: $runtime_diagnostics:expr,
            is_stateless: $runtime_is_stateless:expr,
            state_schema_version: $runtime_state_schema_version:expr,
            command_manifest_schema: $runtime_command_manifest_schema:ident $(($runtime_command_manifest_schema_value:expr))?,
            event_manifest_schema: $runtime_event_manifest_schema:ident $(($runtime_event_manifest_schema_value:expr))?,
            registration_manifest_schema: $runtime_registration_manifest_schema:ident $(($runtime_registration_manifest_schema_value:expr))?,
            command_manifest: $runtime_command_manifest:ident $(($runtime_command_manifest_value:expr))?,
            event_manifest: $runtime_event_manifest:ident $(($runtime_event_manifest_value:expr))?,
            registration_manifest: $runtime_registration_manifest:ident $(($runtime_registration_manifest_value:expr))?,
            invoke_command: $runtime_invoke_command:expr,
            save_state: $runtime_save_state:expr,
            restore_state: $runtime_restore_state:expr,
            unload: $runtime_unload:expr,
            bridge_methods: [
                $(
                    {
                        interface: $runtime_bridge_interface:expr,
                        method: $runtime_bridge_method_name:expr,
                        function: $runtime_bridge_method_function:path,
                        user_data: $runtime_bridge_method_user_data:expr $(,)?
                    }
                ),* $(,)?
            ],
            on_host_ready: $runtime_on_host_ready:expr $(,)?
        },
        editor: {
            required_capabilities: [$($editor_required_capability:literal),* $(,)?],
            denied_capabilities: [$($editor_denied_capability:literal),* $(,)?],
            negotiated_capabilities: $editor_negotiated_capabilities:expr,
            diagnostics: $editor_diagnostics:expr,
            is_stateless: $editor_is_stateless:expr,
            state_schema_version: $editor_state_schema_version:expr,
            command_manifest_schema: $editor_command_manifest_schema:ident $(($editor_command_manifest_schema_value:expr))?,
            event_manifest_schema: $editor_event_manifest_schema:ident $(($editor_event_manifest_schema_value:expr))?,
            registration_manifest_schema: $editor_registration_manifest_schema:ident $(($editor_registration_manifest_schema_value:expr))?,
            command_manifest: $editor_command_manifest:ident $(($editor_command_manifest_value:expr))?,
            event_manifest: $editor_event_manifest:ident $(($editor_event_manifest_value:expr))?,
            registration_manifest: $editor_registration_manifest:ident $(($editor_registration_manifest_value:expr))?,
            invoke_command: $editor_invoke_command:expr,
            save_state: $editor_save_state:expr,
            restore_state: $editor_restore_state:expr,
            unload: $editor_unload:expr,
            bridge_methods: [
                $(
                    {
                        interface: $editor_bridge_interface:expr,
                        method: $editor_bridge_method_name:expr,
                        function: $editor_bridge_method_function:path,
                        user_data: $editor_bridge_method_user_data:expr $(,)?
                    }
                ),* $(,)?
            ],
            on_host_ready: $editor_on_host_ready:expr $(,)?
        } $(,)?
    ) => {
        const __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3: usize =
            <[()]>::len(&[$({
                let _ = $runtime_bridge_method_name;
                ()
            }),*]);
        static __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHODS_V3: $crate::native::NativePluginStatic<
            [$crate::native::NativePluginBridgeMethodV3;
                __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3],
        > = $crate::native::NativePluginStatic::new([
            $(
                $crate::native::NativePluginBridgeMethodV3 {
                    interface_id: ($runtime_bridge_interface).as_ptr().cast(),
                    method_name: ($runtime_bridge_method_name).as_ptr().cast(),
                    method: Some($runtime_bridge_method_function),
                    user_data: $runtime_bridge_method_user_data,
                }
            ),*
        ]);
        static __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_TABLE_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBridgeMethodTableV3,
        > = $crate::native::NativePluginStatic::new(
            $crate::native::NativePluginBridgeMethodTableV3 {
                abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                methods: if __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3 == 0 {
                    ::core::ptr::null()
                } else {
                    __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHODS_V3.as_ptr().cast()
                },
                method_count: __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3,
            },
        );

        const __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3: usize =
            <[()]>::len(&[$({
                let _ = $editor_bridge_method_name;
                ()
            }),*]);
        static __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHODS_V3: $crate::native::NativePluginStatic<
            [$crate::native::NativePluginBridgeMethodV3;
                __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3],
        > = $crate::native::NativePluginStatic::new([
            $(
                $crate::native::NativePluginBridgeMethodV3 {
                    interface_id: ($editor_bridge_interface).as_ptr().cast(),
                    method_name: ($editor_bridge_method_name).as_ptr().cast(),
                    method: Some($editor_bridge_method_function),
                    user_data: $editor_bridge_method_user_data,
                }
            ),*
        ]);
        static __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_TABLE_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBridgeMethodTableV3,
        > = $crate::native::NativePluginStatic::new(
            $crate::native::NativePluginBridgeMethodTableV3 {
                abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                methods: if __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3 == 0 {
                    ::core::ptr::null()
                } else {
                    __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHODS_V3.as_ptr().cast()
                },
                method_count: __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3,
            },
        );

        static __ZIRCON_NATIVE_DIST_DESCRIPTOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginAbiV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginAbiV3 {
            abi_version: $descriptor_abi_version,
            plugin_id: ($plugin_id).as_ptr().cast(),
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            runtime_entry_name: ($runtime_entry_name).as_ptr().cast(),
            editor_entry_name: ($editor_entry_name).as_ptr().cast(),
            requested_capabilities: ($requested_capabilities).as_ptr().cast(),
        });

        static __ZIRCON_NATIVE_DIST_RUNTIME_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: if $runtime_is_stateless { 1 } else { 0 },
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: $runtime_state_schema_version,
                command_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_command_manifest_schema
                    $(($runtime_command_manifest_schema_value))?
                ),
                event_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_event_manifest_schema
                    $(($runtime_event_manifest_schema_value))?
                ),
                registration_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_registration_manifest_schema
                    $(($runtime_registration_manifest_schema_value))?
                ),
            },
            command_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_command_manifest
                $(($runtime_command_manifest_value))?
            ),
            event_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_event_manifest
                $(($runtime_event_manifest_value))?
            ),
            registration_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_registration_manifest
                $(($runtime_registration_manifest_value))?
            ),
            invoke_command: $runtime_invoke_command,
            save_state: $runtime_save_state,
            restore_state: $runtime_restore_state,
            unload: $runtime_unload,
        });

        static __ZIRCON_NATIVE_DIST_EDITOR_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: if $editor_is_stateless { 1 } else { 0 },
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: $editor_state_schema_version,
                command_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_command_manifest_schema
                    $(($editor_command_manifest_schema_value))?
                ),
                event_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_event_manifest_schema
                    $(($editor_event_manifest_schema_value))?
                ),
                registration_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_registration_manifest_schema
                    $(($editor_registration_manifest_schema_value))?
                ),
            },
            command_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_command_manifest
                $(($editor_command_manifest_value))?
            ),
            event_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_event_manifest
                $(($editor_event_manifest_value))?
            ),
            registration_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_registration_manifest
                $(($editor_registration_manifest_value))?
            ),
            invoke_command: $editor_invoke_command,
            save_state: $editor_save_state,
            restore_state: $editor_restore_state,
            unload: $editor_unload,
        });

        const __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($runtime_required_capability, "\n",)* "\0");
        const __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3: &str =
            concat!($($runtime_denied_capability, "\n",)* "\0");
        const __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($editor_required_capability, "\n",)* "\0");
        const __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3: &str =
            concat!($($editor_denied_capability, "\n",)* "\0");

        static __ZIRCON_NATIVE_DIST_RUNTIME_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($runtime_diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($runtime_negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: __ZIRCON_NATIVE_DIST_RUNTIME_BEHAVIOR_V4.as_ptr(),
            bridge_methods: __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_TABLE_V3.as_ptr(),
        });

        static __ZIRCON_NATIVE_DIST_EDITOR_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($editor_diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($editor_negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: __ZIRCON_NATIVE_DIST_EDITOR_BEHAVIOR_V4.as_ptr(),
            bridge_methods: __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_TABLE_V3.as_ptr(),
        });

        static __ZIRCON_NATIVE_DIST_RUNTIME_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_DIST_EDITOR_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_DIST_RUNTIME_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_DIST_RUNTIME_REPORT_V3,
                &__ZIRCON_NATIVE_DIST_RUNTIME_MISSING_HOST_REPORT_V3,
                &[$($runtime_required_capability),*],
                &[$($runtime_denied_capability),*],
                $runtime_on_host_ready,
            );
        static __ZIRCON_NATIVE_DIST_EDITOR_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_DIST_EDITOR_REPORT_V3,
                &__ZIRCON_NATIVE_DIST_EDITOR_MISSING_HOST_REPORT_V3,
                &[$($editor_required_capability),*],
                &[$($editor_denied_capability),*],
                $editor_on_host_ready,
            );

        $crate::export_native_plugin_descriptor_v3!(__ZIRCON_NATIVE_DIST_DESCRIPTOR_V3);
        $crate::export_native_plugin_entry_v3!(
            $runtime_entry,
            __ZIRCON_NATIVE_DIST_RUNTIME_ENTRY_POINT_V3
        );
        $crate::export_native_plugin_entry_v3!(
            $editor_entry,
            __ZIRCON_NATIVE_DIST_EDITOR_ENTRY_POINT_V3
        );
    };
}

/// 只生成 Runtime 入口，并在描述符中将 Editor 入口置空。
/// 行为表来自调用方提供的静态数据；是否满足有状态回调契约由宿主加载器判定。
#[macro_export]
macro_rules! native_dist_runtime_plugin_v3 {
    (
        plugin_id: $plugin_id:expr,
        package_manifest: $package_manifest:expr,
        descriptor_abi_version: $descriptor_abi_version:expr,
        runtime_entry: $runtime_entry:ident,
        runtime_entry_name: $runtime_entry_name:expr,
        requested_capabilities: $requested_capabilities:expr,
        missing_host_diagnostics: $missing_host_diagnostics:expr,
        runtime: {
            required_capabilities: [$($runtime_required_capability:literal),* $(,)?],
            denied_capabilities: [$($runtime_denied_capability:literal),* $(,)?],
            negotiated_capabilities: $runtime_negotiated_capabilities:expr,
            diagnostics: $runtime_diagnostics:expr,
            is_stateless: $runtime_is_stateless:expr,
            state_schema_version: $runtime_state_schema_version:expr,
            command_manifest_schema: $runtime_command_manifest_schema:ident $(($runtime_command_manifest_schema_value:expr))?,
            event_manifest_schema: $runtime_event_manifest_schema:ident $(($runtime_event_manifest_schema_value:expr))?,
            registration_manifest_schema: $runtime_registration_manifest_schema:ident $(($runtime_registration_manifest_schema_value:expr))?,
            command_manifest: $runtime_command_manifest:ident $(($runtime_command_manifest_value:expr))?,
            event_manifest: $runtime_event_manifest:ident $(($runtime_event_manifest_value:expr))?,
            registration_manifest: $runtime_registration_manifest:ident $(($runtime_registration_manifest_value:expr))?,
            invoke_command: $runtime_invoke_command:expr,
            save_state: $runtime_save_state:expr,
            restore_state: $runtime_restore_state:expr,
            unload: $runtime_unload:expr,
            bridge_methods: [
                $(
                    {
                        interface: $runtime_bridge_interface:expr,
                        method: $runtime_bridge_method_name:expr,
                        function: $runtime_bridge_method_function:path,
                        user_data: $runtime_bridge_method_user_data:expr $(,)?
                    }
                ),* $(,)?
            ],
            on_host_ready: $runtime_on_host_ready:expr $(,)?
        } $(,)?
    ) => {
        const __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3: usize =
            <[()]>::len(&[$({
                let _ = $runtime_bridge_method_name;
                ()
            }),*]);
        static __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHODS_V3: $crate::native::NativePluginStatic<
            [$crate::native::NativePluginBridgeMethodV3;
                __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3],
        > = $crate::native::NativePluginStatic::new([
            $(
                $crate::native::NativePluginBridgeMethodV3 {
                    interface_id: ($runtime_bridge_interface).as_ptr().cast(),
                    method_name: ($runtime_bridge_method_name).as_ptr().cast(),
                    method: Some($runtime_bridge_method_function),
                    user_data: $runtime_bridge_method_user_data,
                }
            ),*
        ]);
        static __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_TABLE_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBridgeMethodTableV3,
        > = $crate::native::NativePluginStatic::new(
            $crate::native::NativePluginBridgeMethodTableV3 {
                abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                methods: if __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3 == 0 {
                    ::core::ptr::null()
                } else {
                    __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHODS_V3.as_ptr().cast()
                },
                method_count: __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_COUNT_V3,
            },
        );

        static __ZIRCON_NATIVE_DIST_DESCRIPTOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginAbiV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginAbiV3 {
            abi_version: $descriptor_abi_version,
            plugin_id: ($plugin_id).as_ptr().cast(),
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            runtime_entry_name: ($runtime_entry_name).as_ptr().cast(),
            editor_entry_name: ::core::ptr::null(),
            requested_capabilities: ($requested_capabilities).as_ptr().cast(),
        });

        static __ZIRCON_NATIVE_DIST_RUNTIME_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: if $runtime_is_stateless { 1 } else { 0 },
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: $runtime_state_schema_version,
                command_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_command_manifest_schema
                    $(($runtime_command_manifest_schema_value))?
                ),
                event_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_event_manifest_schema
                    $(($runtime_event_manifest_schema_value))?
                ),
                registration_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $runtime_registration_manifest_schema
                    $(($runtime_registration_manifest_schema_value))?
                ),
            },
            command_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_command_manifest
                $(($runtime_command_manifest_value))?
            ),
            event_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_event_manifest
                $(($runtime_event_manifest_value))?
            ),
            registration_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $runtime_registration_manifest
                $(($runtime_registration_manifest_value))?
            ),
            invoke_command: $runtime_invoke_command,
            save_state: $runtime_save_state,
            restore_state: $runtime_restore_state,
            unload: $runtime_unload,
        });

        const __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($runtime_required_capability, "\n",)* "\0");
        const __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3: &str =
            concat!($($runtime_denied_capability, "\n",)* "\0");

        static __ZIRCON_NATIVE_DIST_RUNTIME_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($runtime_diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($runtime_negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: __ZIRCON_NATIVE_DIST_RUNTIME_BEHAVIOR_V4.as_ptr(),
            bridge_methods: __ZIRCON_NATIVE_DIST_RUNTIME_BRIDGE_METHOD_TABLE_V3.as_ptr(),
        });

        static __ZIRCON_NATIVE_DIST_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_RUNTIME_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_DIST_RUNTIME_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_DIST_RUNTIME_REPORT_V3,
                &__ZIRCON_NATIVE_DIST_MISSING_HOST_REPORT_V3,
                &[$($runtime_required_capability),*],
                &[$($runtime_denied_capability),*],
                $runtime_on_host_ready,
            );

        $crate::export_native_plugin_descriptor_v3!(__ZIRCON_NATIVE_DIST_DESCRIPTOR_V3);
        $crate::export_native_plugin_entry_v3!(
            $runtime_entry,
            __ZIRCON_NATIVE_DIST_RUNTIME_ENTRY_POINT_V3
        );
    };
}

/// 只生成 Editor 入口，并在描述符中将 Runtime 入口置空。
/// 宿主按 Editor 模块身份选择入口，再验证报告和回调契约。
#[macro_export]
macro_rules! native_dist_editor_plugin_v3 {
    (
        plugin_id: $plugin_id:expr,
        package_manifest: $package_manifest:expr,
        descriptor_abi_version: $descriptor_abi_version:expr,
        editor_entry: $editor_entry:ident,
        editor_entry_name: $editor_entry_name:expr,
        requested_capabilities: $requested_capabilities:expr,
        missing_host_diagnostics: $missing_host_diagnostics:expr,
        editor: {
            required_capabilities: [$($editor_required_capability:literal),* $(,)?],
            denied_capabilities: [$($editor_denied_capability:literal),* $(,)?],
            negotiated_capabilities: $editor_negotiated_capabilities:expr,
            diagnostics: $editor_diagnostics:expr,
            is_stateless: $editor_is_stateless:expr,
            state_schema_version: $editor_state_schema_version:expr,
            command_manifest_schema: $editor_command_manifest_schema:ident $(($editor_command_manifest_schema_value:expr))?,
            event_manifest_schema: $editor_event_manifest_schema:ident $(($editor_event_manifest_schema_value:expr))?,
            registration_manifest_schema: $editor_registration_manifest_schema:ident $(($editor_registration_manifest_schema_value:expr))?,
            command_manifest: $editor_command_manifest:ident $(($editor_command_manifest_value:expr))?,
            event_manifest: $editor_event_manifest:ident $(($editor_event_manifest_value:expr))?,
            registration_manifest: $editor_registration_manifest:ident $(($editor_registration_manifest_value:expr))?,
            invoke_command: $editor_invoke_command:expr,
            save_state: $editor_save_state:expr,
            restore_state: $editor_restore_state:expr,
            unload: $editor_unload:expr,
            bridge_methods: [
                $(
                    {
                        interface: $editor_bridge_interface:expr,
                        method: $editor_bridge_method_name:expr,
                        function: $editor_bridge_method_function:path,
                        user_data: $editor_bridge_method_user_data:expr $(,)?
                    }
                ),* $(,)?
            ],
            on_host_ready: $editor_on_host_ready:expr $(,)?
        } $(,)?
    ) => {
        const __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3: usize =
            <[()]>::len(&[$({
                let _ = $editor_bridge_method_name;
                ()
            }),*]);
        static __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHODS_V3: $crate::native::NativePluginStatic<
            [$crate::native::NativePluginBridgeMethodV3;
                __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3],
        > = $crate::native::NativePluginStatic::new([
            $(
                $crate::native::NativePluginBridgeMethodV3 {
                    interface_id: ($editor_bridge_interface).as_ptr().cast(),
                    method_name: ($editor_bridge_method_name).as_ptr().cast(),
                    method: Some($editor_bridge_method_function),
                    user_data: $editor_bridge_method_user_data,
                }
            ),*
        ]);
        static __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_TABLE_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBridgeMethodTableV3,
        > = $crate::native::NativePluginStatic::new(
            $crate::native::NativePluginBridgeMethodTableV3 {
                abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                methods: if __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3 == 0 {
                    ::core::ptr::null()
                } else {
                    __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHODS_V3.as_ptr().cast()
                },
                method_count: __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_COUNT_V3,
            },
        );

        static __ZIRCON_NATIVE_DIST_DESCRIPTOR_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginAbiV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginAbiV3 {
            abi_version: $descriptor_abi_version,
            plugin_id: ($plugin_id).as_ptr().cast(),
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            runtime_entry_name: ::core::ptr::null(),
            editor_entry_name: ($editor_entry_name).as_ptr().cast(),
            requested_capabilities: ($requested_capabilities).as_ptr().cast(),
        });

        static __ZIRCON_NATIVE_DIST_EDITOR_BEHAVIOR_V4: $crate::native::NativePluginStatic<
            $crate::native::NativePluginBehaviorV4,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginBehaviorV4 {
            abi_version: $crate::native::ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
            is_stateless: if $editor_is_stateless { 1 } else { 0 },
            schema_versions: $crate::native::NativePluginSchemaVersionsV3 {
                state_schema_version: $editor_state_schema_version,
                command_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_command_manifest_schema
                    $(($editor_command_manifest_schema_value))?
                ),
                event_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_event_manifest_schema
                    $(($editor_event_manifest_schema_value))?
                ),
                registration_manifest_schema: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                    $editor_registration_manifest_schema
                    $(($editor_registration_manifest_schema_value))?
                ),
            },
            command_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_command_manifest
                $(($editor_command_manifest_value))?
            ),
            event_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_event_manifest
                $(($editor_event_manifest_value))?
            ),
            registration_manifest: $crate::__zircon_native_dist_optional_cstr_ptr_v3!(
                $editor_registration_manifest
                $(($editor_registration_manifest_value))?
            ),
            invoke_command: $editor_invoke_command,
            save_state: $editor_save_state,
            restore_state: $editor_restore_state,
            unload: $editor_unload,
        });

        const __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3: &str =
            concat!($($editor_required_capability, "\n",)* "\0");
        const __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3: &str =
            concat!($($editor_denied_capability, "\n",)* "\0");

        static __ZIRCON_NATIVE_DIST_EDITOR_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($editor_diagnostics).as_ptr().cast(),
            negotiated_capabilities: ($editor_negotiated_capabilities).as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: __ZIRCON_NATIVE_DIST_EDITOR_BEHAVIOR_V4.as_ptr(),
            bridge_methods: __ZIRCON_NATIVE_DIST_EDITOR_BRIDGE_METHOD_TABLE_V3.as_ptr(),
        });

        static __ZIRCON_NATIVE_DIST_MISSING_HOST_REPORT_V3: $crate::native::NativePluginStatic<
            $crate::native::NativePluginEntryReportV3,
        > = $crate::native::NativePluginStatic::new($crate::native::NativePluginEntryReportV3 {
            layout_epoch: $crate::native::ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
            package_manifest_toml: ($package_manifest).as_bytes().as_ptr().cast(),
            diagnostics: ($missing_host_diagnostics).as_ptr().cast(),
            negotiated_capabilities: $crate::native::NATIVE_EMPTY_CSTR.as_ptr().cast(),
            required_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_REQUIRED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            denied_capabilities: __ZIRCON_NATIVE_DIST_EDITOR_DENIED_CAPABILITIES_TEXT_V3
                .as_ptr()
                .cast(),
            behavior: ::core::ptr::null(),
            bridge_methods: ::core::ptr::null(),
        });

        static __ZIRCON_NATIVE_DIST_EDITOR_ENTRY_POINT_V3: $crate::native::NativePluginEntryPointV3 =
            $crate::native::NativePluginEntryPointV3::new(
                &__ZIRCON_NATIVE_DIST_EDITOR_REPORT_V3,
                &__ZIRCON_NATIVE_DIST_MISSING_HOST_REPORT_V3,
                &[$($editor_required_capability),*],
                &[$($editor_denied_capability),*],
                $editor_on_host_ready,
            );

        $crate::export_native_plugin_descriptor_v3!(__ZIRCON_NATIVE_DIST_DESCRIPTOR_V3);
        $crate::export_native_plugin_entry_v3!(
            $editor_entry,
            __ZIRCON_NATIVE_DIST_EDITOR_ENTRY_POINT_V3
        );
    };
}

pub use crate::{
    native_dist_editor_plugin_v3, native_dist_plugin_v3, native_dist_runtime_plugin_v3,
};

#[cfg(test)]
#[path = "tests/dist.rs"]
mod tests;
