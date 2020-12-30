#[cfg(test)]
mod player_test {
    use crate::player::meter::Meter;
    use crate::player::meter::MeterKind::*;

    use crate::player::{Player, Uuid};

    #[test]
    fn player_test_uuid() {
        assert_ne!(Player::new("").uuid(), 0);
    }

    #[test]
    fn test_meter_display() {
        let x = Meter(100, 100);
        assert_eq!(format!("{}", x), "[100 / 100]");
        let y = Hit(x);
        assert_eq!(format!("{}", y), "HIT: [100 / 100]");
    }
}
