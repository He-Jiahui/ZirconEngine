import { useState } from "react";
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
}

export function HubTreeView({ nodes, defaultExpanded = [], onSelect }: HubTreeViewProps) {
  const [expanded, setExpanded] = useState(() => new Set(defaultExpanded));
  const hasSelectHandler = Boolean(onSelect);

  const toggle = (node: HubTreeNode) => {
    const hasChildren = (node.children?.length ?? 0) > 0;
    if (!hasSelectHandler && !hasChildren) {
      return;
    }
    if (!hasChildren) {
      onSelect?.(node);
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

  return (
    <List dense sx={{ p: 0 }}>
      {nodes.map((node) => (
        <TreeNode key={node.id} node={node} depth={0} expanded={expanded} hasSelectHandler={hasSelectHandler} onToggle={toggle} />
      ))}
    </List>
  );
}

function TreeNode({
  node,
  depth,
  expanded,
  hasSelectHandler,
  onToggle,
}: {
  node: HubTreeNode;
  depth: number;
  expanded: Set<string>;
  hasSelectHandler: boolean;
  onToggle: (node: HubTreeNode) => void;
}) {
  const childCount = node.children?.length ?? 0;
  const open = expanded.has(node.id);
  const rowIsActionable = childCount > 0 || hasSelectHandler;
  const Icon = childCount > 0 && open ? ExpandMoreIcon : ChevronRightIcon;

  return (
    <Box>
      <ListItemButton
        disabled={!rowIsActionable}
        onClick={() => onToggle(node)}
        sx={{
          minHeight: 38,
          pl: 0.8 + depth * 2,
          pr: 1,
          borderRadius: `${hubTokens.radius.compact}px`,
          color: hubTokens.colors.textSoft,
          cursor: rowIsActionable ? "pointer" : "default",
          "&:hover": { backgroundColor: rowIsActionable ? "rgba(255,255,255,0.045)" : "transparent" },
          "&.Mui-disabled": {
            opacity: 1,
            color: hubTokens.colors.textSoft,
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
          {node.children!.map((child) => (
            <TreeNode key={child.id} node={child} depth={depth + 1} expanded={expanded} hasSelectHandler={hasSelectHandler} onToggle={onToggle} />
          ))}
        </Collapse>
      ) : null}
    </Box>
  );
}
