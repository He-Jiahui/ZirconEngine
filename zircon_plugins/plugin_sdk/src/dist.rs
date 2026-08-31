//! Native distribution entry helpers.
//!
//! This module keeps one-file `cdylib` exports out of the low-level ABI owner
//! in `native.rs`. A native plugin crate should define its manifests and
//! callbacks, then invoke one of these macros once at crate root.

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
mod tests {
    use std::ffi::CStr;

    use zircon_runtime_interface::{ZrByteBufferRef, ZrByteSlice, ZrStatus, ZrStatusCode};

    use crate::native::{
        self, NativePluginBridgeMethodCallV3, NativePluginByteSliceV3,
        NativePluginCallbackStatusV3, NativePluginHostFunctionTableV3, NativePluginOutputSinkV4,
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    };

    const PLUGIN_ID: &[u8] = b"dist_helper_fixture\0";
    const PACKAGE_MANIFEST: &str = "id = \"dist_helper_fixture\"\nversion = \"0.1.0\"\n\0";
    const RUNTIME_ENTRY: &[u8] = b"dist_helper_runtime_entry_v3\0";
    const EDITOR_ENTRY: &[u8] = b"dist_helper_editor_entry_v3\0";
    const REQUESTED_CAPABILITIES: &[u8] =
        b"runtime.plugin.dist_helper\neditor.extension.dist_helper\0";
    const RUNTIME_NEGOTIATED: &[u8] = b"runtime.plugin.dist_helper\0";
    const EDITOR_NEGOTIATED: &[u8] = b"editor.extension.dist_helper\0";
    const READY: &[u8] = b"ready\0";
    const MISSING: &[u8] = b"missing\0";
    const EMPTY: &[u8] = b"\0";
    const EMPTY_COMMANDS_V4: &[u8] =
        b"schema = \"zircon.native.command-manifest/4\"\ncommands = []\n\0";
    const RUNTIME_INTERFACE: &[u8] = b"dist_helper.runtime\0";
    const RUNTIME_METHOD: &[u8] = b"tick\0";

    crate::native_dist_plugin_v3! {
        plugin_id: PLUGIN_ID,
        package_manifest: PACKAGE_MANIFEST,
        descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        runtime_entry: dist_helper_runtime_entry_v3,
        runtime_entry_name: RUNTIME_ENTRY,
        editor_entry: dist_helper_editor_entry_v3,
        editor_entry_name: EDITOR_ENTRY,
        requested_capabilities: REQUESTED_CAPABILITIES,
        missing_host_diagnostics: MISSING,
        runtime: {
            required_capabilities: ["runtime.plugin.dist_helper"],
            denied_capabilities: ["runtime.plugin.denied_dist_helper"],
            negotiated_capabilities: RUNTIME_NEGOTIATED,
            diagnostics: READY,
            is_stateless: false,
            state_schema_version: 3,
            command_manifest_schema: Some(native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4),
            event_manifest_schema: Some(native::NATIVE_EVENT_MANIFEST_SCHEMA_V3),
            registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
            command_manifest: Some(EMPTY_COMMANDS_V4),
            event_manifest: Some(EMPTY),
            registration_manifest: Some(EMPTY),
            invoke_command: Some(dist_helper_invoke_command),
            save_state: None,
            restore_state: None,
            unload: None,
            bridge_methods: [
                {
                    interface: RUNTIME_INTERFACE,
                    method: RUNTIME_METHOD,
                    function: dist_helper_tick,
                    user_data: 99,
                },
            ],
            on_host_ready: None,
        },
        editor: {
            required_capabilities: ["editor.extension.dist_helper"],
            denied_capabilities: [],
            negotiated_capabilities: EDITOR_NEGOTIATED,
            diagnostics: READY,
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: Some(EMPTY_COMMANDS_V4),
            event_manifest: Some(EMPTY),
            registration_manifest: None,
            invoke_command: Some(dist_helper_invoke_command),
            save_state: None,
            restore_state: None,
            unload: None,
            bridge_methods: [],
            on_host_ready: None,
        },
    }

    #[test]
    fn dist_plugin_one_file_export_compiles() {
        let descriptor = zircon_native_plugin_descriptor_v3();
        assert!(!descriptor.is_null());
        let descriptor = unsafe { &*descriptor };
        assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.plugin_id) },
            CStr::from_bytes_with_nul(PLUGIN_ID).expect("plugin id is nul terminated")
        );

        let granted = b"runtime.plugin.dist_helper\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 7,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        let report = dist_helper_runtime_entry_v3(&host);
        assert!(!report.is_null());
        let report = unsafe { &*report };
        assert!(!report.behavior.is_null());
        assert!(!report.bridge_methods.is_null());

        let behavior = unsafe { &*report.behavior };
        assert!(!behavior.registration_manifest.is_null());
        let bridge_methods = unsafe { &*report.bridge_methods };
        assert_eq!(bridge_methods.method_count, 1);
        assert!(!bridge_methods.methods.is_null());

        let bridge_method = unsafe { &*bridge_methods.methods };
        assert_eq!(bridge_method.user_data, 99);
        let call = NativePluginBridgeMethodCallV3 {
            interface_slot: 0,
            method_slot: 0,
            payload: ZrByteSlice::empty(),
            output: ZrByteBufferRef::empty(),
            user_data: bridge_method.user_data,
        };
        let status = unsafe { bridge_method.method.expect("bridge method is present")(call) };
        assert_eq!(status.status_code(), ZrStatusCode::Ok);
    }

    unsafe extern "C" fn dist_helper_invoke_command(
        _command_slot: u32,
        _payload: NativePluginByteSliceV3,
        _output: NativePluginOutputSinkV4,
    ) -> NativePluginCallbackStatusV3 {
        native::callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, MISSING)
    }

    unsafe extern "C" fn dist_helper_tick(_call: NativePluginBridgeMethodCallV3) -> ZrStatus {
        ZrStatus::ok()
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
