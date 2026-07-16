use std::collections::HashMap;

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;

pub(super) fn unload_services(
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
        entry.invalidate_for_unload();
    }
}
