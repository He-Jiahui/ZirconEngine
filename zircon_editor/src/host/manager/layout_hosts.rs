use std::collections::{BTreeMap, BTreeSet};

use crate::layout::{
    ActivityDrawerLayout, DocumentNode, MainHostPageLayout, MainPageId, TabStackLayout,
    WorkbenchLayout,
};
use crate::view::{ViewHost, ViewInstanceId};

use super::builtin_layout::builtin_hybrid_layout;

pub(super) fn active_tab_from_document(node: &DocumentNode) -> Option<ViewInstanceId> {
    match node {
        DocumentNode::Tabs(stack) => stack.active_tab.clone(),
        DocumentNode::SplitNode { first, second, .. } => {
            active_tab_from_document(first).or_else(|| active_tab_from_document(second))
        }
    }
}

pub(super) fn collect_instance_hosts(
    layout: &WorkbenchLayout,
) -> BTreeMap<ViewInstanceId, ViewHost> {
    let mut placements = BTreeMap::new();

    for (slot, drawer) in &layout.drawers {
        for instance_id in &drawer.tab_stack.tabs {
            placements.insert(instance_id.clone(), ViewHost::Drawer(*slot));
        }
    }

    for page in &layout.main_pages {
        match page {
            MainHostPageLayout::WorkbenchPage {
                id,
                document_workspace,
                ..
            } => collect_document_hosts(document_workspace, &mut placements, |path| {
                ViewHost::Document(id.clone(), path)
            }),
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } => {
                placements.insert(window_instance.clone(), ViewHost::ExclusivePage(id.clone()));
            }
        }
    }

    for window in &layout.floating_windows {
        collect_document_hosts(&window.workspace, &mut placements, |path| {
            ViewHost::FloatingWindow(window.window_id.clone(), path)
        });
    }

    placements
}

fn collect_document_hosts(
    node: &DocumentNode,
    placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
    make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
) {
    fn visit(
        node: &DocumentNode,
        path: &mut Vec<usize>,
        placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
        make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
    ) {
        match node {
            DocumentNode::Tabs(stack) => {
                let host = make_host(path.clone());
                for instance_id in &stack.tabs {
                    placements.insert(instance_id.clone(), host.clone());
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                path.push(0);
                visit(first, path, placements, make_host);
                path.pop();
                path.push(1);
                visit(second, path, placements, make_host);
                path.pop();
            }
        }
    }

    let mut path = Vec::new();
    visit(node, &mut path, placements, make_host);
}

pub(super) fn repair_builtin_shell_layout(layout: &mut WorkbenchLayout) {
    let baseline = builtin_hybrid_layout();
    let mut present: BTreeSet<ViewInstanceId> =
        collect_instance_hosts(layout).into_keys().collect();

    for (slot, baseline_drawer) in &baseline.drawers {
        let target_drawer = layout
            .drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if present.insert(instance_id.clone()) {
                target_drawer.tab_stack.tabs.push(instance_id.clone());
            }
        }

        if target_drawer
            .tab_stack
            .active_tab
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.tab_stack.active_tab = baseline_drawer
                .tab_stack
                .active_tab
                .clone()
                .filter(|active| target_drawer.tab_stack.tabs.contains(active))
                .or_else(|| target_drawer.tab_stack.tabs.first().cloned());
        }

        if target_drawer
            .active_view
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.active_view = target_drawer.tab_stack.active_tab.clone();
        }
    }

    let Some(baseline_stack) = baseline_main_page_tabs(&baseline) else {
        return;
    };
    let stack = first_tab_stack_mut(ensure_workbench_document_root(layout));
    for instance_id in baseline_stack.tabs {
        if present.insert(instance_id.clone()) {
            stack.tabs.push(instance_id);
        }
    }
    if stack
        .active_tab
        .as_ref()
        .is_none_or(|active| !stack.tabs.contains(active))
    {
        stack.active_tab = baseline_stack
            .active_tab
            .filter(|active| stack.tabs.contains(active))
            .or_else(|| stack.tabs.first().cloned());
    }
}

fn baseline_main_page_tabs(layout: &WorkbenchLayout) -> Option<TabStackLayout> {
    layout.main_pages.iter().find_map(|page| match page {
        MainHostPageLayout::WorkbenchPage {
            document_workspace, ..
        } => first_tab_stack(document_workspace).cloned(),
        MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
    })
}

fn first_tab_stack(node: &DocumentNode) -> Option<&TabStackLayout> {
    match node {
        DocumentNode::Tabs(stack) => Some(stack),
        DocumentNode::SplitNode { first, second, .. } => {
            first_tab_stack(first).or_else(|| first_tab_stack(second))
        }
    }
}

fn first_tab_stack_mut(node: &mut DocumentNode) -> &mut TabStackLayout {
    match node {
        DocumentNode::Tabs(stack) => stack,
        DocumentNode::SplitNode { first, second, .. } => {
            if let Some(stack) = first_tab_stack(first) {
                let _ = stack;
                first_tab_stack_mut(first)
            } else {
                first_tab_stack_mut(second)
            }
        }
    }
}

fn ensure_workbench_document_root(layout: &mut WorkbenchLayout) -> &mut DocumentNode {
    if let Some(index) = layout
        .main_pages
        .iter()
        .position(|page| matches!(page, MainHostPageLayout::WorkbenchPage { .. }))
    {
        match &mut layout.main_pages[index] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    } else {
        layout.main_pages.insert(
            0,
            MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: DocumentNode::default(),
            },
        );
        match &mut layout.main_pages[0] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    }
}
