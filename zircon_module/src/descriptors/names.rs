use zircon_core::{DependencySpec, RegistryName, ServiceKind};

pub fn qualified_name(module: &str, kind: ServiceKind, service: &str) -> RegistryName {
    RegistryName::from_parts(module, kind, service)
}

pub fn dependency_on(module: &str, kind: ServiceKind, service: &str) -> DependencySpec {
    DependencySpec::named(qualified_name(module, kind, service))
}
