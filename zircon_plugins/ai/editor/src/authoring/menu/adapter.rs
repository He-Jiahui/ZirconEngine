use zircon_plugin_ai_runtime::behavior_tree::{
    BehaviorNodeCategory, BehaviorNodeDescriptor, BehaviorNodeSemantics,
};
use zircon_runtime::core::framework::ai::{AiBehaviorNodeDescriptor, AiBehaviorNodeKind as Kind};

use super::super::{
    search::matches_query, topology, AiBehaviorTreeAsset, AiBehaviorTreeEdit,
    AiBehaviorTreeEditCandidate, AiBlackboardProviderSnapshot,
};
use super::{
    AiBehaviorTreeMenuCommand as Command, AiBehaviorTreeMenuDisabledReason as Disabled,
    AiBehaviorTreeMenuError as Error, AiBehaviorTreeMenuRequest as Request, AiBehaviorTreeMenuRow,
    AiBehaviorTreeNodeMenuRow,
};

/// A disposable view over one source, selection and provider snapshot. `writable`
/// is the host's read-only state, not a budget permit or transaction admission.
/// Rebuild after source, selection or provider lifecycle changes. The host still
/// checks live document identity/revision and selection before one prepare/apply.
pub struct AiBehaviorTreeMenuAdapter<'a> {
    asset: &'a AiBehaviorTreeAsset,
    selection: &'a [&'a str],
    writable: bool,
    providers: AiBlackboardProviderSnapshot<'a>,
}

impl<'a> AiBehaviorTreeMenuAdapter<'a> {
    pub fn new(
        asset: &'a AiBehaviorTreeAsset,
        selection: &'a [&'a str],
        writable: bool,
        providers: AiBlackboardProviderSnapshot<'a>,
    ) -> Self {
        Self {
            asset,
            selection,
            writable,
            providers,
        }
    }

    /// Borrowed, allocation-free search for SearchInput.query -> ListRow text/value.
    /// Keyboard navigation, popup lifetime, IME and focus remain the shared host's job.
    pub fn rows<'b>(&'b self, query: &'b str) -> impl Iterator<Item = AiBehaviorTreeMenuRow> + 'b {
        [
            (Command::CreateChild, "Create Child", "create_child"),
            (Command::WrapDecorator, "Add Decorator", "wrap_decorator"),
            (
                Command::UnwrapDecorator,
                "Remove Decorator",
                "unwrap_decorator",
            ),
            (Command::DeleteSubtree, "Delete Subtree", "delete_subtree"),
            (Command::AddService, "Add Service", "add_service"),
            (
                Command::ToggleBreakpoint,
                "Toggle Breakpoint",
                "toggle_breakpoint",
            ),
            (Command::StepInto, "Step Into", "step_into"),
            (Command::StepOver, "Step Over", "step_over"),
        ]
        .into_iter()
        .filter(move |(_, text, value)| matches_query(query, text, value))
        .map(move |(command, text, value)| AiBehaviorTreeMenuRow {
            command,
            text,
            value,
            disabled: self.eligibility(command).err(),
        })
    }

    pub fn node_rows<'b>(
        &'b self,
        command: Command,
        query: &'b str,
    ) -> impl Iterator<Item = AiBehaviorTreeNodeMenuRow<'b>> + 'b {
        self.providers
            .catalog
            .descriptors()
            .iter()
            .filter(move |descriptor| match command {
                Command::CreateChild => matches!(
                    descriptor.category(),
                    BehaviorNodeCategory::Composite | BehaviorNodeCategory::Task
                ),
                Command::WrapDecorator => descriptor.category() == BehaviorNodeCategory::Decorator,
                _ => false,
            })
            .filter(move |descriptor| {
                matches_query(query, descriptor.display_name(), descriptor.id())
            })
            .map(move |descriptor| {
                let kind = creation_kind(descriptor);
                let disabled = self
                    .eligibility(command)
                    .err()
                    .or_else(|| self.provider_eligibility(descriptor).err());
                AiBehaviorTreeNodeMenuRow {
                    descriptor,
                    kind,
                    disabled,
                }
            })
    }

    pub fn eligibility(&self, command: Command) -> Result<(), Disabled> {
        match command {
            Command::AddService => return Err(Disabled::ServiceAttachmentUnsupported),
            Command::ToggleBreakpoint | Command::StepInto | Command::StepOver => {
                return Err(Disabled::RuntimeTraceControlUnsupported)
            }
            _ => {}
        }
        if !self.writable {
            return Err(Disabled::ReadOnly);
        }
        let node = self.selected()?;
        match command {
            Command::CreateChild => {
                if !matches!(node.kind, Kind::Selector | Kind::Sequence | Kind::Parallel) {
                    return Err(if node.kind == Kind::Decorator {
                        Disabled::RequiredDecoratorChild
                    } else {
                        Disabled::LeafHasNoChildren
                    });
                }
                editable_children(node)
            }
            Command::DeleteSubtree => {
                if self.asset.descriptor().root_node == node.id {
                    return Err(Disabled::RootDeletion);
                }
                self.editable_parent(node, true)
            }
            Command::WrapDecorator => self.editable_parent(node, false),
            Command::UnwrapDecorator => {
                if node.kind != Kind::Decorator {
                    return Err(Disabled::NotDecorator);
                }
                self.editable_parent(node, false)
            }
            _ => unreachable!("unsupported commands returned above"),
        }
    }

    /// After host allocation admission, build exactly one fully compiled candidate
    /// and its inverse. Never publish per-node edits, save, mutate a runtime manager
    /// or install a history entry here. No-op remains None for the existing host.
    pub fn prepare(
        &self,
        request: Request<'_>,
    ) -> Result<Option<AiBehaviorTreeEditCandidate>, Error> {
        let command = request.command();
        self.eligibility(command).map_err(Error::Disabled)?;
        let selected = self.selected().map_err(Error::Disabled)?;
        let edit = match request {
            Request::CreateChild { index, node } => {
                self.check_node(node).map_err(Error::Disabled)?;
                if node.kind == Kind::Decorator || node.kind == Kind::Service {
                    return Err(Error::Disabled(Disabled::NodeKindMismatch));
                }
                AiBehaviorTreeEdit::InsertChild {
                    parent_id: selected.id.clone(),
                    index,
                    node: node.clone(),
                }
            }
            Request::WrapDecorator { decorator } => {
                self.check_node(decorator).map_err(Error::Disabled)?;
                if decorator.kind != Kind::Decorator {
                    return Err(Error::Disabled(Disabled::NodeKindMismatch));
                }
                AiBehaviorTreeEdit::WrapDecorator {
                    child_id: selected.id.clone(),
                    decorator: decorator.clone(),
                }
            }
            Request::UnwrapDecorator => AiBehaviorTreeEdit::UnwrapDecorator {
                node_id: selected.id.clone(),
            },
            Request::DeleteSubtree => AiBehaviorTreeEdit::DeleteSubtree {
                node_id: selected.id.clone(),
            },
            _ => unreachable!("unsupported commands returned before candidate preparation"),
        };
        Ok(self.asset.prepare_edit(&edit)?)
    }

    fn selected(&self) -> Result<&AiBehaviorNodeDescriptor, Disabled> {
        let id = match self.selection {
            [] => return Err(Disabled::NoSelection),
            [id] => *id,
            _ => return Err(Disabled::MultipleSelection),
        };
        self.asset
            .descriptor()
            .nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or(Disabled::MissingNode)
    }

    fn editable_parent(
        &self,
        node: &AiBehaviorNodeDescriptor,
        deleting: bool,
    ) -> Result<(), Disabled> {
        if let Some(parent) = self
            .asset
            .descriptor()
            .nodes
            .iter()
            .find(|parent| parent.children.contains(&node.id))
        {
            if deleting && parent.kind == Kind::Decorator {
                return Err(Disabled::RequiredDecoratorChild);
            }
            editable_children(parent)?;
        }
        Ok(())
    }

    fn provider_eligibility(&self, descriptor: &BehaviorNodeDescriptor) -> Result<(), Disabled> {
        if self.providers.registry_identity.trim().is_empty() {
            return Err(Disabled::MissingProviderIdentity);
        }
        let owner = self
            .providers
            .catalog
            .resolve(descriptor.id())
            .and_then(|slot| self.providers.catalog.owner_for_slot(slot))
            .ok_or(Disabled::UnknownProvider)?;
        // The existing asset compiler supports the built-in source language. An
        // external factory/catalog row does not grant this adapter its typed schema.
        if owner != self.providers.builtin_provider || creation_kind(descriptor).is_none() {
            return Err(Disabled::UnsupportedProvider);
        }
        Ok(())
    }

    fn check_node(&self, node: &AiBehaviorNodeDescriptor) -> Result<(), Disabled> {
        let descriptor = self
            .providers
            .catalog
            .resolve(&node.implementation)
            .and_then(|slot| self.providers.catalog.get(slot))
            .ok_or(Disabled::UnknownProvider)?;
        self.provider_eligibility(descriptor)?;
        if creation_kind(descriptor) != Some(node.kind) {
            return Err(Disabled::NodeKindMismatch);
        }
        Ok(())
    }
}

fn editable_children(node: &AiBehaviorNodeDescriptor) -> Result<(), Disabled> {
    if topology::has_explicit_child_weights(node) {
        Err(Disabled::WeightedTopology)
    } else {
        Ok(())
    }
}

fn creation_kind(descriptor: &BehaviorNodeDescriptor) -> Option<Kind> {
    use BehaviorNodeSemantics as S;
    // This is the supported source-language mapping, not a second node catalog.
    // A replacement descriptor cannot borrow a built-in ID with different semantics.
    let kind = match (descriptor.id(), descriptor.semantics()) {
        ("selector", S::Selector) | ("random_selector", S::RandomSelector) => Kind::Selector,
        ("sequence", S::Sequence) => Kind::Sequence,
        ("parallel", S::Parallel) => Kind::Parallel,
        ("blackboard_condition", S::BlackboardCondition)
        | ("cooldown", S::Cooldown)
        | ("time_limit", S::TimeLimit)
        | ("loop", S::Loop)
        | ("inverter", S::Inverter)
        | ("force_result", S::ForceResult) => Kind::Decorator,
        ("wait", S::Wait)
        | ("move_to", S::MoveTo)
        | ("play_animation", S::PlayAnimation)
        | ("set_blackboard", S::SetBlackboard)
        | ("emit_event", S::EmitEvent)
        | ("script_task", S::ScriptTask) => Kind::Task,
        ("run_subtree", S::RunSubtree) => Kind::Subtree,
        _ => return None,
    };
    let category = match kind {
        Kind::Selector | Kind::Sequence | Kind::Parallel => BehaviorNodeCategory::Composite,
        Kind::Decorator => BehaviorNodeCategory::Decorator,
        Kind::Task | Kind::Subtree => BehaviorNodeCategory::Task,
        Kind::Service => return None,
    };
    (category == descriptor.category()).then_some(kind)
}
