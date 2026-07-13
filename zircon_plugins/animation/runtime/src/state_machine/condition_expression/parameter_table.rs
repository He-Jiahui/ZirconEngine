use std::collections::BTreeMap;

use super::ConditionExpressionCompileError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::state_machine) struct ParameterSlot(u32);

impl ParameterSlot {
    pub(in crate::state_machine) fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Default)]
pub(in crate::state_machine) struct ParameterTableBuilder {
    names: Vec<String>,
    slots: BTreeMap<String, ParameterSlot>,
}

impl ParameterTableBuilder {
    pub(in crate::state_machine) fn intern(
        &mut self,
        name: &str,
    ) -> Result<ParameterSlot, ConditionExpressionCompileError> {
        if let Some(slot) = self.slots.get(name) {
            return Ok(*slot);
        }
        let slot = ParameterSlot(
            u32::try_from(self.names.len())
                .map_err(|_| ConditionExpressionCompileError::CapacityExceeded)?,
        );
        self.names.push(name.to_string());
        self.slots.insert(name.to_string(), slot);
        Ok(slot)
    }

    pub(in crate::state_machine) fn finish(self) -> Box<[String]> {
        self.names.into_boxed_slice()
    }
}
