use crate::plugin::PluginPackageManifest;

// Borrowed coordinate field state keeps the presence rule independent from shape validation.
pub(super) struct RuntimePluginPackageCoordinateFields<'a> {
    fields: [&'a str; 3],
}

impl<'a> RuntimePluginPackageCoordinateFields<'a> {
    pub(super) fn from_manifest(package_manifest: &'a PluginPackageManifest) -> Self {
        Self {
            fields: [
                package_manifest.package_prefix.as_str(),
                package_manifest.package_company.as_str(),
                package_manifest.package_name.as_str(),
            ],
        }
    }

    pub(super) fn declares_any(&self) -> bool {
        self.fields.iter().any(|value| !value.is_empty())
    }

    pub(super) fn declares_all(&self) -> bool {
        self.fields.iter().all(|value| !value.is_empty())
    }
}
