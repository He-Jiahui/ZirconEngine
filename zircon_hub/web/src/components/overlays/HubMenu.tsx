import type { ReactNode } from "react";
import { Menu, MenuItem, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubMenuItem {
  id: string;
  label: string;
  icon?: ReactNode;
}

export interface HubMenuProps {
  anchorEl: HTMLElement | null;
  open: boolean;
  items: HubMenuItem[];
  onClose: () => void;
  onSelect: (id: string) => void;
}

export function HubMenu({ anchorEl, open, items, onClose, onSelect }: HubMenuProps) {
  return (
    <Menu
      anchorEl={anchorEl}
      open={open}
      onClose={onClose}
      slotProps={{
        list: { dense: true },
        paper: { sx: { minWidth: 188, mt: 1 } },
      }}
    >
      {items.map((item) => (
        <MenuItem
          key={item.id}
          onClick={() => {
            onSelect(item.id);
            onClose();
          }}
          sx={{ gap: 1.2, color: hubTokens.colors.textSoft }}
        >
          {item.icon}
          <Typography variant="body2">{item.label}</Typography>
        </MenuItem>
      ))}
    </Menu>
  );
}
