use rinex::{Constellation, Sv};
use cggtts::{Track, track::CommonViewClass, track::GlonassChannel};

#[cfg(test)]
mod track {
    use super::*;
    #[test]
    fn constructor () {
        let track = Track::default();
        assert_eq!(track.duration.as_secs(), 780);
        assert_eq!(track.elevation, 0.0);
        assert_eq!(track.azimuth, 0.0);
        assert_eq!(track.refsv, 0.0);
        assert_eq!(track.srsv, 0.0);
        assert_eq!(track.refsys, 0.0);
        assert_eq!(track.srsys, 0.0);
        assert_eq!(track.ionospheric, None);
        assert_eq!(track.space_vehicule, None);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.space_vehicule_combination(), true);
    }

    fn basic() {
        let track = Track::default();
        let t = track
            .with_duration(std::time::Duration::from_secs(10))
            .with_azimuth(180.0)
            .with_elevation(90.0);
        assert_eq!(t.follows_bipm_specs(), false);
        assert_eq!(t.azimuth, 180.0);
        assert_eq!(t.elevation, 180.0);
    }
}
