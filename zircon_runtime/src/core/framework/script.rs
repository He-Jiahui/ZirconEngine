//! Script-facing framework contracts shared by VM backends and host exports.

#[path = "script/argument_views.rs"]
mod argument_views;
mod behavior_bridge;
#[path = "script/call_frame.rs"]
mod call_frame;
#[path = "script/descriptors.rs"]
mod descriptors;
#[path = "script/hot_path_metrics.rs"]
mod hot_path_metrics;
#[path = "script/value_contracts.rs"]
mod value_contracts;

pub(crate) use argument_views::ScriptHostOwnedArgumentSource;
pub use argument_views::{
    ScriptHostArgumentSource, ScriptHostArguments, ScriptHostByteSource, ScriptHostByteView,
    ScriptHostFromArgument, ScriptHostValueRef,
};
pub use behavior_bridge::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID,
};
pub use call_frame::ScriptHostCallFrame;
pub use descriptors::{
    ScriptHostFieldDescriptor, ScriptHostFieldProjection, ScriptHostFunctionDescriptor,
    ScriptHostModuleDescriptor, ScriptHostParameterDescriptor, ScriptHostTypeDescriptor,
    ScriptHostTypeProjection, ZirconScriptType,
};
pub use hot_path_metrics::{ScriptHostHotPathMetrics, ScriptHostHotPathMetricsSnapshot};
pub use value_contracts::{
    ScriptHostError, ScriptHostHandleValue, ScriptHostIntoValue, ScriptHostPrototypeKind,
    ScriptHostResult, ScriptHostTypeRef, ScriptHostValue, ScriptHostValueKind,
};

#[doc(hidden)]
pub mod __reflect {
    pub use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectScriptVisibility,
        ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypeKind, ReflectTypePath,
        ReflectTypeRegistration,
    };
}

#[cfg(test)]
mod tests {
    use super::{ScriptHostIntoValue, ScriptHostTypeRef, ScriptHostValue, ScriptHostValueKind};

    #[test]
    fn bytes_default_to_the_zr_vm_byte_array_type() {
        assert_eq!(
            ScriptHostValueKind::Bytes.default_zr_type_name(),
            "container.Array<uint>"
        );
        assert_eq!(
            ScriptHostTypeRef::from_value_kind(ScriptHostValueKind::Bytes).type_name,
            "container.Array<uint>"
        );
    }

    #[test]
    fn byte_vectors_encode_through_the_owned_return_value_contract() {
        let bytes = vec![0, 104, 128, 255];
        let host_value = bytes.clone().into_script_host_value();

        assert_eq!(host_value, ScriptHostValue::Bytes(bytes));
    }
}
