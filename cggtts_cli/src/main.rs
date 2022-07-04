//! Command line tool to parse and analyze `CGGTTS` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/cggtts>     
//! Homepage: <https://github.com/gwbres/cggtts>
use clap::App;
use clap::load_yaml;

/*use gnuplot::{Figure, Caption};
use gnuplot::{Color, PointSymbol, LineStyle, DashType};
use gnuplot::{PointSize, AxesCommon, LineWidth};*/

use cggtts::{Cggtts, Track};
use cggtts::track::{CommonViewClass};
use rinex::constellation::Constellation;

pub fn main () -> Result<(), Box<dyn std::error::Error>> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let filepaths : Vec<&str> = matches.value_of("filepath")
        .unwrap()
            .split(",")
            .collect();

    let header = matches.is_present("header");
    let tracks = matches.is_present("tracks");
    let bipm = matches.is_present("bipm");
    let bipm_compliance = matches.is_present("bipm-compliant");
    let single = matches.is_present("single");
    let dual = matches.is_present("dual");
    let iono = matches.is_present("ionospheric");
    let sv = matches.is_present("sv");
    let unique = matches.is_present("unique");
    
    for fp in filepaths.iter() {
        let cggtts = Cggtts::from_file(fp);
        if cggtts.is_err() {
            println!("Failed to parse file \"{}\"", fp);
            continue;
        }
        let cggtts = cggtts.unwrap();
        if header {
            let mut c = cggtts.clone();
            c.tracks = Vec::new();
            println!("{:#?}", c);
        }
        
        // from now on, tracks might be filtered
        /*if unique {
            let data : Vec<_> = cggtts.tracks
                .iter()
                .filter(|t| t.space_vehicule.prn != 99)
                .collect();
            tracks = data;
        }*/
        
        if tracks {
            println!("{:#?}", cggtts.tracks);
        }
        if bipm_compliance {
            println!("{}", cggtts.follows_bipm_specs())
        }
        if iono {
            let data : Vec<_> = cggtts.tracks
                .iter()
                .filter(|t| t.has_ionospheric_data())
                .map(|t| t.ionospheric)
                .flatten()
                .collect();
            println!("{:#?}", data);
        }
        if single {
            if let Some(t) = cggtts.tracks.first() {
                println!("{}", t.class == CommonViewClass::Single)
            } else {
                println!("{}", false);
            }
        }
        if dual {
            if let Some(t) = cggtts.tracks.first() {
                println!("{}", t.class == CommonViewClass::Multiple)
            } else {
                println!("{}", false);
            }
        }
    }
    Ok(())
}
