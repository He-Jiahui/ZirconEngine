import type { IconButtonProps, SxProps, Theme } from "@mui/material";
import { IconButton, Tooltip } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubIconButtonProps extends IconButtonProps {
  selected?: boolean;
  label: string;
  tooltip?: string;
}

export function HubIconButton({ selected = false, label, tooltip, sx, ...props }: HubIconButtonProps) {
  return (
    <Tooltip title={tooltip ?? label}>
      <span style={{ display: "inline-flex" }}>
        <IconButton
          {...props}
          aria-label={label}
          sx={[
            {
              width: 50,
              height: 42,
              color: selected ? "#eefefe" : hubTokens.colors.textSoft,
              backgroundColor: selected ? "rgba(9,94,91,0.56)" : "rgba(31,31,31,0.72)",
              border: `1px solid ${selected ? "rgba(45,212,207,0.48)" : hubTokens.colors.lineStrong}`,
              "&:hover": {
                color: hubTokens.colors.text,
                backgroundColor: selected ? "rgba(11,112,109,0.68)" : "#292929",
              },
              "&.Mui-disabled": {
                color: hubTokens.colors.textMuted,
                backgroundColor: "transparent",
                borderColor: "transparent",
              },
            },
            ...asSxArray(sx),
          ]}
        />
      </span>
    </Tooltip>
  );
}

function asSxArray(sx?: SxProps<Theme>) {
  if (!sx) {
    return [];
  }
  return Array.isArray(sx) ? sx : [sx];
}
