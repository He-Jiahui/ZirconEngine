use std::collections::BTreeMap;

use crate::plugin::PluginModuleKind;

/// A borrowed lookup key. Plugin ids become owned only when a loaded entry is inserted; every
/// stable lookup carries its module kind separately and can use the caller's `&str` directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativePluginLiveKey<'a> {
    module_kind: PluginModuleKind,
    plugin_id: &'a str,
}

impl<'a> NativePluginLiveKey<'a> {
    pub(super) const fn new(module_kind: PluginModuleKind, plugin_id: &'a str) -> Self {
        Self {
            module_kind,
            plugin_id,
        }
    }

    const fn module_kind(self) -> PluginModuleKind {
        self.module_kind
    }

    const fn plugin_id(self) -> &'a str {
        self.plugin_id
    }
}

pub(super) const fn live_key<'a>(
    module_kind: PluginModuleKind,
    plugin_id: &'a str,
) -> NativePluginLiveKey<'a> {
    NativePluginLiveKey::new(module_kind, plugin_id)
}

/// Module-kind partitioning keeps the stored key typed without needing an allocated composite
/// string for every lookup. Runtime, editor, native, and VM ids may therefore safely share the
/// same plugin-id spelling.
#[derive(Debug)]
pub(super) struct NativePluginLiveRegistry<T> {
    runtime: BTreeMap<String, T>,
    editor: BTreeMap<String, T>,
    native: BTreeMap<String, T>,
    vm: BTreeMap<String, T>,
}

impl<T> Default for NativePluginLiveRegistry<T> {
    fn default() -> Self {
        Self {
            runtime: BTreeMap::new(),
            editor: BTreeMap::new(),
            native: BTreeMap::new(),
            vm: BTreeMap::new(),
        }
    }
}

impl<T> NativePluginLiveRegistry<T> {
    pub(super) fn get(&self, key: &NativePluginLiveKey<'_>) -> Option<&T> {
        self.map(key.module_kind()).get(key.plugin_id())
    }

    pub(super) fn insert(&mut self, key: NativePluginLiveKey<'_>, value: T) -> Option<T> {
        let map = self.map_mut(key.module_kind());
        if let Some(current) = map.get_mut(key.plugin_id()) {
            let previous = std::mem::replace(current, value);
            return Some(previous);
        }
        map.insert(key.plugin_id().to_string(), value)
    }

    pub(super) fn remove(&mut self, key: &NativePluginLiveKey<'_>) -> Option<T> {
        self.map_mut(key.module_kind()).remove(key.plugin_id())
    }

    pub(super) fn plugin_ids(&self, module_kind: PluginModuleKind) -> impl Iterator<Item = &str> {
        self.map(module_kind).keys().map(String::as_str)
    }

    pub(super) fn entries(
        &self,
        module_kind: PluginModuleKind,
    ) -> impl Iterator<Item = (&str, &T)> {
        self.map(module_kind)
            .iter()
            .map(|(plugin_id, value)| (plugin_id.as_str(), value))
    }

    fn map(&self, module_kind: PluginModuleKind) -> &BTreeMap<String, T> {
        match module_kind {
            PluginModuleKind::Runtime => &self.runtime,
            PluginModuleKind::Editor => &self.editor,
            PluginModuleKind::Native => &self.native,
            PluginModuleKind::Vm => &self.vm,
        }
    }

    fn map_mut(&mut self, module_kind: PluginModuleKind) -> &mut BTreeMap<String, T> {
        match module_kind {
            PluginModuleKind::Runtime => &mut self.runtime,
            PluginModuleKind::Editor => &mut self.editor,
            PluginModuleKind::Native => &mut self.native,
            PluginModuleKind::Vm => &mut self.vm,
        }
    }
}

pub(super) fn module_kind_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime",
        PluginModuleKind::Editor => "editor",
        PluginModuleKind::Native => "native",
        PluginModuleKind::Vm => "vm",
    }
}

pub(super) fn module_kind_article_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "a runtime",
        PluginModuleKind::Editor => "an editor",
        PluginModuleKind::Native => "a native",
        PluginModuleKind::Vm => "a vm",
    }
}

#[cfg(test)]
mod tests {
    use super::{NativePluginLiveRegistry, live_key};
    use crate::plugin::PluginModuleKind;

    #[test]
    fn replacement_returns_previous_value_without_replacing_key() {
        let mut registry = NativePluginLiveRegistry::default();
        assert_eq!(
            registry.insert(live_key(PluginModuleKind::Runtime, "physics"), 1_u8),
            None
        );
        let original_key = registry
            .plugin_ids(PluginModuleKind::Runtime)
            .next()
            .expect("runtime plugin id")
            .as_ptr();

        let borrowed_id = String::from("physics");
        assert_eq!(
            registry.insert(
                live_key(PluginModuleKind::Runtime, borrowed_id.as_str()),
                2_u8,
            ),
            Some(1)
        );

        assert_eq!(
            registry
                .plugin_ids(PluginModuleKind::Runtime)
                .next()
                .expect("runtime plugin id")
                .as_ptr(),
            original_key
        );
        assert_eq!(
            registry.get(&live_key(PluginModuleKind::Runtime, "physics")),
            Some(&2)
        );
    }
}
