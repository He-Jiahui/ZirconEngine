use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::core::{CoreError, CoreResult, ModuleDescriptor};
use crate::engine_module::EngineModule;
use crate::plugin::RuntimeProfileDescriptor;

use super::super::ids::BuiltinRuntimeModuleId;

struct BuiltinModuleCandidateRegistry {
    candidates: Vec<(BuiltinRuntimeModuleId, Arc<dyn EngineModule>)>,
    index_by_id: HashMap<BuiltinRuntimeModuleId, usize>,
}

pub(in crate::builtin::runtime_modules) struct SelectedBuiltinRuntimeModules {
    pub(in crate::builtin::runtime_modules) modules: Vec<Arc<dyn EngineModule>>,
    pub(in crate::builtin::runtime_modules) descriptors_by_name: HashMap<String, ModuleDescriptor>,
}

impl BuiltinModuleCandidateRegistry {
    fn from_modules(modules: Vec<Arc<dyn EngineModule>>) -> CoreResult<Self> {
        let mut registry = Self::with_capacity(modules.len());
        for module in modules {
            let id = BuiltinRuntimeModuleId::for_module_name(module.module_name())
                .ok_or_else(|| CoreError::MissingModule(module.module_name().to_owned()))?;
            let index = registry.candidates.len();
            if registry.index_by_id.insert(id, index).is_some() {
                return Err(CoreError::DuplicateModule(module.module_name().to_owned()));
            }
            registry.candidates.push((id, module));
        }
        Ok(registry)
    }

    fn with_capacity(candidate_count: usize) -> Self {
        Self {
            candidates: Vec::with_capacity(candidate_count),
            index_by_id: HashMap::with_capacity(candidate_count),
        }
    }

    fn select(
        self,
        profile: &RuntimeProfileDescriptor,
    ) -> CoreResult<SelectedBuiltinRuntimeModules> {
        let mut selected = profile
            .builtin_modules
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let mut pending = profile
            .builtin_modules
            .iter()
            .copied()
            .collect::<VecDeque<_>>();
        let mut descriptors_by_id = HashMap::new();

        // Complete the dependency closure from the same target-owned candidate registry.
        while let Some(id) = pending.pop_front() {
            let Some(&index) = self.index_by_id.get(&id) else {
                return Err(CoreError::MissingModule(id.module_name().to_owned()));
            };
            let module = &self.candidates[index].1;
            let descriptor = module.descriptor();
            for dependency in &descriptor.module_dependencies {
                let Some(dependency_id) =
                    BuiltinRuntimeModuleId::for_module_name(&dependency.module_name)
                else {
                    continue;
                };
                if !self.index_by_id.contains_key(&dependency_id) {
                    return Err(CoreError::MissingModuleDependency {
                        module: module.module_name().to_owned(),
                        dependency: dependency.module_name.clone(),
                    });
                }
                if selected.insert(dependency_id) {
                    pending.push_back(dependency_id);
                }
            }
            descriptors_by_id.insert(id, descriptor);
        }

        let mut modules = Vec::with_capacity(selected.len());
        let mut descriptors_by_name = HashMap::with_capacity(selected.len());
        for (id, module) in self.candidates {
            if !selected.contains(&id) {
                continue;
            }
            let descriptor = descriptors_by_id
                .remove(&id)
                .ok_or_else(|| CoreError::MissingModule(id.module_name().to_owned()))?;
            descriptors_by_name.insert(module.module_name().to_owned(), descriptor);
            modules.push(module);
        }
        Ok(SelectedBuiltinRuntimeModules {
            modules,
            descriptors_by_name,
        })
    }
}

pub(in crate::builtin::runtime_modules) fn select_runtime_profile_builtin_module_descriptors(
    profile: &RuntimeProfileDescriptor,
    candidates: Vec<Arc<dyn EngineModule>>,
) -> CoreResult<SelectedBuiltinRuntimeModules> {
    BuiltinModuleCandidateRegistry::from_modules(candidates)?.select(profile)
}

#[cfg(test)]
#[path = "profile_selection/capacity_tests.rs"]
mod capacity_tests;
