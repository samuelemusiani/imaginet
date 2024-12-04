use std::{fs, process};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to file
    #[arg(short, long)]
    path: String,
}

mod vde;
mod config;
mod executor;

fn main() {
    let args = Args::parse();

    let file = fs::read_to_string(args.path);

    match file {
        Err(e) => {
            eprintln!("Error opening file {}", e);
            process::exit(1);
        }
        Ok(file) => {
            println!("{file}");
            let c = config::Config::from_string(&file);

            let t = config_to_vde_topology(c);

            executor::start(t).unwrap();
        }
    } 
}

fn config_to_vde_topology(c: config::Config) -> vde::Topology {
    let mut t = vde::Topology::new();

    if let Some(sws) = c.switch {
        for sw in sws {
            let s = vde::Switch::new(sw.name);
            t.add_switch(s);
        }
    }

    return t;
}
