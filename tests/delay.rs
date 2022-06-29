use rinex::Constellation;
use cggtts::{Delay, CalibratedDelay};

#[cfg(test)]
mod delay {
    use super::*;
    
    #[test]
    fn test_delay() {
        let delay = Delay::Reference(10.0);
        assert_eq!(delay.value(), 10.0); 
        assert_eq!(delay.value_seconds(), 10.0E-9)
    }

    #[test]
    fn test_calibrated_delay() {
        let delay = CalibratedDelay::new(Delay::Reference(10.0), None);
        assert_eq!(delay.trusted(), false);
        assert_eq!(delay.non_trusted(), true);
        let delay = delay.with_constellation(Constellation::GPS);
        assert_eq!(delay.trusted(), true);
        assert_eq!(delay.non_trusted(), false);
    }
}
