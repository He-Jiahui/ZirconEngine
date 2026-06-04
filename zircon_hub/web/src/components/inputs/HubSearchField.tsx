import SearchIcon from "@mui/icons-material/Search";
import { InputAdornment, TextField } from "@mui/material";
import { hubTokens } from "../../theme/tokens";

export interface HubSearchFieldProps {
  value: string;
  placeholder: string;
  compact?: boolean;
  onChange: (value: string) => void;
}

export function HubSearchField({ value, placeholder, compact = false, onChange }: HubSearchFieldProps) {
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
      }}
      sx={{
        width: compact ? 260 : 307,
        maxWidth: "100%",
        "& .MuiOutlinedInput-root": {
          height: compact ? 36 : 47,
          color: hubTokens.colors.text,
          borderColor: compact ? hubTokens.colors.lineStrong : "rgba(45,212,207,0.92)",
          boxShadow: compact ? "none" : hubTokens.shadows.accent,
          "& fieldset": {
            borderColor: compact ? hubTokens.colors.lineStrong : "rgba(45,212,207,0.92)",
          },
        },
        "& input::placeholder": {
          color: hubTokens.colors.textMuted,
          opacity: 1,
        },
      }}
    />
  );
}
