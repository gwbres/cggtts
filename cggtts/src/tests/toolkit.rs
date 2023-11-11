use crate::prelude::CGGTTS;
use rand::{distributions::Alphanumeric, Rng};

pub fn cmp_dut_model(dut: &CGGTTS, model: &CGGTTS) {
    assert_eq!(dut.version, model.version, "wrong version");
    assert_eq!(dut.release_date, model.release_date, "wrong release date");
    assert_eq!(dut.station, model.station, "bad station name");
    assert_eq!(dut.rcvr, model.rcvr, "bad receiver data");
    assert_eq!(dut.nb_channels, model.nb_channels, "bad receiver channels");
    assert_eq!(
        dut.reference_time, model.reference_time,
        "bad reference time"
    );
    assert_eq!(
        dut.apc_coordinates, model.apc_coordinates,
        "bad apc coordinates"
    );
    assert_eq!(dut.comments, model.comments, "wrong comments content");
    assert_eq!(dut.delay, model.delay, "bad delay values");
}

/*
 * Tool to generate random names when we need to produce a file
 */
pub fn random_name(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}
