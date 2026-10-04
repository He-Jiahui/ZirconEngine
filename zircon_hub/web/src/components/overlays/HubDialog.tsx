import type { PropsWithChildren, ReactNode } from "react";
import { useId } from "react";
import { Dialog, DialogActions, DialogContent, DialogTitle } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubDialogProps extends PropsWithChildren {
  open: boolean;
  title: string;
  actions?: ReactNode;
  onClose: () => void;
}

export function HubDialog({ open, title, actions, onClose, children }: HubDialogProps) {
  const idSuffix = useId().replace(/:/g, "");
  const titleId = `hub-dialog-title-${idSuffix}`;
  const descriptionId = `hub-dialog-description-${idSuffix}`;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
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
      <DialogTitle id={titleId}>{title}</DialogTitle>
      <DialogContent id={descriptionId}>{children}</DialogContent>
      {actions ? <DialogActions>{actions}</DialogActions> : null}
    </Dialog>
  );
}
