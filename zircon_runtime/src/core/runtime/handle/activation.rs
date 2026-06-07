use std::collections::HashMap;

use crate::core::error::CoreError;
use crate::core::lifecycle::LifecycleState;

use super::super::descriptors::RegistryName;
use super::super::state::ServiceEntry;
use super::CoreHandle;

mod blocked_dependencies;

use self::blocked_dependencies::{
    dependency_slice_contains_service, first_blocked_five_service_dependency,
    first_blocked_four_service_dependency, first_blocked_three_service_dependency,
    first_blocked_two_service_dependency, FiveServiceDependencyMatch, FourServiceDependencyMatch,
    ThreeServiceDependencyMatch, TwoServiceDependencyMatch,
};

const BLOCKED_DEPENDENT_INITIAL_CAPACITY: usize = 1;

impl CoreHandle {
    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "activate_module");
        let startup_services = {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            if entry.lifecycle == LifecycleState::Running {
                return Ok(());
            }
            entry.lifecycle = LifecycleState::Initializing;
            if entry.startup_service_names.is_empty() {
                entry.lifecycle = LifecycleState::Running;
                return Ok(());
            }
            entry.startup_service_names.clone()
        };

        let result = (|| {
            self.resolve_startup_services(startup_services.as_ref())?;

            self.finish_module_activation(module_name)
        })();

        if result.is_err() {
            self.reset_initializing_module(module_name);
        }

        result
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "deactivate_module");
        let (previous_lifecycle, unload_order) = {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            let previous_lifecycle = entry.lifecycle;
            entry.lifecycle = LifecycleState::Stopping;
            if entry.shutdown_service_names.is_empty() {
                entry.lifecycle = LifecycleState::Unloaded;
                return Ok(());
            }
            (previous_lifecycle, entry.shutdown_service_names.clone())
        };
        let blocked_unload = {
            let mut services = self.inner.services.lock().unwrap();
            let unload_order = unload_order.as_ref();
            let blocked_unload = first_blocked_unload(&services, unload_order);
            if blocked_unload.is_none() {
                unload_services(&mut services, unload_order);
            }
            blocked_unload
        };
        if let Some((service_name, dependents)) = blocked_unload {
            self.reset_stopping_module(module_name, previous_lifecycle);
            return Err(CoreError::UnloadBlocked(service_name, dependents));
        }

        self.finish_module_deactivation(module_name)
    }

    fn resolve_startup_services(&self, startup_services: &[RegistryName]) -> Result<(), CoreError> {
        if let [service] = startup_services {
            self.resolve_registered_service(service, None)?;
            return Ok(());
        }
        if let [first_service, second_service] = startup_services {
            self.resolve_registered_service(first_service, None)?;
            self.resolve_registered_service(second_service, None)?;
            return Ok(());
        }
        if let [first_service, second_service, third_service] = startup_services {
            self.resolve_registered_service(first_service, None)?;
            self.resolve_registered_service(second_service, None)?;
            self.resolve_registered_service(third_service, None)?;
            return Ok(());
        }
        if let [first_service, second_service, third_service, fourth_service] = startup_services {
            self.resolve_registered_service(first_service, None)?;
            self.resolve_registered_service(second_service, None)?;
            self.resolve_registered_service(third_service, None)?;
            self.resolve_registered_service(fourth_service, None)?;
            return Ok(());
        }
        if let [first_service, second_service, third_service, fourth_service, fifth_service] =
            startup_services
        {
            self.resolve_registered_service(first_service, None)?;
            self.resolve_registered_service(second_service, None)?;
            self.resolve_registered_service(third_service, None)?;
            self.resolve_registered_service(fourth_service, None)?;
            self.resolve_registered_service(fifth_service, None)?;
            return Ok(());
        }

        for service in startup_services {
            self.resolve_registered_service(service, None)?;
        }

        Ok(())
    }

    fn reset_initializing_module(&self, module_name: &str) {
        let mut modules = self.inner.modules.lock().unwrap();
        if let Some(entry) = modules.get_mut(module_name) {
            if entry.lifecycle == LifecycleState::Initializing {
                entry.lifecycle = LifecycleState::Registered;
            }
        }
    }

    fn reset_stopping_module(&self, module_name: &str, previous_lifecycle: LifecycleState) {
        let mut modules = self.inner.modules.lock().unwrap();
        if let Some(entry) = modules.get_mut(module_name) {
            if entry.lifecycle == LifecycleState::Stopping {
                entry.lifecycle = previous_lifecycle;
            }
        }
    }

    fn finish_module_activation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Running;
        Ok(())
    }

    fn finish_module_deactivation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Unloaded;
        Ok(())
    }
}

fn unload_services(
    services: &mut HashMap<RegistryName, ServiceEntry>,
    unload_order: &[RegistryName],
) {
    if let [service_name] = unload_order {
        unload_service(services, service_name);
        return;
    }
    if let [first_service_name, second_service_name] = unload_order {
        unload_service(services, first_service_name);
        unload_service(services, second_service_name);
        return;
    }
    if let [first_service_name, second_service_name, third_service_name] = unload_order {
        unload_service(services, first_service_name);
        unload_service(services, second_service_name);
        unload_service(services, third_service_name);
        return;
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name] =
        unload_order
    {
        unload_service(services, first_service_name);
        unload_service(services, second_service_name);
        unload_service(services, third_service_name);
        unload_service(services, fourth_service_name);
        return;
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name, fifth_service_name] =
        unload_order
    {
        unload_service(services, first_service_name);
        unload_service(services, second_service_name);
        unload_service(services, third_service_name);
        unload_service(services, fourth_service_name);
        unload_service(services, fifth_service_name);
        return;
    }

    for service_name in unload_order {
        unload_service(services, service_name);
    }
}

fn unload_service(services: &mut HashMap<RegistryName, ServiceEntry>, service_name: &RegistryName) {
    if let Some(entry) = services.get_mut(service_name) {
        entry.instance = None;
        entry.lifecycle = LifecycleState::Unloaded;
    }
}

fn first_blocked_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    unload_order: &[RegistryName],
) -> Option<(String, Vec<String>)> {
    if let [service_name] = unload_order {
        return first_blocked_single_service_unload(services, service_name);
    }
    if let [first_service_name, second_service_name] = unload_order {
        return first_blocked_two_service_unload(services, first_service_name, second_service_name);
    }
    if let [first_service_name, second_service_name, third_service_name] = unload_order {
        return first_blocked_three_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
        );
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name] =
        unload_order
    {
        return first_blocked_four_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
        );
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name, fifth_service_name] =
        unload_order
    {
        return first_blocked_five_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
            fifth_service_name,
        );
    }

    let mut unload_indices: HashMap<&RegistryName, usize> =
        HashMap::with_capacity(unload_order.len());
    for (index, service_name) in unload_order.iter().enumerate() {
        unload_indices.insert(service_name, index);
    }
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if unload_indices.contains_key(dependent_name) || entry.instance.is_none() {
            continue;
        }

        for dependency in entry.dependencies.iter() {
            if let Some(index) = unload_indices.get(dependency).copied() {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    index,
                    dependent_name,
                );
            }
        }
    }

    match (blocked_index, blocked_dependents) {
        (Some(index), Some(dependents)) => Some((unload_order[index].to_string(), dependents)),
        _ => None,
    }
}

fn first_blocked_single_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == service_name || entry.instance.is_none() {
            continue;
        }

        if dependency_slice_contains_service(entry.dependencies.as_ref(), service_name) {
            blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                .push(dependent_name.as_str().to_owned());
        }
    }

    blocked_dependents.map(|dependents| (service_name.to_string(), dependents))
}

fn first_blocked_two_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut first_blocked_dependents: Option<Vec<String>> = None;
    let mut second_blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_two_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
        ) {
            Some(TwoServiceDependencyMatch::FirstService) => {
                first_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(TwoServiceDependencyMatch::SecondService) => {
                second_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            None => {}
        }
    }

    let blocked_service_name = if first_blocked_dependents.is_some() {
        first_service_name
    } else {
        second_service_name
    };
    let blocked_dependents = first_blocked_dependents.or(second_blocked_dependents)?;
    Some((blocked_service_name.to_string(), blocked_dependents))
}

fn first_blocked_three_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut first_blocked_dependents: Option<Vec<String>> = None;
    let mut second_blocked_dependents: Option<Vec<String>> = None;
    let mut third_blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_three_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
        ) {
            Some(ThreeServiceDependencyMatch::FirstService) => {
                first_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(ThreeServiceDependencyMatch::SecondService) => {
                second_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(ThreeServiceDependencyMatch::ThirdService) => {
                third_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            None => {}
        }
    }

    if let Some(dependents) = first_blocked_dependents {
        return Some((first_service_name.to_string(), dependents));
    }
    if let Some(dependents) = second_blocked_dependents {
        return Some((second_service_name.to_string(), dependents));
    }
    third_blocked_dependents.map(|dependents| (third_service_name.to_string(), dependents))
}

fn first_blocked_four_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
    fourth_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut first_blocked_dependents: Option<Vec<String>> = None;
    let mut second_blocked_dependents: Option<Vec<String>> = None;
    let mut third_blocked_dependents: Option<Vec<String>> = None;
    let mut fourth_blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || dependent_name == fourth_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_four_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
        ) {
            Some(FourServiceDependencyMatch::FirstService) => {
                first_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FourServiceDependencyMatch::SecondService) => {
                second_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FourServiceDependencyMatch::ThirdService) => {
                third_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FourServiceDependencyMatch::FourthService) => {
                fourth_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            None => {}
        }
    }

    if let Some(dependents) = first_blocked_dependents {
        return Some((first_service_name.to_string(), dependents));
    }
    if let Some(dependents) = second_blocked_dependents {
        return Some((second_service_name.to_string(), dependents));
    }
    if let Some(dependents) = third_blocked_dependents {
        return Some((third_service_name.to_string(), dependents));
    }
    fourth_blocked_dependents.map(|dependents| (fourth_service_name.to_string(), dependents))
}

fn first_blocked_five_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
    fourth_service_name: &RegistryName,
    fifth_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut first_blocked_dependents: Option<Vec<String>> = None;
    let mut second_blocked_dependents: Option<Vec<String>> = None;
    let mut third_blocked_dependents: Option<Vec<String>> = None;
    let mut fourth_blocked_dependents: Option<Vec<String>> = None;
    let mut fifth_blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || dependent_name == fourth_service_name
            || dependent_name == fifth_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_five_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
            fifth_service_name,
        ) {
            Some(FiveServiceDependencyMatch::FirstService) => {
                first_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FiveServiceDependencyMatch::SecondService) => {
                second_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FiveServiceDependencyMatch::ThirdService) => {
                third_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FiveServiceDependencyMatch::FourthService) => {
                fourth_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            Some(FiveServiceDependencyMatch::FifthService) => {
                fifth_blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                    .push(dependent_name.as_str().to_owned());
            }
            None => {}
        }
    }

    if let Some(dependents) = first_blocked_dependents {
        return Some((first_service_name.to_string(), dependents));
    }
    if let Some(dependents) = second_blocked_dependents {
        return Some((second_service_name.to_string(), dependents));
    }
    if let Some(dependents) = third_blocked_dependents {
        return Some((third_service_name.to_string(), dependents));
    }
    if let Some(dependents) = fourth_blocked_dependents {
        return Some((fourth_service_name.to_string(), dependents));
    }
    fifth_blocked_dependents.map(|dependents| (fifth_service_name.to_string(), dependents))
}

fn record_blocked_dependent(
    blocked_index: &mut Option<usize>,
    blocked_dependents: &mut Option<Vec<String>>,
    index: usize,
    dependent_name: &RegistryName,
) {
    match *blocked_index {
        Some(current_index) if index > current_index => {}
        Some(current_index) if index == current_index => {
            blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                .push(dependent_name.as_str().to_owned());
        }
        _ => {
            *blocked_index = Some(index);
            let dependents = blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY));
            dependents.clear();
            dependents.push(dependent_name.as_str().to_owned());
        }
    }
}
