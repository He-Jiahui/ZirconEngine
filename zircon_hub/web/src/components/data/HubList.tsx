import type { ReactNode } from "react";
import { Box, List, ListItemButton, ListItemIcon, ListItemText, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubListItem {
  id: string;
  title: string;
  detail?: string;
  secondaryDetail?: string;
  meta?: string;
  icon?: ReactNode;
  selected?: boolean;
  disabled?: boolean;
}

export interface HubListProps {
  items: HubListItem[];
  onSelect?: (item: HubListItem) => void;
}

export function HubList({ items, onSelect }: HubListProps) {
  const hasSelectHandler = Boolean(onSelect);

  return (
    <List dense sx={{ display: "grid", gap: 0.7, p: 0 }}>
      {items.map((item) => {
        const itemDisabled = item.disabled || !hasSelectHandler;
        return (
          <ListItemButton
            key={item.id}
            selected={item.selected}
            disabled={itemDisabled}
            onClick={() => onSelect?.(item)}
            sx={{
              minHeight: item.secondaryDetail ? 64 : 48,
              px: 1.1,
              borderRadius: `${hubTokens.radius.compact}px`,
              border: `1px solid ${item.selected ? "rgba(45,212,207,0.34)" : hubTokens.colors.lineStrong}`,
              backgroundColor: item.selected ? "rgba(18,82,80,0.38)" : "rgba(32,32,32,0.54)",
              cursor: hasSelectHandler && !item.disabled ? "pointer" : "default",
              "&.Mui-selected, &.Mui-selected:hover": { backgroundColor: "rgba(18,82,80,0.46)" },
              "&.Mui-disabled": {
                opacity: item.disabled ? 0.52 : 1,
                color: item.disabled ? hubTokens.colors.textMuted : hubTokens.colors.text,
              },
            }}
          >
            {item.icon ? <ListItemIcon sx={{ minWidth: 34, color: hubTokens.colors.textSoft }}>{item.icon}</ListItemIcon> : null}
            <ListItemText
              primary={<Typography variant="body2" noWrap>{item.title}</Typography>}
              secondary={
                item.detail || item.secondaryDetail ? (
                  <Box sx={{ minWidth: 0, display: "grid", gap: 0.15 }}>
                    {item.detail ? <Typography variant="caption" noWrap>{item.detail}</Typography> : null}
                    {item.secondaryDetail ? (
                      <Typography variant="caption" noWrap sx={{ color: hubTokens.colors.textMuted }}>
                        {item.secondaryDetail}
                      </Typography>
                    ) : null}
                  </Box>
                ) : null
              }
              sx={{ minWidth: 0, my: 0 }}
            />
            {item.meta ? (
              <Typography variant="caption" noWrap sx={{ ml: 1, color: hubTokens.colors.textMuted }}>
                {item.meta}
              </Typography>
            ) : null}
          </ListItemButton>
        );
      })}
    </List>
  );
}
