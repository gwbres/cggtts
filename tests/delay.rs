use rinex::Constellation;
use cggtts::delay::SystemDelay;
use cggtts::{Delay, CalibratedDelay};

#[cfg(test)]
mod delay {
    use super::*;
    
    #[test]
    fn test_delay() {
        let delay = Delay::Reference(10.0);
        assert_eq!(delay.value(), 10.0); 
        assert_eq!(delay.value_seconds(), 10.0E-9);
        assert_eq!(delay == Delay::Reference(10.0), true);
        assert_eq!(delay != Delay::Reference(10.1), true);
        assert_eq!(delay != Delay::Internal(10.0), true);
        let d = delay.add_value(20.0);
        assert_eq!(delay.value() +20.0, d.value());
    }

    #[test]
    fn test_calibrated_delay() {
        let delay = CalibratedDelay::new(Delay::Reference(10.0), None);
        assert_eq!(delay.trusted(), false);
        let mut delay = delay.with_constellation(Constellation::GPS);
        assert_eq!(delay.trusted(), true);
        delay.add_value(20.0);
        assert_eq!(delay.value(), 30.0);
    }

    #[test]
    fn test_system_delay() {
        let mut system_delay = SystemDelay::new();
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::Internal(10.0_f64),
                constellation: Constellation::GPS,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 10.0);
        assert_eq!(system_delay.value_seconds(), 10.0E-9);
        
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::Internal(20.0_f64),
                constellation: Constellation::GPS,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 30.0);
        
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::Reference(50.0_f64),
                constellation: Constellation::GPS,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 80.0);

        // discarded add: does not match previous constellation
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::Reference(10.0_f64),
                constellation: Constellation::Glonass,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 80.0);
        
        // permitted add: Mixed against GPS constellation 
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::Reference(10.0_f64),
                constellation: Constellation::Mixed,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), true);
        assert_eq!(system_delay.value(), 90.0);
        
        // permitted add: Mixed against GPS constellation 
        system_delay.add_delay(
            CalibratedDelay {
                delay: Delay::RfCable(1.0_f64),
                constellation: Constellation::Mixed,
                info: None,
            }
        );
        assert_eq!(system_delay.trusted(), false);
        assert_eq!(system_delay.value(), 91.0);
    }
}
