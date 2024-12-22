use std::{fs, process};
use anyhow::{Context, Result};

use clap::Parser;

/// Create and manage VDE topologies
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help=true)]
struct Args {

    #[arg(short, long, help = "Base directory for all imaginet files")]
    pub base_dir: Option<String>,

    #[arg(short, long, help = "Terminal to open when starting or attaching to a device")]
    pub terminal: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

}

#[derive(Parser, Debug)]
enum Commands {
    #[command(about = "Attach to a device in a topology")]
    Attach {
        /// Name of the device to attach to
        device: String
    },

    #[command(about = "Create a topology")]
    Create {
        /// Path to configuration file
        config: String
    },

    #[command(about = "Start a topology")]  
    Start {
    },

    #[command(about = "Status of running topology")]  
    Status {
    },

    #[command(about = "Stop a topology")]
    Stop {
    },
}

mod vde;
mod config;
mod executor;

fn main() -> Result<()> {
    let args = Args::parse();

    // Options for the executor
    let opts = executor::Options {
        // Terminal to open when starting or attaching to a device
        terminal: if let Some(term) = args.terminal {
            term
        } else {
            get_terminal()
        },
        // Working directory for all the imaginet files
        working_dir: if let Some(dir) = args.base_dir {
            dir
        } else {
            "/tmp/imnet".to_owned()
        },
    };

    match args.command {
        Some(command) => match command {
            Commands::Create { config } => create(opts, config)?,
            Commands::Start {} => executor::topology_start(opts)?,
            Commands::Status {} => executor::topology_status(opts)?,
            Commands::Stop {} => executor::topology_stop(opts)?,
            Commands::Attach { device } => executor::attach(opts, device)?,
        } None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    };

    Ok(())
}

fn create(opts: executor::Options, config: String) -> Result<()> {
    let file = fs::read_to_string(config)
        .context("Reading config file")?;

    let c = config::Config::from_string(&file)
        .context("Parsing config")?;

    let t = config_to_vde_topology(c);

    executor::write_topology(opts, t)?;

    Ok(())
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

            if let Some(ports) = sw.ports {
                s.set_ports(ports);
            }

            t.add_switch(s);
        }
    }

    if let Some(nss) = &c.namespace {
        for ns in nss {
            let mut n = vde::Namespace::new(ns.name.clone());
            for i in &ns.interfaces {
                let endp = vde::calculate_endpoint_type(&t, &i.endpoint.name);
                let ni = vde::NSInterface::new(i.name.clone(), i.ip.clone(), endp, i.endpoint.port);
                n.add_interface(ni);
            }
            t.add_namespace(n);
        }
    }

    if let Some(conns) = &c.connections {
        for c in conns {
            let endp_a = vde::calculate_endpoint_type(&t, &c.endpoint_a.name);
            let port_a = c.endpoint_a.port;
            let endp_b = vde::calculate_endpoint_type(&t, &c.endpoint_b.name);
            let port_b = c.endpoint_b.port;
            let conn = vde::Connection::new(
                c.name.clone(), endp_a, port_a, endp_b, port_b, c.wirefilter);
            t.add_connection(conn);
        }
    }

    return t;
}

fn get_terminal() -> String {
    let term = std::env::var("TERM");
    match term {
        Ok(t) => t,
        Err(_) => "foot".to_owned(),
    }
}
