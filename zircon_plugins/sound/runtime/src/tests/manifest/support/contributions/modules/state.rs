use super::super::StaticModule;
use super::line::parse_module_contribution_line;

// Preserves the static plugin.toml scanner's table-boundary behavior for module rows.
#[derive(Default)]
pub(super) struct ModuleContributionParserState {
    modules: Vec<StaticModule>,
    current_name: Option<String>,
    current_kind: Option<zircon_runtime::plugin::PluginModuleKind>,
    current_crate_name: Option<String>,
    current_target_modes: Vec<zircon_runtime::RuntimeTargetMode>,
    current_capabilities: Vec<String>,
    inside_module: bool,
}

impl ModuleContributionParserState {
    pub(super) fn parse_manifest_line(&mut self, line: &str) {
        if line == "[[modules]]" {
            self.push_current_module();
            self.inside_module = true;
            return;
        }
        if line.starts_with("[[") {
            self.push_current_module();
            self.inside_module = false;
        }
        if !self.inside_module {
            return;
        }
        parse_module_contribution_line(
            line,
            &mut self.current_name,
            &mut self.current_kind,
            &mut self.current_crate_name,
            &mut self.current_target_modes,
            &mut self.current_capabilities,
        );
    }

    pub(super) fn finish(mut self) -> Vec<StaticModule> {
        self.push_current_module();
        self.modules
    }

    fn push_current_module(&mut self) {
        let Some(name) = self.current_name.take() else {
            return;
        };
        self.modules.push((
            name,
            self.current_kind
                .take()
                .expect("sound module should declare kind"),
            self.current_crate_name
                .take()
                .expect("sound module should declare crate_name"),
            std::mem::take(&mut self.current_target_modes),
            std::mem::take(&mut self.current_capabilities),
        ));
    }
}
