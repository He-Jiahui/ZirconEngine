import type { ButtonProps, SxProps, Theme } from "@mui/material";
import { Button } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export type HubButtonTone = "primary" | "secondary" | "tertiary" | "danger";

export interface HubButtonProps extends Omit<ButtonProps, "variant"> {
  tone?: HubButtonTone;
}

const toneStyles: Record<HubButtonTone, SxProps<Theme>> = {
  primary: {
    color: "#eefefe",
    backgroundColor: hubTokens.colors.accentDim,
    borderColor: "rgba(45, 212, 207, 0.48)",
    "&:hover": {
      backgroundColor: "rgba(17, 127, 124, 0.92)",
      borderColor: "rgba(45, 212, 207, 0.68)",
    },
  },
  secondary: {
    color: hubTokens.colors.text,
    backgroundColor: "rgba(32,32,32,0.82)",
    borderColor: hubTokens.colors.lineStrong,
    "&:hover": {
      backgroundColor: "#292929",
      borderColor: "rgba(255,255,255,0.22)",
    },
  },
  tertiary: {
    color: hubTokens.colors.accent,
    backgroundColor: "transparent",
    borderColor: "transparent",
    "&:hover": {
      backgroundColor: "rgba(33,213,207,0.08)",
      borderColor: "transparent",
    },
  },
  danger: {
    color: "#ffd8d5",
    backgroundColor: "rgba(120,25,25,0.54)",
    borderColor: "rgba(245,111,102,0.48)",
    "&:hover": {
      backgroundColor: "rgba(142,30,29,0.72)",
    },
  },
};

export function HubButton({ tone = "secondary", sx, ...props }: HubButtonProps) {
  return (
    <Button
      {...props}
      variant="contained"
      sx={[
        {
          border: "1px solid",
          px: 2.5,
          minWidth: 0,
        },
        toneStyles[tone],
        ...asSxArray(sx),
      ]}
    />
  );
}

function asSxArray(sx?: SxProps<Theme>) {
  if (!sx) {
    return [];
  }
  return Array.isArray(sx) ? sx : [sx];
}
