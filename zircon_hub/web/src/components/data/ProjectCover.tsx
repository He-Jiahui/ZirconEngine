import { Box } from "@mui/material";
import { brandMark, coverById } from "../../data/hubData";
import { hubTokens } from "../../theme/tokens";

export interface ProjectCoverProps {
  coverId: string;
  size?: "card" | "thumb";
}

export function ProjectCover({ coverId, size = "card" }: ProjectCoverProps) {
  const coverUrl = coverById[coverId] ?? coverById.elysium;
  const thumb = size === "thumb";

  return (
    <Box
      sx={{
        position: "relative",
        width: thumb ? 30 : "100%",
        height: thumb ? 30 : "100%",
        overflow: "hidden",
        borderRadius: thumb ? `${hubTokens.radius.thumb}px` : "inherit",
        backgroundColor: hubTokens.colors.coverBackdrop,
        border: "1px solid rgba(255,255,255,0.08)",
      }}
    >
      <Box
        component="img"
        src={coverUrl}
        alt=""
        sx={{
          width: "100%",
          height: "100%",
          display: "block",
          objectFit: "cover",
          filter: "saturate(0.98) contrast(0.98) brightness(0.98)",
        }}
      />
      <Box
        sx={{
          position: "absolute",
          inset: 0,
          background:
            "linear-gradient(90deg, rgba(255,255,255,0.035), transparent 18%, transparent 82%, rgba(0,0,0,0.12)), linear-gradient(180deg, transparent 52%, rgba(0,0,0,0.16))",
          pointerEvents: "none",
        }}
      />
      {!thumb ? (
        <Box
          component="img"
          src={brandMark}
          alt=""
          sx={{
            position: "absolute",
            left: 14,
            bottom: 14,
            width: 30,
            height: 30,
            borderRadius: `${hubTokens.radius.brandMark}px`,
            backgroundColor: "rgba(10,20,22,0.72)",
            p: "4px",
          }}
        />
      ) : null}
    </Box>
  );
}
