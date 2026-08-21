use crate::core::CoreError;

use super::super::super::descriptors::RegistryName;
use super::super::CoreHandle;

impl CoreHandle {
    pub(super) fn resolve_startup_services(
        &self,
        startup_services: &[RegistryName],
    ) -> Result<(), CoreError> {
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
}
