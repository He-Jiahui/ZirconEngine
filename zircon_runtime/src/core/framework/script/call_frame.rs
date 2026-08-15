use std::any::Any;

use super::ScriptHostArguments;

pub struct ScriptHostCallFrame<'call> {
    pub module_name: &'call str,
    pub function_name: &'call str,
    pub arguments: ScriptHostArguments<'call>,
    pub granted_capabilities: &'call [String],
    /// Runtime-owned data borrowed only for this synchronous host export call.
    runtime_context: Option<&'call dyn Any>,
}

impl<'call> ScriptHostCallFrame<'call> {
    pub(crate) fn new(
        module_name: &'call str,
        function_name: &'call str,
        arguments: ScriptHostArguments<'call>,
        granted_capabilities: &'call [String],
        runtime_context: Option<&'call dyn Any>,
    ) -> Self {
        Self {
            module_name,
            function_name,
            arguments,
            granted_capabilities,
            runtime_context,
        }
    }

    pub(crate) fn runtime_context<T: Any>(&self) -> Option<&T> {
        self.runtime_context
            .and_then(|context| context.downcast_ref())
    }
}

impl std::fmt::Debug for ScriptHostCallFrame<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptHostCallFrame")
            .field("module_name", &self.module_name)
            .field("function_name", &self.function_name)
            .field("argument_count", &self.arguments.len())
            .field("granted_capabilities", &self.granted_capabilities)
            .field(
                "runtime_context",
                &self.runtime_context.as_ref().map(|_| "<borrowed>"),
            )
            .finish()
    }
}
