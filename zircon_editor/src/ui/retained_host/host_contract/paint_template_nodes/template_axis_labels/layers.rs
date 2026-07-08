const SCALE_LINK_CONNECTOR_OFFSET: i32 = 1;

pub(super) fn scale_link_connector_order(lobe_order: i32) -> i32 {
    lobe_order + SCALE_LINK_CONNECTOR_OFFSET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_link_connector_paints_above_lobes() {
        let lobes = 50;

        assert!(lobes < scale_link_connector_order(lobes));
    }
}
