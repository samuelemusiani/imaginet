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
            let c: config::Config = serde_yaml::from_str(&file).unwrap();

            executor::run_net(c);
        }
    } 
}

