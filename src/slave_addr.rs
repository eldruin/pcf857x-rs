//! Slave address definition

/// Possible slave addresses
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlaveAddr {
    /// Default slave address
    Default,
    /// Alternative slave address providing bit values for A2, A1 and A0
    Alternative(bool, bool, bool),
}

impl Default for SlaveAddr {
    /// Default slave address
    fn default() -> Self {
        SlaveAddr::Default
    }
}

impl SlaveAddr {
    pub(crate) fn addr(self, default: u8) -> u8 {
        match self {
            SlaveAddr::Default => default,
            SlaveAddr::Alternative(a2, a1, a0) => {
                default | ((a2 as u8) << 2) | ((a1 as u8) << 1) | a0 as u8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SlaveAddr;

    #[test]
    fn can_get_default_address() {
        let addr = SlaveAddr::default();
        assert_eq!(0b010_0000, addr.addr(0b010_0000));
    }

    #[test]
    fn can_generate_alternative_addresses() {
        let default = 0b010_0000;
        assert_eq!(
            0b010_0000,
            SlaveAddr::Alternative(false, false, false).addr(default)
        );
        assert_eq!(
            0b010_0001,
            SlaveAddr::Alternative(false, false, true).addr(default)
        );
        assert_eq!(
            0b010_0010,
            SlaveAddr::Alternative(false, true, false).addr(default)
        );
        assert_eq!(
            0b010_0100,
            SlaveAddr::Alternative(true, false, false).addr(default)
        );
        assert_eq!(
            0b010_0111,
            SlaveAddr::Alternative(true, true, true).addr(default)
        );
    }
}
