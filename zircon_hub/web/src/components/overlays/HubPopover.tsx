import type { PropsWithChildren } from "react";
import { Box, Popover } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubPopoverProps extends PropsWithChildren {
  anchorEl: HTMLElement | null;
  open: boolean;
  width?: number;
  align?: "left" | "right";
  onClose: () => void;
}

export function HubPopover({ anchorEl, open, width = 340, align = "left", onClose, children }: HubPopoverProps) {
  return (
    <Popover
      anchorEl={anchorEl}
      open={open}
      onClose={onClose}
      anchorOrigin={{
        vertical: "bottom",
        horizontal: align,
      }}
      transformOrigin={{
        vertical: "top",
        horizontal: align,
      }}
      slotProps={{
        paper: {
          sx: {
            mt: 1,
            width,
            maxWidth: "calc(100vw - 32px)",
            color: hubTokens.colors.text,
            backgroundImage: "none",
            backgroundColor: "rgba(25,29,29,0.98)",
            border: `1px solid ${hubTokens.colors.lineStrong}`,
            borderRadius: `${hubTokens.radius.panel}px`,
            boxShadow: "0 24px 60px rgba(0,0,0,0.46), 0 0 0 1px rgba(45,212,207,0.08)",
            overflow: "hidden",
          },
        },
      }}
    >
      <Box sx={{ p: 1.2 }}>{children}</Box>
    </Popover>
  );
}
