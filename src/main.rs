use anyhow::{Context, Result};
use clap::Parser;
use env_logger;
use home;
use log;
use std::{fs, path::PathBuf, process, str::FromStr};
use vde::VdeConnProtocols;

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

    #[arg(short, long, help = "Path to global configuration file")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, action = clap::ArgAction::Count, help = "Verbosity level. Can be used multiple times for increased verbosity")]
    pub verbose: u8,
}

#[derive(Parser, Debug)]
enum Commands {
    #[command(subcommand, about = "Add a device to the current topology")]
    Add(AddSubcommands),

    #[command(about = "Attach to a device in the topology")]
    Attach {
        #[arg(short, long, help = "Attach inline: do not open a new terminal")]
        inline: bool,

        /// Name of the device to attach to
        device: String,
    },

    #[command(about = "Create a topology from a yaml configuration")]
    Create {
        /// Path to configuration file. If not provided, an empty topology is created
        config: Option<String>,

        #[arg(
            short,
            long,
            help = "Force the creation of a new topology, deleting the current one"
        )]
        force: bool,
    },

    #[command(about = "Stop and delete the current topology")]
    Clear {
        #[arg(
            short,
            long,
            help = "Force the deletion of the current topology, even if parsing is not possible. Devices could not be stopped"
        )]
        force: bool,
    },

    #[command(about = "Dump current raw configuration")]
    Dump {},

    #[command(about = "Execute a command in a device")]
    Exec {
        /// Name of the device in which to execute the command
        device: String,

        /// Command to execute with arguments
        command: Vec<String>,
    },

    #[command(about = "Import a topology from a raw configuration file (generated with dump)")]
    Import {
        /// Path to the topology file
        config: String,

        #[arg(
            short,
            long,
            help = "Force the import of a new topology, deleting the current one"
        )]
        force: bool,
    },

    #[command(about = "Remove a device from the topology")]
    Rm {
        /// Name of the device
        device: String,
    },

    #[command(about = "Start devices in the current topology")]
    Start {
        #[arg(
            short,
            long,
            help = r#"Start inline: do not open a new terminal and start/attach to the
process in the current one. If this option is specified you must
pass only a single device "#
        )]
        inline: bool,

        /// List of device names to start
        devices: Option<Vec<String>>,
    },

    #[command(about = "Status of running topology")]
    Status {
        /// List of device names to get status
        devices: Option<Vec<String>>,

        #[arg(short, long, action = clap::ArgAction::Count, help = "Verbosity level. Can be used multiple times for increased verbosity")]
        verbose: u8,
    },

    #[command(about = "Stop devices in the current topology")]
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
        /// and should have the following format: --iface <name> [ip] [gateway]
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

    #[command(about = "Add a cable to the current topology")]
    Cable {
        /// Name of the cable. Must be unique in all the topology
        name: String,

        /// Name of the first endpoint
        a: String,

        #[arg(long, help = "Port number on endpoint A", value_name = "PORT")]
        port_a: Option<String>,

        #[arg(long, help = "Open connection on endpoint A", value_name = "BOOL")]
        open_a: Option<bool>,

        /// Name of the second endpoint
        b: String,

        #[arg(long, help = "Port number on endpoint A", value_name = "PORT")]
        port_b: Option<String>,

        #[arg(long, help = "Open connection on endpoint B", value_name = "BOOL")]
        open_b: Option<bool>,

        #[arg(short, long, help = "Make the cable with wirefilter", group = "wr")]
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

    #[command(about = "Add a slirp connection to the current topology")]
    Slirp {
        /// Name of the slirp. Must be unique in all the topology
        name: String,
    },

    #[command(about = "Add a VXVDE connection to the current topology")]
    VXVDE {
        /// Name of the vxvde. Must be unique in all the topology
        name: String,

        /// Address of the vxvde connection
        #[arg(short, long, help = "Address of the vxvde connection")]
        addr: Option<String>,

        /// Port of the vxvde connection
        #[arg(short, long, help = "Port of the vxvde connection")]
        port: Option<u16>,
    },
}

#[derive(serde::Deserialize)]
struct Terminal {
    executable: String,
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

    let conf = if let Some(config) = args.config {
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
            term.executable.clone()
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
            Commands::Create { config, force } => topology_create(opts, config, force)?,
            Commands::Clear { force } => {
                let res = executor::topology_stop(&opts, None);
                if !force {
                    res?;
                } else {
                    if let Err(e) = res {
                        log::warn!("Error stopping topology: {e}. But continuing anyway. Some devices could not be stopped");
                    }
                }
                executor::clear_topology(&opts)?;
            }
            Commands::Dump {} => {
                let t = executor::get_topology(&opts).context("Getting topology")?;
                print!(
                    "{}",
                    t.to_string().context("Converting topology to string")?
                );
            }
            Commands::Import { config, force } => topology_import(opts, config, force)?,
            Commands::Start { devices, inline } => executor::topology_start(opts, devices, inline)?,
            Commands::Status { devices, verbose } => {
                executor::topology_status(opts, devices, verbose)?
            }
            Commands::Stop { devices } => executor::topology_stop(&opts, devices)?,
            Commands::Attach { device, inline } => executor::topology_attach(opts, device, inline)?,
            Commands::Exec { device, command } => executor::topology_exec(opts, device, command)?,
            Commands::Add(d) => {
                let mut t = executor::get_topology(&opts).context("Getting topology")?;
                match d {
                    AddSubcommands::Namespace { name, interfaces } => {
                        let real_interfaces =
                            parse_interfaces(interfaces).context("Parsing interfaces")?;
                        let mut ns = vde::Namespace::new(name);
                        for i in real_interfaces {
                            let ni = vde::NSInterface::new(i.name.clone(), i.ip, i.gateway);
                            ns.add_interface(ni);
                        }

                        for c in vde::Namespace::default_config() {
                            ns.add_config(c);
                        }
                        for c in ns.config_for_interfaces() {
                            ns.add_config(c);
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
                    AddSubcommands::Cable {
                        name,
                        a,
                        port_a,
                        open_a,
                        b,
                        port_b,
                        open_b,
                        wirefilter,
                        config,
                    } => {
                        let endp_a =
                            vde::find_endpoint_path(&t, &a, port_a.as_ref(), open_a).context(
                                format!("Finding endpoint path for {} on connection {}", &a, &name),
                            )?;
                        let endp_b =
                            vde::find_endpoint_path(&t, &b, port_b.as_ref(), open_b).context(
                                format!("Finding endpoint path for {} on connection {}", &b, &name),
                            )?;

                        let open_a = open_a.unwrap_or(false);
                        let open_b = open_b.unwrap_or(false);

                        let endp_a_proto = if open_a {
                            VdeConnProtocols::PTP
                        } else {
                            vde::find_endpoint_protocol(&t, &a).context(format!(
                                "Finding endpoint protocol for {} on connection {}",
                                &a, &name
                            ))?
                        };

                        let endp_b_proto = if open_b {
                            VdeConnProtocols::PTP
                        } else {
                            vde::find_endpoint_protocol(&t, &b).context(format!(
                                "Finding endpoint protocol for {} on connection {}",
                                &b, &name
                            ))?
                        };
                        let mut conn = vde::Cable::new(
                            name,
                            endp_a,
                            port_a,
                            endp_a_proto,
                            open_a,
                            endp_b,
                            port_b,
                            endp_b_proto,
                            open_b,
                            Some(wirefilter),
                        );

                        if let Some(config) = config {
                            let conf =
                                fs::read_to_string(config).context("Config file not found")?;
                            conf.lines().for_each(|l| conn.add_config(l.to_owned()));
                        }

                        t.add_cable(conn).context("Adding cable to topology")?;
                    }
                    AddSubcommands::Slirp { name } => {
                        let s = vde::Slirp::new(name);
                        t.add_slirp(s).context("Adding slirp to topology")?;
                    }
                    AddSubcommands::VXVDE { name, addr, port } => {
                        let mut vx = vde::VXVDE::new(name);
                        if let Some(addr) = addr {
                            vx.set_addr(addr);
                        }
                        if let Some(port) = port {
                            vx.set_port(port);
                        }

                        t.add_vxvde(vx).context("Adding vxvde to topology")?;
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

fn topology_create(opts: executor::Options, config: Option<String>, force: bool) -> Result<()> {
    if executor::topology_exists(&opts) {
        if !force {
            println!("Topology already exists. Use --force to overwrite or the clear command");
            return Err(anyhow::anyhow!("Topology already exists"));
        } else {
            executor::topology_stop(&opts, None)?;
            executor::clear_topology(&opts)?;
        }
    }

    let t;
    if let Some(config) = config {
        let file = fs::read_to_string(&config).context("Reading config file")?;

        let mut relative_path = PathBuf::from_str(&config)
            .context("Converting config path provided to a valid path")?;
        relative_path.pop();

        let c = config::Config::from_string(&file, relative_path).context("Parsing config")?;

        t = config_to_vde_topology(c).context("Converting config to vde topology")?;
    } else {
        t = vde::Topology::new();
    }

    executor::write_topology(opts.clone(), t).context("Writing topology")?;

    println!("Topology created");

    Ok(())
}

fn topology_import(opts: executor::Options, config: String, force: bool) -> Result<()> {
    if executor::topology_exists(&opts) {
        if !force {
            println!("Topology already exists. Use --force to overwrite or the clear command");
            return Err(anyhow::anyhow!("Topology already exists"));
        } else {
            executor::topology_stop(&opts, None)?;
            executor::clear_topology(&opts)?;
        }
    }

    let file = fs::read_to_string(config).context("Reading config file")?;

    // Before writing the configuration we need to check if is valid
    vde::Topology::from_string(&file).context("Import file is not valid. Maybe use `create`?")?;

    executor::write_raw_topology(opts, file).context("Writing topology to file")?;

    return Ok(());
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
                log::debug!(
                    "Adding interface {} to namespace {}. Ip: {}",
                    i.name,
                    ns.name,
                    i.ip.clone().unwrap_or_else(|| "None".to_string()),
                );
                let ni = vde::NSInterface::new(i.name.clone(), i.ip.clone(), i.gateway.clone());
                n.add_interface(ni);
            }
            for c in vde::Namespace::default_config() {
                n.add_config(c);
            }
            for c in n.config_for_interfaces() {
                n.add_config(c);
            }
            t.add_namespace(n).context("Adding namespace to topology")?;
        }
    }

    if let Some(sls) = &c.slirp {
        for s in sls {
            log::debug!("Parsing slirp {}", s.name);
            let s = vde::Slirp::new(s.name.clone());
            t.add_slirp(s).context("Adding slirp to topology")?;
        }
    }

    if let Some(vxvdes) = &c.vxvde {
        for v in vxvdes {
            log::debug!("Parsing vxvde {}", v.name);
            let mut vx = vde::VXVDE::new(v.name.clone());
            if let Some(addr) = &v.addr {
                vx.set_addr(addr.clone());
            }
            if let Some(port) = v.port {
                vx.set_port(port);
            }

            t.add_vxvde(vx).context("Adding vxvde to topology")?;
        }
    }

    if let Some(conns) = &c.cable {
        for c in conns {
            let endp_a = vde::find_endpoint_path(
                &t,
                &c.endpoint_a.name,
                c.endpoint_a.port.as_ref(),
                c.endpoint_a.open,
            )
            .context(format!(
                "Finding endpoint path for {} on connection {}",
                &c.endpoint_a.name, &c.name
            ))?;
            let port_a = c.endpoint_a.port.clone();
            let endp_b = vde::find_endpoint_path(
                &t,
                &c.endpoint_b.name,
                c.endpoint_b.port.as_ref(),
                c.endpoint_b.open,
            )
            .context(format!(
                "Finding endpoint path for {} on connection {}",
                &c.endpoint_b.name, &c.name
            ))?;
            let port_b = c.endpoint_b.port.clone();

            let open_a = c.endpoint_a.open.unwrap_or(false);
            let open_b = c.endpoint_b.open.unwrap_or(false);

            let endp_a_proto = if open_a {
                VdeConnProtocols::PTP
            } else {
                vde::find_endpoint_protocol(&t, &c.endpoint_a.name).context(format!(
                    "Finding endpoint protocol for {} on connection {}",
                    &c.endpoint_a.name, &c.name
                ))?
            };
            let endp_b_proto = if open_b {
                VdeConnProtocols::PTP
            } else {
                vde::find_endpoint_protocol(&t, &c.endpoint_b.name).context(format!(
                    "Finding endpoint protocol for {} on connection {}",
                    &c.endpoint_b.name, &c.name
                ))?
            };

            let mut conn = vde::Cable::new(
                c.name.clone(),
                endp_a,
                port_a,
                endp_a_proto,
                open_a,
                endp_b,
                port_b,
                endp_b_proto,
                open_b,
                c.wirefilter,
            );

            if let Some(config) = &c.config {
                let conf = fs::read_to_string(config).context("Config file not found")?;
                conf.lines().for_each(|l| conn.add_config(l.to_owned()));
            }

            t.add_cable(conn).context("Adding cable to topology")?;
        }
    }

    return Ok(t);
}

fn parse_config_file(file: &str) -> Result<Config> {
    let file = fs::read_to_string(file).context("Reading config file")?;
    let c = Config::from_string(&file).context("Parsing config")?;
    Ok(c)
}

fn parse_interfaces(interfaces: Vec<String>) -> Result<Vec<config::NSInterface>> {
    if interfaces.is_empty() {
        return Ok(vec![]);
    }

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
        if i.len() < 1 || i.len() > 3 {
            anyhow::bail!("Interface {n} definition must have between 1 and 3 elements");
        }
    }

    let mut real_interfaces: Vec<config::NSInterface> = Vec::new();
    for i in tmp.iter() {
        let name = i[0].clone();
        let (ip, gateway) = if i.len() >= 2 {
            let ip = Some(i[1].clone());
            if i.len() == 3 {
                (ip, Some(i[2].clone()))
            } else {
                (ip, None)
            }
        } else {
            (None, None)
        };
        let inter = config::NSInterface { name, ip, gateway };

        inter
            .checks()
            .context(format!("Checking interface {}", i[0]))?;
        real_interfaces.push(inter);
    }

    return Ok(real_interfaces);
}
