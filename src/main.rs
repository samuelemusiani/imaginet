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
            let c = config::Config::from_string(&file);

            let t = config_to_vde_topology(c);

            executor::start(t).unwrap();
        }
    } 
}

fn config_to_vde_topology(c: config::Config) -> vde::Topology {
    let mut t = vde::Topology::new();

    if let Some(sws) = &c.switch {
        for sw in sws {
            let mut s = vde::Switch::new(sw.name.clone());

            if let Some(config) = &sw.config {
                let c = fs::read_to_string(config).expect("Config file not found");
                c.lines().for_each(|l| s.add_config(l.to_owned()));
            }

            t.add_switch(s);
        }
    }

    if let Some(nss) = &c.namespace {
        for ns in nss {
            let mut n = vde::Namespace::new(ns.name.clone());
            for i in &ns.interfaces {
                let endp = vde::calculate_endpoint_type(&t, &i.endpoint);
                let ni = vde::NSInterface::new(i.name.clone(), i.ip.clone(), endp, i.port);
                n.add_interface(ni);
            }
            t.add_namespace(n);
        }
    }

    if let Some(conns) = &c.connections {
        for c in conns {
            let endp_a = vde::calculate_endpoint_type(&t, &c.a);
            let endp_b = vde::calculate_endpoint_type(&t, &c.b);
            let conn = vde::Connection::new(
                c.name.clone(), endp_a, endp_b, c.wirefilter);
            t.add_connection(conn);
        }
    }

    return t;
}
