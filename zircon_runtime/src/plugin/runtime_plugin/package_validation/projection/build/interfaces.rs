use std::collections::HashSet;

use crate::plugin::PluginPackageManifest;

use super::super::duplicate_identity::DuplicateIdentity;
use super::super::duplicate_occurrence::DuplicateOccurrence;
use super::index_identity;

/// 提供接口的身份属于包域；方法名与槽位属于所在接口，所需能力属于所在方法。
/// 这些作用域必须与接口校验传入的原始行号一致，避免不同接口的方法互相误报。
pub(super) fn index_interfaces<'a>(
    manifest: &'a PluginPackageManifest,
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    count: &mut usize,
) {
    for (interface_index, interface) in manifest.provides_interfaces.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::ProvidedInterface(&interface.id),
            DuplicateOccurrence::ProvidedInterface(interface_index),
            count,
        );
        for (method_index, method) in interface.methods.iter().enumerate() {
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::ProvidedMethodName {
                    interface: interface_index,
                    value: &method.name,
                },
                DuplicateOccurrence::ProvidedMethodName {
                    interface: interface_index,
                    method: method_index,
                },
                count,
            );
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::ProvidedMethodSlot {
                    interface: interface_index,
                    value: method.method_slot,
                },
                DuplicateOccurrence::ProvidedMethodSlot {
                    interface: interface_index,
                    method: method_index,
                },
                count,
            );
            for (capability_index, capability) in method.required_capabilities.iter().enumerate() {
                index_identity(
                    seen,
                    duplicates,
                    DuplicateIdentity::ProvidedMethodCapability {
                        interface: interface_index,
                        method: method_index,
                        value: capability,
                    },
                    DuplicateOccurrence::ProvidedMethodCapability {
                        interface: interface_index,
                        method: method_index,
                        capability: capability_index,
                    },
                    count,
                );
            }
        }
    }
}
