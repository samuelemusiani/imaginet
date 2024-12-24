use anyhow::{Context, Result};
use clap::Parser;
use home;
use std::{fs, process};

mod config;
mod executor;
mod vde;

/// Create and manage VDE topologies
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help=true)]
struct Args {
    #[arg(short, long, help = "Base directory for all imaginet files")]
    pub base_dir: Option<String>,

    #[arg(
        short,
        long,
        help = "Terminal to open when starting or attaching to a device"
    )]
    pub terminal: Option<String>,

    #[arg(short, long, help = "Path to configuration file")]
    pub conifg: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    #[command(about = "Attach to a device in a topology")]
    Attach {
        #[arg(short, long, help = "Attach inline: do not open a new terminal")]
        inline: bool,

        /// Name of the device to attach to
        device: String,
    },

    #[command(about = "Create a topology")]
    Create {
        /// Path to configuration file
        config: String,
    },

    #[command(about = "Start a topology")]
    Start {
        /// List of device names to start
        devices: Option<Vec<String>>,
    },

    #[command(about = "Status of running topology")]
    Status {},

    #[command(about = "Stop a topology")]
    Stop {
        /// List of device names to start
        devices: Option<Vec<String>>,
    },
}

#[derive(serde::Deserialize)]
struct Terminal {
    path: String,
    args: Vec<String>,
}

/// This is the config struct for imaginet. Not to be confused with the
/// config module and his config struct (config::Config)
#[derive(serde::Deserialize)]
struct Config {
    terminal: Option<Terminal>,
}

impl Config {
    fn new() -> Self {
        Config { terminal: None }
    }

    fn from_string(file: &str) -> Result<Config> {
        let c = serde_yaml::from_str::<Self>(&file).context("Deserialize config file failed")?;
        Ok(c)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let conf = if let Some(config) = args.conifg {
        parse_config_file(&config)
    } else {
        let home = home::home_dir().context("Getting home directory")?;
        let config_file = home.join(".config").join("imaginet").join("config.yaml");
        if config_file.exists() {
            parse_config_file(config_file.to_str().unwrap())
        } else {
            Ok(Config::new())
        }
    }
    .context("Getting config")?;

    // Options for the executor
    let opts = executor::Options {
        // Terminal to open when starting or attaching to a device. The cli argument
        // has precedence over the config file, which has precedence over the TERM env
        // variable
        terminal: if let Some(term) = args.terminal {
            term
        } else if let Some(term) = &conf.terminal {
            term.path.clone()
        } else {
            std::env::var("TERM")
                .context("Could not find a terminal emulator in TERM environment variable: {e}")?
        },

        // Some terminals require additional arguments in order to execute a program
        // different from the default shell. For example, the gnome-terminal requires
        // the `--` argument to separate the terminal arguments from the actual program
        terminal_args: if let Some(term) = conf.terminal {
            term.args
        } else {
            vec![]
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
            Commands::Start { devices } => executor::topology_start(opts, devices)?,
            Commands::Status {} => executor::topology_status(opts)?,
            Commands::Stop { devices } => executor::topology_stop(opts, devices)?,
            Commands::Attach { device, inline } => executor::attach(opts, device, inline)?,
        },
        None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    };

    Ok(())
}

fn create(opts: executor::Options, config: String) -> Result<()> {
    let file = fs::read_to_string(config).context("Reading config file")?;

    let c = config::Config::from_string(&file).context("Parsing config")?;

    let t = config_to_vde_topology(c);

    executor::write_topology(opts.clone(), t)?;

    let _ = executor::topology_status(opts);

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

            if let Some(hub) = sw.hub {
                s.set_hub(hub);
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
            let mut conn =
                vde::Connection::new(c.name.clone(), endp_a, port_a, endp_b, port_b, c.wirefilter);

            if let Some(config) = &c.config {
                let conf = fs::read_to_string(config).expect("Config file not found");
                conf.lines().for_each(|l| conn.add_config(l.to_owned()));
            }

            t.add_connection(conn);
        }
    }

    return t;
}

fn parse_config_file(file: &str) -> Result<Config> {
    let file = fs::read_to_string(file).context("Reading config file")?;
    let c = Config::from_string(&file).context("Parsing config")?;
    Ok(c)
}
