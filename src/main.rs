use anyhow::{Context, Result};
use clap::Parser;
use env_logger;
use home;
use log;
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

    #[arg(short, long, action = clap::ArgAction::Count, help = "Verbosity level. Can be used multiple times for increased verbosity")]
    pub verbose: u8,
}

#[derive(Parser, Debug)]
enum Commands {
    #[command(subcommand)]
    Add(AddSubcommands),

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

    #[command(about = "Execute a command in a device")]
    Exec {
        /// Name of the device in which to execute the command
        device: String,

        /// Command to execute with arguments
        command: Vec<String>,
    },

    #[command(about = "Remove a device from the topology")]
    Rm {
        /// Name of the device
        device: String,
    },

    #[command(about = "Start a topology")]
    Start {
        /// List of device names to start
        devices: Option<Vec<String>>,
    },

    #[command(about = "Status of running topology")]
    Status {
        /// List of device names to get status
        devices: Option<Vec<String>>,
    },

    #[command(about = "Stop a topology")]
    Stop {
        /// List of device names to stop
        devices: Option<Vec<String>>,
    },
}

#[derive(Parser, Debug)]
enum AddSubcommands {
    #[command(about = "Add a namespace to the current topology")]
    Namespace {
        /// Name of the namespace. Must be unique in all the topology
        name: String,

        /// List of interfaces for the namespace. Each interface must start with --iface
        /// and should have the following format: --iface <name> <ip> <endpoint> [<port>]
        #[clap(verbatim_doc_comment)]
        interfaces: Vec<String>,
    },

    #[command(about = "Add a switch to the current topology")]
    Switch {
        /// Name of the switch. Must be unique in all the topology
        name: String,

        #[arg(short, long, help = "Set number of ports for the switch")]
        ports: Option<u32>,

        #[arg(short = 'd', long, help = "Set the switch to be a hub")]
        hub: bool,

        #[arg(short, long, help = "Load config from file", value_name = "PATH")]
        config: Option<String>,
    },

    #[command(about = "Add a connection to the current topology")]
    Connection {
        /// Name of the connection. Must be unique in all the topology
        name: String,

        /// Name of the first endpoint
        a: String,

        #[arg(long, help = "Port number on endpoint A", value_name = "PORT")]
        port_a: Option<u32>,

        /// Name of the second endpoint
        b: String,

        #[arg(long, help = "Port number on endpoint A", value_name = "PORT")]
        port_b: Option<u32>,

        #[arg(
            short,
            long,
            help = "Make the connection with wirefilter",
            group = "wr"
        )]
        wirefilter: bool,

        #[arg(
            short,
            long,
            help = "Load config from file. Only valid if wirefilter is specified",
            requires = "wirefilter",
            value_name = "PATH"
        )]
        config: Option<String>,
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

    env_logger::Builder::new()
        .target(env_logger::Target::Stderr)
        .filter_level(match args.verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .format_timestamp(None)
        .init();

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
            Commands::Create { config } => topology_create(opts, config)?,
            Commands::Start { devices } => executor::topology_start(opts, devices)?,
            Commands::Status { devices } => executor::topology_status(opts, devices)?,
            Commands::Stop { devices } => executor::topology_stop(&opts, devices)?,
            Commands::Attach { device, inline } => executor::topology_attach(opts, device, inline)?,
            Commands::Exec { device, command } => executor::topology_exec(opts, device, command)?,
            Commands::Add(d) => {
                let mut t = executor::get_topology(&opts).context("Getting topology")?;
                match d {
                    AddSubcommands::Namespace { name, interfaces } => {
                        // Interface parsing
                        let mut tmp: Vec<Vec<String>> = Vec::new();

                        if interfaces[0] != "--iface" {
                            anyhow::bail!("Interface definition must start with --iface");
                        }

                        for i in interfaces.iter() {
                            if i == "--iface" {
                                tmp.push(Vec::new());
                            } else {
                                tmp.last_mut()
                                    .ok_or(anyhow::anyhow!("Empty vector"))?
                                    .push(i.clone());
                            }
                        }

                        for (n, i) in tmp.iter().enumerate() {
                            if i.len() < 2 || i.len() > 4 {
                                anyhow::bail!(
                                    "Interface {n} definition must have between 2 and 4 elements"
                                );
                            }
                        }

                        let mut real_interfaces: Vec<config::NSInterface> = Vec::new();
                        for i in tmp.iter() {
                            let name = i[0].clone();
                            let ip = i[1].clone();
                            let endpoint = config::Endpoint {
                                name: i[2].clone(),
                                port: if i.len() == 4 {
                                    Some(i[3].clone().parse()?)
                                } else {
                                    None
                                },
                            };

                            let inter = config::NSInterface { name, ip, endpoint };
                            inter
                                .checks()
                                .context(format!("Checking interface {}", i[0]))?;
                            real_interfaces.push(inter);
                        }

                        let mut ns = vde::Namespace::new(name);
                        for i in real_interfaces {
                            let endp = vde::calculate_endpoint_type(&t, &i.endpoint.name);
                            let ni = vde::NSInterface::new(
                                i.name.clone(),
                                i.ip.clone(),
                                endp,
                                i.endpoint.port,
                            );
                            ns.add_interface(ni);
                        }

                        t.add_namespace(ns)
                            .context("Adding namespace to topology")?;
                    }
                    AddSubcommands::Switch {
                        name,
                        ports,
                        hub,
                        config,
                    } => {
                        let mut s = vde::Switch::new(name);

                        if let Some(config) = config {
                            let c = fs::read_to_string(config).context("Config file not found")?;
                            c.lines().for_each(|l| s.add_config(l.to_owned()));
                        }

                        if let Some(ports) = ports {
                            s.set_ports(ports);
                        }

                        if hub {
                            s.set_hub(hub);
                        }

                        t.add_switch(s).context("Adding switch to topology")?;
                    }
                    AddSubcommands::Connection {
                        name,
                        a,
                        port_a,
                        b,
                        port_b,
                        wirefilter,
                        config,
                    } => {
                        let endp_a = vde::calculate_endpoint_type(&t, &a);
                        let endp_b = vde::calculate_endpoint_type(&t, &b);
                        let mut conn = vde::Connection::new(
                            name,
                            endp_a,
                            port_a,
                            endp_b,
                            port_b,
                            Some(wirefilter),
                        );

                        if let Some(config) = config {
                            let conf =
                                fs::read_to_string(config).context("Config file not found")?;
                            conf.lines().for_each(|l| conn.add_config(l.to_owned()));
                        }

                        t.add_connection(conn)
                            .context("Adding connection to topology")?;
                    }
                }

                executor::write_topology(opts.clone(), t).context("Writing topology")?;
            }
            Commands::Rm { device } => {
                executor::topology_stop(&opts, Some(vec![device.clone()]))?;

                let mut t = executor::get_topology(&opts).context("Getting topology")?;
                t.remove_device(&device)
                    .context("Removing device from topology")?;
                executor::write_topology(opts.clone(), t).context("Writing topology")?;
            }
        },
        None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    };

    Ok(())
}

fn topology_create(opts: executor::Options, config: String) -> Result<()> {
    let file = fs::read_to_string(config).context("Reading config file")?;

    let c = config::Config::from_string(&file).context("Parsing config")?;

    let t = config_to_vde_topology(c).context("Converting config to vde topology")?;

    executor::write_topology(opts.clone(), t).context("Writing topology")?;

    println!("Topology created");

    Ok(())
}

fn config_to_vde_topology(c: config::Config) -> Result<vde::Topology> {
    let mut t = vde::Topology::new();

    if let Some(sws) = &c.switch {
        for sw in sws {
            let mut s = vde::Switch::new(sw.name.clone());

            if let Some(config) = &sw.config {
                let c = fs::read_to_string(config).context("Config file not found")?;
                c.lines().for_each(|l| s.add_config(l.to_owned()));
            }

            if let Some(ports) = sw.ports {
                s.set_ports(ports);
            }

            if let Some(hub) = sw.hub {
                s.set_hub(hub);
            }

            t.add_switch(s).context("Adding switch to topology")?;
        }
    }

    if let Some(nss) = &c.namespace {
        for ns in nss {
            log::debug!("Parsing namespace {}", ns.name);
            let mut n = vde::Namespace::new(ns.name.clone());
            for i in &ns.interfaces {
                let endp = vde::calculate_endpoint_type(&t, &i.endpoint.name);
                log::debug!(
                    "Adding interface {} to namespace {}. Ip: {}, endp: {}, port: {}",
                    i.name,
                    ns.name,
                    i.ip,
                    endp,
                    i.endpoint.port.unwrap_or(0)
                );
                let ni = vde::NSInterface::new(i.name.clone(), i.ip.clone(), endp, i.endpoint.port);
                n.add_interface(ni);
            }
            t.add_namespace(n).context("Adding namespace to topology")?;
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
                let conf = fs::read_to_string(config).context("Config file not found")?;
                conf.lines().for_each(|l| conn.add_config(l.to_owned()));
            }

            t.add_connection(conn)
                .context("Adding connection to topology")?;
        }
    }

    return Ok(t);
}

fn parse_config_file(file: &str) -> Result<Config> {
    let file = fs::read_to_string(file).context("Reading config file")?;
    let c = Config::from_string(&file).context("Parsing config")?;
    Ok(c)
}
