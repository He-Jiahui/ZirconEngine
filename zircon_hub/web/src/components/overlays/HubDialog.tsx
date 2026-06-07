import type { PropsWithChildren, ReactNode } from "react";
import { Dialog, DialogActions, DialogContent, DialogTitle } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubDialogProps extends PropsWithChildren {
  open: boolean;
  title: string;
  actions?: ReactNode;
  onClose: () => void;
}

export function HubDialog({ open, title, actions, onClose, children }: HubDialogProps) {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      slotProps={{
        paper: {
          sx: {
            border: `1px solid ${hubTokens.colors.lineStrong}`,
            backgroundImage: "none",
            backgroundColor: "rgba(28,28,28,0.98)",
          },
        },
      }}
    >
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>{children}</DialogContent>
      {actions ? <DialogActions>{actions}</DialogActions> : null}
    </Dialog>
  );
}
