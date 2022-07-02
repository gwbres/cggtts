use cggtts::delay::SystemDelay;
use cggtts::{Delay, CalibratedDelay};
use rinex::constellation::Constellation;

#[cfg(test)]
mod delay {
    use super::*;
    #[test]
    fn test_delay() {
        let delay = Delay::Internal(10.0);
        assert_eq!(delay.value(), 10.0); 
        assert_eq!(delay.value_seconds(), 10.0E-9);
        assert_eq!(delay == Delay::Internal(10.0), true);
        assert_eq!(delay == Delay::System(10.0), false);
        let d = delay.add_value(20.0);
        assert_eq!(d, Delay::Internal(30.0));
        assert_eq!(delay.value() +20.0, d.value());
    }
    #[test]
    fn test_calibrated_delay() {
        let delay = CalibratedDelay::new(Delay::System(10.0), None);
        assert_eq!(delay.trusted(), false);
        assert_eq!(delay.non_trusted(), true);
        let delay = delay.with_constellation(Constellation::GPS);
        assert_eq!(delay.trusted(), true);
        assert_eq!(delay.non_trusted(), false);
        
        let rhs = CalibratedDelay::new(Delay::System(5.0), None);
        let rhs = rhs.with_constellation(Constellation::Glonass);
        let delay = delay + rhs;
        assert_eq!(delay.trusted(), true);
        assert_eq!(delay.value(), 10.0);
        
        let rhs = CalibratedDelay::new(Delay::System(5.0), None);
        let rhs = rhs.with_constellation(Constellation::GPS);
        let delay = delay + rhs;
        assert_eq!(delay.trusted(), true);
        assert_eq!(delay.value(), 15.0);
        
        let rhs = CalibratedDelay::new(Delay::System(2.0), None);
        let delay = delay + rhs;
        assert_eq!(delay.trusted(), false);
        assert_eq!(delay.value(), 17.0);
    }

    #[test]
    fn test_system_delay() {
        let mut system_delay = SystemDelay::new();
        system_delay.rf_cable_delay = 10.0;
        system_delay.calib_delay = CalibratedDelay {
            info: None,
            constellation: Constellation::Glonass,
            delay: Delay::Internal(5.0),
        };
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 15.0);
    }        
}
