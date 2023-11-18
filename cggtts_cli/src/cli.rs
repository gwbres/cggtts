use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("cggtts-cli")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("CGGTTS post processing and clock comparison tool")
                    .arg_required_else_help(true)
                    .color(ColorChoice::Always)
                    .arg(Arg::new("filepath")
                        .short('f')
                        .long("fp")
                        .value_name("FILE")
                        .action(ArgAction::Append)
                        .required_unless_present("directory")
                        .help("Input CGGTTS file. Usually you want to load two CGGTTS, one from the local clock, one from remote clock, but you can load as many as you want."))
                    .arg(Arg::new("directory")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .required_unless_present("filepath")
                        .help("Load CGGTTS files contained in directory."))
                    .arg(Arg::new("workspace")
                        .short('w')
                        .value_name("DIRECTORY")
                        .action(ArgAction::Append)
                        .help("Define custom workspace location (folder).
Default location is cggtts-cli/workspace.
Folder does not have to exist."))
                    .arg(Arg::new("id")
                        .short('i')
                        .action(ArgAction::SetTrue)
                        .help("Identify local and remote setups."))
                    .get_matches()
            },
        }
    }
    /// Returns list of input directories
    pub fn input_directories(&self) -> Vec<&String> {
        if let Some(fp) = self.matches.get_many::<String>("directory") {
            fp.collect()
        } else {
            Vec::new()
        }
    }
    /// Returns individual input filepaths
    pub fn input_files(&self) -> Vec<&String> {
        if let Some(fp) = self.matches.get_many::<String>("filepath") {
            fp.collect()
        } else {
            Vec::new()
        }
    }
    pub fn identification(&self) -> bool {
        self.matches.get_flag("id")
    }
    fn get_flag(&self, flag: &str) -> bool {
        self.matches.get_flag(flag)
    }
    pub fn workspace(&self) -> Option<&String> {
        self.matches.get_one::<String>("workspace")
    }
}
