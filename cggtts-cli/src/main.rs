mod cli;
use cli::Cli;

mod plot;
use plot::PlotContext;

mod processing;

use std::io::Write;
use std::path::Path;
use std::process::Command;

#[macro_use]
extern crate log;

use env_logger::{Builder, Target};
use walkdir::WalkDir;

use cggtts::prelude::CGGTTS;

use itertools::Itertools;

#[cfg(target_os = "linux")]
pub fn open_with_web_browser(path: &str) {
    let web_browsers = vec!["firefox", "chromium"];
    for browser in web_browsers {
        let child = Command::new(browser).args([path]).spawn();
        if child.is_ok() {
            return;
        }
    }
}

#[cfg(target_os = "macos")]
pub fn open_with_web_browser(path: &str) {
    Command::new("open")
        .args(&[path])
        .output()
        .expect("open() failed, can't open HTML content automatically");
}

#[cfg(target_os = "windows")]
pub fn open_with_web_browser(path: &str) {
    Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"start {}"#, path))
        .output()
        .expect("failed to open generated HTML content");
}

fn load_files(cli: &Cli) -> Vec<CGGTTS> {
    let mut pool = Vec::<CGGTTS>::new();
    for filepath in cli.input_files() {
        let cggtts = CGGTTS::from_file(filepath);
        if cggtts.is_ok() {
            pool.push(cggtts.unwrap());
            info!("loaded \"{}\"", filepath);
        } else {
            warn!(
                "failed to load \"{}\" - {}",
                filepath,
                cggtts.err().unwrap()
            );
        }
    }
    for dir in cli.input_directories() {
        let walkdir = WalkDir::new(dir).max_depth(5);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let filepath = entry.path().to_string_lossy().to_string();
                let cggtts = CGGTTS::from_file(&filepath);
                if cggtts.is_ok() {
                    pool.push(cggtts.unwrap());
                    info!("loaded \"{}\"", filepath);
                } else {
                    warn!(
                        "failed to load \"{}\" - {}",
                        filepath,
                        cggtts.err().unwrap()
                    );
                }
            }
        }
    }
    pool
}

pub fn main() {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    let cli = Cli::new();
    let mut plot_ctx = PlotContext::new();

    let quiet = cli.quiet();

    let workspace_path = match cli.workspace() {
        Some(w) => Path::new(w).to_path_buf(),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("workspace")
            .to_path_buf(),
    };

    let pool = load_files(&cli);

    if cli.identification() {
        for p in pool {
            info!("STATION          \"{}\"", p.station);
            info!("NUMBER OF TRACKS  {}", p.tracks.len());
            info!(
                "CODES            {:?}",
                p.tracks()
                    .map(|trk| trk.frc.clone())
                    .unique()
                    .collect::<Vec<_>>()
            );
            info!(
                "SV               {:?}",
                p.tracks()
                    .map(|trk| trk.sv.to_string())
                    .unique()
                    .collect::<Vec<_>>()
            );
        }
        return;
    }

    if pool.len() == 1 {
        processing::single_clock(&pool[0], &mut plot_ctx);
    } else {
        processing::clock_comparison(&workspace_path, &pool, &mut plot_ctx);
    }

    /*
     * Render graphs
     */
    let html_path = workspace_path.join("graphs.html");
    let html_path = html_path.to_str().unwrap();

    let mut fd = std::fs::File::create(html_path)
        .unwrap_or_else(|_| panic!("failed to generate graphs \"{}\"", &html_path));
    write!(fd, "{}", plot_ctx.to_html()).expect("failed to render graphs");
    info!("graphs rendered in $WORKSPACE/graphs.html");

    if !quiet {
        open_with_web_browser(html_path);
    }
}
