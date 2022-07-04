//! Command line tool to parse and analyze `CGGTTS` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/cggtts>     
//! Homepage: <https://github.com/gwbres/cggtts>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;

/*use gnuplot::{Figure, Caption};
use gnuplot::{Color, PointSymbol, LineStyle, DashType};
use gnuplot::{PointSize, AxesCommon, LineWidth};*/

use cggtts::{Cggtts, Track};
use rinex::constellation::Constellation;

pub fn main () -> Result<(), Box<dyn std::error::Error>> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let pretty = matches.is_present("pretty");
    let filepaths : Vec<&str> = matches.value_of("filepath")
        .unwrap()
            .split(",")
            .collect();

    let header = matches.is_present("header");
    let bipm = matches.is_present("bipm");
    let cggtts = Cggtts::from_file(filepaths[0])
        .unwrap();
    
    if header {
        let mut c = cggtts.clone();
        c.tracks = Vec::new();
        println!("{:#?}", c);
    }

    Ok(())
}
