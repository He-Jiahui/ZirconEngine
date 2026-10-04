import { useState } from "react";
import type { KeyboardEvent } from "react";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import { Box, Collapse, List, ListItemButton, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubTreeNode {
  id: string;
  label: string;
  detail?: string;
  children?: HubTreeNode[];
}

export interface HubTreeViewProps {
  nodes: HubTreeNode[];
  defaultExpanded?: string[];
  onSelect?: (node: HubTreeNode) => void;
  ariaLabel?: string;
}

interface VisibleTreeNode {
  node: HubTreeNode;
  depth: number;
  parentId?: string;
}

export function HubTreeView({ nodes, defaultExpanded = [], onSelect, ariaLabel = "Tree" }: HubTreeViewProps) {
  const [expanded, setExpanded] = useState(() => new Set(defaultExpanded));
  const [focusedNodeId, setFocusedNodeId] = useState(() => nodes[0]?.id ?? null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const hasSelectHandler = Boolean(onSelect);
  const visibleNodes = flattenVisibleNodes(nodes, expanded);
  const visibleIds = new Set(visibleNodes.map(({ node }) => node.id));
  const tabbableNodeId = focusedNodeId && visibleIds.has(focusedNodeId) ? focusedNodeId : visibleNodes[0]?.node.id;

  const focusNode = (nodeId: string) => {
    setFocusedNodeId(nodeId);
    if (typeof document === "undefined") {
      return;
    }
    const moveFocus = () => {
      const target = [...document.querySelectorAll<HTMLElement>("[data-hub-tree-node-id]")]
        .find((element) => element.dataset.hubTreeNodeId === nodeId);
      if (target) {
        target.focus();
        return;
      }
      requestAnimationFrame(() => {
        const nextTarget = [...document.querySelectorAll<HTMLElement>("[data-hub-tree-node-id]")]
          .find((element) => element.dataset.hubTreeNodeId === nodeId);
        nextTarget?.focus();
      });
    };
    moveFocus();
  };

  const selectNode = (node: HubTreeNode) => {
    setSelectedNodeId(node.id);
    onSelect?.(node);
  };

  const toggle = (node: HubTreeNode) => {
    const hasChildren = (node.children?.length ?? 0) > 0;
    if (!hasSelectHandler && !hasChildren) {
      return;
    }
    if (!hasChildren) {
      selectNode(node);
      return;
    }
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(node.id)) {
        next.delete(node.id);
      } else {
        next.add(node.id);
      }
      return next;
    });
  };

  const handleKeyDown = (event: KeyboardEvent, entry: VisibleTreeNode) => {
    const entryIndex = visibleNodes.findIndex(({ node }) => node.id === entry.node.id);
    if (entryIndex < 0) {
      return;
    }
    const hasChildren = (entry.node.children?.length ?? 0) > 0;
    const open = expanded.has(entry.node.id);
    let targetId: string | undefined;

    switch (event.key) {
      case "ArrowDown":
        targetId = visibleNodes[entryIndex + 1]?.node.id;
        break;
      case "ArrowUp":
        targetId = visibleNodes[entryIndex - 1]?.node.id;
        break;
      case "Home":
        targetId = visibleNodes[0]?.node.id;
        break;
      case "End":
        targetId = visibleNodes.at(-1)?.node.id;
        break;
      case "ArrowRight":
        if (hasChildren && !open) {
          event.preventDefault();
          toggle(entry.node);
          return;
        }
        if (hasChildren && open) {
          targetId = visibleNodes[entryIndex + 1]?.node.id;
        }
        break;
      case "ArrowLeft":
        if (hasChildren && open) {
          event.preventDefault();
          toggle(entry.node);
          return;
        }
        targetId = entry.parentId;
        break;
      case "Enter":
      case " ":
        event.preventDefault();
        toggle(entry.node);
        return;
      default:
        return;
    }

    if (targetId) {
      event.preventDefault();
      focusNode(targetId);
    }
  };

  return (
    <List dense role="tree" aria-label={ariaLabel} sx={{ p: 0 }}>
      {nodes.map((node) => (
        <TreeNode
          key={node.id}
          node={node}
          depth={0}
          expanded={expanded}
          hasSelectHandler={hasSelectHandler}
          onToggle={toggle}
          onKeyDown={handleKeyDown}
          onFocus={setFocusedNodeId}
          focusedNodeId={tabbableNodeId}
          selectedNodeId={selectedNodeId}
        />
      ))}
    </List>
  );
}

function flattenVisibleNodes(
  nodes: HubTreeNode[],
  expanded: Set<string>,
  depth = 0,
  parentId?: string,
): VisibleTreeNode[] {
  const visible: VisibleTreeNode[] = [];
  for (const node of nodes) {
    visible.push({ node, depth, parentId });
    if ((node.children?.length ?? 0) > 0 && expanded.has(node.id)) {
      visible.push(...flattenVisibleNodes(node.children!, expanded, depth + 1, node.id));
    }
  }
  return visible;
}

function TreeNode({
  node,
  depth,
  parentId,
  expanded,
  hasSelectHandler,
  onToggle,
  onKeyDown,
  onFocus,
  focusedNodeId,
  selectedNodeId,
}: {
  node: HubTreeNode;
  depth: number;
  parentId?: string;
  expanded: Set<string>;
  hasSelectHandler: boolean;
  onToggle: (node: HubTreeNode) => void;
  onKeyDown: (event: KeyboardEvent, entry: VisibleTreeNode) => void;
  onFocus: (nodeId: string) => void;
  focusedNodeId?: string;
  selectedNodeId: string | null;
}) {
  const childCount = node.children?.length ?? 0;
  const open = expanded.has(node.id);
  const rowIsActionable = childCount > 0 || hasSelectHandler;
  const Icon = childCount > 0 && open ? ExpandMoreIcon : ChevronRightIcon;
  const entry = { node, depth, parentId };

  return (
    <Box>
      <ListItemButton
        role="treeitem"
        aria-level={depth + 1}
        aria-expanded={childCount > 0 ? open : undefined}
        aria-selected={selectedNodeId === node.id}
        data-hub-tree-node-id={node.id}
        tabIndex={focusedNodeId === node.id ? 0 : -1}
        onFocus={() => onFocus(node.id)}
        onKeyDown={(event) => onKeyDown(event, entry)}
        onClick={() => onToggle(node)}
        sx={{
          minHeight: 38,
          pl: 0.8 + depth * 2,
          pr: 1,
          borderRadius: `${hubTokens.radius.compact}px`,
          color: hubTokens.colors.textSoft,
          cursor: rowIsActionable ? "pointer" : "default",
          "&:hover": { backgroundColor: rowIsActionable ? "rgba(255,255,255,0.045)" : "transparent" },
          "&[aria-selected=\"true\"]": {
            backgroundColor: "rgba(18,82,80,0.38)",
          },
        }}
      >
        <Icon sx={{ mr: 0.8, fontSize: 18, opacity: childCount > 0 ? 1 : 0.4 }} />
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="body2" noWrap sx={{ color: hubTokens.colors.text }}>
            {node.label}
          </Typography>
          {node.detail ? (
            <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
              {node.detail}
            </Typography>
          ) : null}
        </Box>
      </ListItemButton>
      {childCount > 0 ? (
        <Collapse in={open} timeout={140} unmountOnExit>
          <Box role="group">
            {node.children!.map((child) => (
              <TreeNode
                key={child.id}
                node={child}
                depth={depth + 1}
                parentId={node.id}
                expanded={expanded}
                hasSelectHandler={hasSelectHandler}
                onToggle={onToggle}
                onKeyDown={onKeyDown}
                onFocus={onFocus}
                focusedNodeId={focusedNodeId}
                selectedNodeId={selectedNodeId}
              />
            ))}
          </Box>
        </Collapse>
      ) : null}
    </Box>
  );
}
