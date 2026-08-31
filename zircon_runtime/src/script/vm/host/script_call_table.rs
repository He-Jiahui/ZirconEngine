use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::core::framework::script::{
    ScriptHostArguments, ScriptHostCallFrame, ScriptHostFunctionDescriptor,
    ScriptHostHotPathMetrics, ScriptHostValue,
};

use super::super::runtime_context::with_active_script_runtime_call_context;
use super::super::{CapabilitySet, VmError};
use super::host_export_registry::HostExportCallback;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptCallSiteId(u32);

impl ScriptCallSiteId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone)]
pub struct ScriptCallSite {
    id: ScriptCallSiteId,
    module_name: Arc<str>,
    function_name: Arc<str>,
    descriptor: ScriptHostFunctionDescriptor,
    callback: HostExportCallback,
}

impl ScriptCallSite {
    pub(crate) fn new(
        id: ScriptCallSiteId,
        module_name: Arc<str>,
        descriptor: ScriptHostFunctionDescriptor,
        callback: HostExportCallback,
    ) -> Self {
        let function_name = Arc::<str>::from(descriptor.name.as_str());
        Self {
            id,
            module_name,
            function_name,
            descriptor,
            callback,
        }
    }

    pub const fn id(&self) -> ScriptCallSiteId {
        self.id
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub const fn descriptor(&self) -> &ScriptHostFunctionDescriptor {
        &self.descriptor
    }

    pub fn call<'call>(
        &self,
        arguments: ScriptHostArguments<'call>,
        granted_capabilities: &CapabilitySet,
    ) -> Result<ScriptHostValue, VmError> {
        validate_call_arity(self, arguments.len())?;
        validate_call_capabilities(self, granted_capabilities)?;

        ScriptHostHotPathMetrics::record_host_call();
        let result = with_active_script_runtime_call_context(|runtime_context| {
            let frame = ScriptHostCallFrame::new(
                self.module_name(),
                self.function_name(),
                arguments,
                &granted_capabilities.capabilities,
                runtime_context.map(|context| context as &dyn std::any::Any),
            );
            (self.callback)(&frame)
        });
        result.map_err(|error| {
            VmError::Operation(format!(
                "host export call failed: {}.{}: {}",
                self.module_name(),
                self.function_name(),
                error.message
            ))
        })
    }
}

impl fmt::Debug for ScriptCallSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptCallSite")
            .field("id", &self.id)
            .field("module_name", &self.module_name)
            .field("function_name", &self.function_name)
            .field("descriptor", &self.descriptor)
            .field("callback", &"<host export callback>")
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScriptCallTable {
    generation: u64,
    entries: Arc<Vec<ScriptCallSite>>,
    by_name: Arc<HashMap<Arc<str>, HashMap<Arc<str>, ScriptCallSiteId>>>,
}

impl ScriptCallTable {
    pub(crate) fn from_entries(generation: u64, entries: Vec<ScriptCallSite>) -> Self {
        let module_count = entries
            .windows(2)
            .filter(|pair| pair[0].module_name != pair[1].module_name)
            .count()
            + usize::from(!entries.is_empty());
        let mut by_name = HashMap::with_capacity(module_count);
        let mut group_start = 0;
        while group_start < entries.len() {
            let mut group_end = group_start + 1;
            while group_end < entries.len()
                && entries[group_start].module_name == entries[group_end].module_name
            {
                group_end += 1;
            }

            let function_count = group_end - group_start;
            let functions = by_name
                .entry(Arc::clone(&entries[group_start].module_name))
                .or_insert_with(|| HashMap::with_capacity(function_count));
            for entry in &entries[group_start..group_end] {
                functions.insert(Arc::clone(&entry.function_name), entry.id());
            }
            group_start = group_end;
        }
        Self {
            generation,
            entries: Arc::new(entries),
            by_name: Arc::new(by_name),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, id: ScriptCallSiteId) -> Option<&ScriptCallSite> {
        self.entries.get(id.raw() as usize)
    }

    pub fn resolve(&self, module_name: &str, function_name: &str) -> Option<&ScriptCallSite> {
        let id = self.by_name.get(module_name)?.get(function_name)?;
        self.get(*id)
    }

    pub fn call<'call>(
        &self,
        id: ScriptCallSiteId,
        arguments: ScriptHostArguments<'call>,
        granted_capabilities: &CapabilitySet,
    ) -> Result<ScriptHostValue, VmError> {
        let Some(site) = self.get(id) else {
            return Err(VmError::Operation(format!(
                "script call site {} is not registered",
                id.raw()
            )));
        };
        site.call(arguments, granted_capabilities)
    }
}

pub(crate) struct ScriptCallTableBuilder {
    generation: u64,
    entries: Vec<ScriptCallSite>,
}

impl ScriptCallTableBuilder {
    pub(crate) fn new(generation: u64) -> Self {
        Self {
            generation,
            entries: Vec::new(),
        }
    }

    pub(crate) fn add(
        &mut self,
        module_name: Arc<str>,
        descriptor: ScriptHostFunctionDescriptor,
        callback: HostExportCallback,
    ) {
        let id = ScriptCallSiteId(self.entries.len() as u32);
        self.entries
            .push(ScriptCallSite::new(id, module_name, descriptor, callback));
    }

    pub(crate) fn build(self) -> ScriptCallTable {
        ScriptCallTable::from_entries(self.generation, self.entries)
    }
}

fn validate_call_arity(site: &ScriptCallSite, argument_count: usize) -> Result<(), VmError> {
    let descriptor = site.descriptor();
    if argument_count < descriptor.min_argument_count
        || argument_count > descriptor.max_argument_count
    {
        return Err(VmError::Operation(format!(
            "host export call {}.{} expected {}..={} arguments, received {argument_count}",
            site.module_name(),
            site.function_name(),
            descriptor.min_argument_count,
            descriptor.max_argument_count
        )));
    }
    Ok(())
}

fn validate_call_capabilities(
    site: &ScriptCallSite,
    granted_capabilities: &CapabilitySet,
) -> Result<(), VmError> {
    for capability in &site.descriptor().required_capabilities {
        if !granted_capabilities.capabilities.contains(capability) {
            return Err(VmError::Operation(format!(
                "host export call {}.{} missing capability {capability}",
                site.module_name(),
                site.function_name()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::script::ScriptHostValueKind;

    fn test_site(id: u32, module_name: &str, function_name: &str) -> ScriptCallSite {
        let callback: HostExportCallback = Arc::new(|_| Ok(ScriptHostValue::Null));
        ScriptCallSite::new(
            ScriptCallSiteId(id),
            Arc::from(module_name),
            ScriptHostFunctionDescriptor::new(function_name, 0, 0, ScriptHostValueKind::Null),
            callback,
        )
    }

    #[test]
    fn from_entries_preserves_non_contiguous_module_groups() {
        let table = ScriptCallTable::from_entries(
            7,
            vec![
                test_site(0, "runtime.time", "now"),
                test_site(1, "runtime.input", "poll"),
                test_site(2, "runtime.time", "delta"),
            ],
        );

        assert!(table.resolve("runtime.time", "now").is_some());
        assert!(table.resolve("runtime.input", "poll").is_some());
        assert!(table.resolve("runtime.time", "delta").is_some());
    }
}
