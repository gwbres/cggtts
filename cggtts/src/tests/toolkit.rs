use crate::prelude::CGGTTS;
use rand::{distributions::Alphanumeric, Rng};

pub fn cmp_dut_model(dut: &CGGTTS, model: &CGGTTS) {
    assert_eq!(dut.version, model.version);
    assert_eq!(dut.release_date, model.release_date);
    assert_eq!(dut.station, model.station);
    assert_eq!(dut.rcvr, model.rcvr);
    assert_eq!(dut.nb_channels, model.nb_channels);
    assert_eq!(dut.reference_time, model.reference_time);
    assert_eq!(dut.apc_coordinates, model.apc_coordinates);
    assert_eq!(dut.comments, model.comments);
    assert_eq!(dut.delay, model.delay);
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
