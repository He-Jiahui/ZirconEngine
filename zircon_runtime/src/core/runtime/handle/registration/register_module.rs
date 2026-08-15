use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind};

use super::super::super::descriptors::ModuleDescriptor;
use super::super::super::state::{ModuleEntry, ServiceEntry, ServiceEntryFactory};
use super::super::CoreHandle;
use super::descriptor_entries::{
    prepare_service_entry, prepare_single_descriptor_service_entry,
    prepare_two_descriptor_service_entries,
};
use super::descriptor_entries_five::prepare_five_descriptor_service_entries;
use super::descriptor_entries_four::prepare_four_descriptor_service_entries;
use super::descriptor_entries_three::prepare_three_descriptor_service_entries;
use super::duplicates::duplicate_existing_pending_service_name;
use super::service_lists::{module_service_lists, single_service_module_lists};
use super::validation::is_canonical_module_name;

const FIRST_REGISTERED_SERVICE_INDEX: u32 = 1;

static NEXT_REGISTERED_SERVICE_INDEX: AtomicU32 = AtomicU32::new(FIRST_REGISTERED_SERVICE_INDEX);

impl CoreHandle {
    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "register_module");
        // The graph snapshot takes this lock before reading module descriptors. Keep it for
        // the whole registration transaction so a successful registration cannot be omitted
        // from a concurrently-created frozen graph.
        let frozen_graph = self.lock_frozen_module_graph();
        if frozen_graph.is_some() {
            return Err(CoreError::ModuleGraphFrozen);
        }
        let module_name = descriptor.name.clone();
        if !is_canonical_module_name(&module_name) {
            return Err(CoreError::InvalidModuleName(module_name));
        }
        let driver_count = descriptor.drivers.len();
        let manager_count = descriptor.managers.len();
        let plugin_count = descriptor.plugins.len();
        let service_count = driver_count + manager_count + plugin_count;
        if service_count == 0 {
            return self.register_empty_module(module_name, descriptor);
        }

        {
            let modules = self.lock_modules();
            if modules.contains_key(&module_name) {
                return Err(CoreError::DuplicateModule(module_name));
            }
        }
        if service_count == 1 {
            return self.register_single_service_module(module_name, descriptor);
        }
        if service_count == 2 {
            return self.register_two_service_module(
                module_name,
                descriptor,
                driver_count,
                manager_count,
                plugin_count,
            );
        }
        if service_count == 3 {
            return self.register_three_service_module(
                module_name,
                descriptor,
                driver_count,
                manager_count,
                plugin_count,
            );
        }
        if service_count == 4 {
            return self.register_four_service_module(
                module_name,
                descriptor,
                driver_count,
                manager_count,
                plugin_count,
            );
        }
        if service_count == 5 {
            return self.register_five_service_module(
                module_name,
                descriptor,
                driver_count,
                manager_count,
                plugin_count,
            );
        }

        let mut pending_services = Vec::with_capacity(service_count);
        let mut pending_keys = HashSet::with_capacity(service_count);
        for driver in &descriptor.drivers {
            prepare_service_entry(
                &module_name,
                ServiceKind::Driver,
                driver.name.clone(),
                driver.startup_mode,
                &driver.dependencies,
                ServiceEntryFactory::Service(driver.factory.clone()),
                &mut pending_keys,
                &mut pending_services,
            )?;
        }
        for manager in &descriptor.managers {
            prepare_service_entry(
                &module_name,
                ServiceKind::Manager,
                manager.name.clone(),
                manager.startup_mode,
                &manager.dependencies,
                ServiceEntryFactory::Service(manager.factory.clone()),
                &mut pending_keys,
                &mut pending_services,
            )?;
        }
        for plugin in &descriptor.plugins {
            prepare_service_entry(
                &module_name,
                ServiceKind::Plugin,
                plugin.name.clone(),
                plugin.startup_mode,
                &plugin.dependencies,
                ServiceEntryFactory::Plugin(plugin.factory.clone()),
                &mut pending_keys,
                &mut pending_services,
            )?;
        }
        let module_service_lists =
            module_service_lists(&pending_services, driver_count, manager_count, plugin_count);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        if let Some(duplicate_name) =
            duplicate_existing_pending_service_name(&services, &pending_services)
        {
            return Err(CoreError::DuplicateService(duplicate_name.to_string()));
        }
        assign_service_indices(
            pending_services.iter_mut().map(|(_, entry)| entry),
            service_count,
        )?;
        for (key, entry) in pending_services {
            services.insert(key, entry);
        }
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_empty_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
    ) -> Result<(), CoreError> {
        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: Arc::default(),
                startup_service_names: Arc::default(),
                shutdown_service_names: Arc::default(),
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_single_service_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
    ) -> Result<(), CoreError> {
        let (service_name, mut service_entry) =
            prepare_single_descriptor_service_entry(&module_name, &descriptor)?;
        let module_service_lists = single_service_module_lists(&service_name, &service_entry);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        if services.contains_key(&service_name) {
            return Err(CoreError::DuplicateService(service_name.to_string()));
        }
        assign_service_indices(std::iter::once(&mut service_entry), 1)?;
        services.insert(service_name, service_entry);
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_two_service_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
        driver_count: usize,
        manager_count: usize,
        plugin_count: usize,
    ) -> Result<(), CoreError> {
        let [(first_service_name, first_service_entry), (second_service_name, second_service_entry)] =
            prepare_two_descriptor_service_entries(
                &module_name,
                &descriptor,
                driver_count,
                manager_count,
                plugin_count,
            )?;
        if first_service_name == second_service_name {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        let pending_services = [
            (first_service_name, first_service_entry),
            (second_service_name, second_service_entry),
        ];
        let module_service_lists =
            module_service_lists(&pending_services, driver_count, manager_count, plugin_count);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        let [(first_service_name, mut first_service_entry), (second_service_name, mut second_service_entry)] =
            pending_services;
        if services.contains_key(&first_service_name) {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if services.contains_key(&second_service_name) {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        assign_service_indices([&mut first_service_entry, &mut second_service_entry], 2)?;
        services.insert(first_service_name, first_service_entry);
        services.insert(second_service_name, second_service_entry);
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_three_service_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
        driver_count: usize,
        manager_count: usize,
        plugin_count: usize,
    ) -> Result<(), CoreError> {
        let [(first_service_name, first_service_entry), (second_service_name, second_service_entry), (third_service_name, third_service_entry)] =
            prepare_three_descriptor_service_entries(
                &module_name,
                &descriptor,
                driver_count,
                manager_count,
                plugin_count,
            )?;
        if first_service_name == second_service_name || first_service_name == third_service_name {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if second_service_name == third_service_name {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        let pending_services = [
            (first_service_name, first_service_entry),
            (second_service_name, second_service_entry),
            (third_service_name, third_service_entry),
        ];
        let module_service_lists =
            module_service_lists(&pending_services, driver_count, manager_count, plugin_count);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        let [(first_service_name, mut first_service_entry), (second_service_name, mut second_service_entry), (third_service_name, mut third_service_entry)] =
            pending_services;
        if services.contains_key(&first_service_name) {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if services.contains_key(&second_service_name) {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        if services.contains_key(&third_service_name) {
            return Err(CoreError::DuplicateService(third_service_name.to_string()));
        }
        assign_service_indices(
            [
                &mut first_service_entry,
                &mut second_service_entry,
                &mut third_service_entry,
            ],
            3,
        )?;
        services.insert(first_service_name, first_service_entry);
        services.insert(second_service_name, second_service_entry);
        services.insert(third_service_name, third_service_entry);
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_four_service_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
        driver_count: usize,
        manager_count: usize,
        plugin_count: usize,
    ) -> Result<(), CoreError> {
        let [(first_service_name, first_service_entry), (second_service_name, second_service_entry), (third_service_name, third_service_entry), (fourth_service_name, fourth_service_entry)] =
            prepare_four_descriptor_service_entries(
                &module_name,
                &descriptor,
                driver_count,
                manager_count,
                plugin_count,
            )?;
        if first_service_name == second_service_name
            || first_service_name == third_service_name
            || first_service_name == fourth_service_name
        {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if second_service_name == third_service_name || second_service_name == fourth_service_name {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        if third_service_name == fourth_service_name {
            return Err(CoreError::DuplicateService(third_service_name.to_string()));
        }
        let pending_services = [
            (first_service_name, first_service_entry),
            (second_service_name, second_service_entry),
            (third_service_name, third_service_entry),
            (fourth_service_name, fourth_service_entry),
        ];
        let module_service_lists =
            module_service_lists(&pending_services, driver_count, manager_count, plugin_count);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        let [(first_service_name, mut first_service_entry), (second_service_name, mut second_service_entry), (third_service_name, mut third_service_entry), (fourth_service_name, mut fourth_service_entry)] =
            pending_services;
        if services.contains_key(&first_service_name) {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if services.contains_key(&second_service_name) {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        if services.contains_key(&third_service_name) {
            return Err(CoreError::DuplicateService(third_service_name.to_string()));
        }
        if services.contains_key(&fourth_service_name) {
            return Err(CoreError::DuplicateService(fourth_service_name.to_string()));
        }
        assign_service_indices(
            [
                &mut first_service_entry,
                &mut second_service_entry,
                &mut third_service_entry,
                &mut fourth_service_entry,
            ],
            4,
        )?;
        services.insert(first_service_name, first_service_entry);
        services.insert(second_service_name, second_service_entry);
        services.insert(third_service_name, third_service_entry);
        services.insert(fourth_service_name, fourth_service_entry);
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn register_five_service_module(
        &self,
        module_name: String,
        descriptor: ModuleDescriptor,
        driver_count: usize,
        manager_count: usize,
        plugin_count: usize,
    ) -> Result<(), CoreError> {
        let [(first_service_name, first_service_entry), (second_service_name, second_service_entry), (third_service_name, third_service_entry), (fourth_service_name, fourth_service_entry), (fifth_service_name, fifth_service_entry)] =
            prepare_five_descriptor_service_entries(
                &module_name,
                &descriptor,
                driver_count,
                manager_count,
                plugin_count,
            )?;
        if first_service_name == second_service_name
            || first_service_name == third_service_name
            || first_service_name == fourth_service_name
            || first_service_name == fifth_service_name
        {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if second_service_name == third_service_name
            || second_service_name == fourth_service_name
            || second_service_name == fifth_service_name
        {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        if third_service_name == fourth_service_name || third_service_name == fifth_service_name {
            return Err(CoreError::DuplicateService(third_service_name.to_string()));
        }
        if fourth_service_name == fifth_service_name {
            return Err(CoreError::DuplicateService(fourth_service_name.to_string()));
        }
        let pending_services = [
            (first_service_name, first_service_entry),
            (second_service_name, second_service_entry),
            (third_service_name, third_service_entry),
            (fourth_service_name, fourth_service_entry),
            (fifth_service_name, fifth_service_entry),
        ];
        let module_service_lists =
            module_service_lists(&pending_services, driver_count, manager_count, plugin_count);

        let mut modules = self.lock_modules();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }
        let mut services = self.lock_services();
        let [(first_service_name, mut first_service_entry), (second_service_name, mut second_service_entry), (third_service_name, mut third_service_entry), (fourth_service_name, mut fourth_service_entry), (fifth_service_name, mut fifth_service_entry)] =
            pending_services;
        if services.contains_key(&first_service_name) {
            return Err(CoreError::DuplicateService(first_service_name.to_string()));
        }
        if services.contains_key(&second_service_name) {
            return Err(CoreError::DuplicateService(second_service_name.to_string()));
        }
        if services.contains_key(&third_service_name) {
            return Err(CoreError::DuplicateService(third_service_name.to_string()));
        }
        if services.contains_key(&fourth_service_name) {
            return Err(CoreError::DuplicateService(fourth_service_name.to_string()));
        }
        if services.contains_key(&fifth_service_name) {
            return Err(CoreError::DuplicateService(fifth_service_name.to_string()));
        }
        assign_service_indices(
            [
                &mut first_service_entry,
                &mut second_service_entry,
                &mut third_service_entry,
                &mut fourth_service_entry,
                &mut fifth_service_entry,
            ],
            5,
        )?;
        services.insert(first_service_name, first_service_entry);
        services.insert(second_service_name, second_service_entry);
        services.insert(third_service_name, third_service_entry);
        services.insert(fourth_service_name, fourth_service_entry);
        services.insert(fifth_service_name, fifth_service_entry);
        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                service_names: module_service_lists.service_names,
                startup_service_names: module_service_lists.startup_service_names,
                shutdown_service_names: module_service_lists.shutdown_service_names,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }
}

fn assign_service_indices<'a>(
    entries: impl IntoIterator<Item = &'a mut ServiceEntry>,
    service_count: usize,
) -> Result<(), CoreError> {
    let service_count =
        u32::try_from(service_count).map_err(|_| CoreError::ServiceIdentityIndexExhausted)?;
    let first_index = NEXT_REGISTERED_SERVICE_INDEX
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next_index| {
            next_index.checked_add(service_count)
        })
        .map_err(|_| CoreError::ServiceIdentityIndexExhausted)?;

    for (offset, entry) in entries.into_iter().enumerate() {
        let offset = u32::try_from(offset).expect("service identity offset should fit in u32");
        entry.assign_index(first_index + offset);
    }
    Ok(())
}
