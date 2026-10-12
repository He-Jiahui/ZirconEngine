use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

impl HostActivityRailPointerSide {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(format!("Unknown activity rail side {value}")),
        }
    }
}
