import type { TextFieldProps } from "@mui/material";
import { TextField } from "@mui/material";

export interface HubTextFieldProps extends Omit<TextFieldProps, "variant" | "size"> {
  minWidth?: number;
}

export function HubTextField({ minWidth = 0, sx, ...props }: HubTextFieldProps) {
  return (
    <TextField
      {...props}
      variant="outlined"
      size="small"
      sx={[
        {
          minWidth,
          "& .MuiInputBase-root": { minHeight: 42 },
        },
        ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
      ]}
    />
  );
}
