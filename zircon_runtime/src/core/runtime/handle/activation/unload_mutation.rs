use std::collections::HashMap;

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;
use crate::core::runtime::ServiceObject;

// 调用准入关闭、在途守卫排空和模块清理完成后才使槽位代次失效。
pub(super) fn unload_services(
    module_name: &str,
    services: &mut HashMap<RegistryName, ServiceEntry>,
    unload_order: &[RegistryName],
) -> Vec<(String, String, ServiceObject)> {
    let mut retired = Vec::with_capacity(unload_order.len());
    if let [service_name] = unload_order {
        unload_service(module_name, services, service_name, &mut retired);
        return retired;
    }
    if let [first_service_name, second_service_name] = unload_order {
        unload_service(module_name, services, first_service_name, &mut retired);
        unload_service(module_name, services, second_service_name, &mut retired);
        return retired;
    }
    if let [first_service_name, second_service_name, third_service_name] = unload_order {
        unload_service(module_name, services, first_service_name, &mut retired);
        unload_service(module_name, services, second_service_name, &mut retired);
        unload_service(module_name, services, third_service_name, &mut retired);
        return retired;
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name] =
        unload_order
    {
        unload_service(module_name, services, first_service_name, &mut retired);
        unload_service(module_name, services, second_service_name, &mut retired);
        unload_service(module_name, services, third_service_name, &mut retired);
        unload_service(module_name, services, fourth_service_name, &mut retired);
        return retired;
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name, fifth_service_name] =
        unload_order
    {
        unload_service(module_name, services, first_service_name, &mut retired);
        unload_service(module_name, services, second_service_name, &mut retired);
        unload_service(module_name, services, third_service_name, &mut retired);
        unload_service(module_name, services, fourth_service_name, &mut retired);
        unload_service(module_name, services, fifth_service_name, &mut retired);
        return retired;
    }

    for service_name in unload_order {
        unload_service(module_name, services, service_name, &mut retired);
    }
    retired
}

fn unload_service(
    module_name: &str,
    services: &mut HashMap<RegistryName, ServiceEntry>,
    service_name: &RegistryName,
    retired: &mut Vec<(String, String, ServiceObject)>,
) {
    if let Some(entry) = services.get_mut(service_name) {
        if let Some(instance) = entry.invalidate_for_unload() {
            retired.push((module_name.to_owned(), service_name.to_string(), instance));
        }
    }
}
