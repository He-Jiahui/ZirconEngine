import SearchIcon from "@mui/icons-material/Search";
import { InputAdornment, TextField } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubSearchFieldProps {
  value: string;
  placeholder: string;
  ariaLabel?: string;
  compact?: boolean;
  onChange: (value: string) => void;
}

export function HubSearchField({ value, placeholder, ariaLabel = placeholder, compact = false, onChange }: HubSearchFieldProps) {
  return (
    <TextField
      value={value}
      placeholder={placeholder}
      size="small"
      onChange={(event) => onChange(event.target.value)}
      slotProps={{
        input: {
          startAdornment: (
            <InputAdornment position="start">
              <SearchIcon sx={{ color: hubTokens.colors.textSoft, fontSize: 22 }} />
            </InputAdornment>
          ),
        },
        htmlInput: { "aria-label": ariaLabel },
      }}
      sx={{
        width: "100%",
        minWidth: 0,
        maxWidth: "100%",
        "@media (min-width: 761px)": { width: compact ? 260 : 307 },
        "& .MuiOutlinedInput-root": {
          height: compact ? 36 : 47,
          minWidth: 0,
          color: hubTokens.colors.text,
          borderColor: compact ? hubTokens.colors.lineStrong : "rgba(45,212,207,0.92)",
          boxShadow: compact ? "none" : hubTokens.shadows.accent,
          "& fieldset": {
            borderColor: compact ? hubTokens.colors.lineStrong : "rgba(45,212,207,0.92)",
          },
        },
        "& input": { minWidth: 0 },
        "& input::placeholder": {
          color: hubTokens.colors.textMuted,
          opacity: 1,
        },
      }}
    />
  );
}
