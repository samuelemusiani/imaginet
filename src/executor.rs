use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use std::io::Write;
use std::os::unix::{fs::PermissionsExt, process::CommandExt}; // Used for exec(), and permissions set on the namespace starter script
use std::{fs, process, thread};

const ERR_DEAD_DEVICE: &str = "Device not alive";

#[derive(Clone, Debug)]
pub struct Options {
    pub terminal: String,
    pub terminal_args: Vec<String>,
    pub working_dir: String,
}

pub fn get_topology(opts: &Options) -> Result<crate::vde::Topology> {
    let path = format!("{}/topology", opts.working_dir);
    let t = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(anyhow!(format!(
                    "Topology file not found in path: {path}. Have you created a topology?"
                )));
            };
            return Err(e.into());
        }
    };

    let t = crate::vde::Topology::from_string(&t).context("Converting file into topology")?;

    Ok(t)
}

pub fn topology_exists(opts: &Options) -> bool {
    let path = format!("{}/topology", opts.working_dir);
    fs::metadata(&path).is_ok()
}

/// If None is provided as devices, all devices are started
pub fn topology_start(opts: Options, devices: Option<Vec<String>>, inline: bool) -> Result<()> {
    log::debug!("Starting the topology");
    log::debug!("opts: {:?} devices: {:?} inline {inline}", opts, devices);
    let t = get_topology(&opts).context("Gettin topology")?;

    let devices = devices.unwrap_or(vec![]);

    if inline && (devices.len() > 1 || devices.len() == 0) {
        bail!("If the flag inline is specified you must pass only one device");
    }

    // Check if all the device provided are in the topology
    for d in &devices {
        if !t.is_name_used(d) {
            bail!("Device {d} does not exists on the current topology");
        }
    }

    log::trace!("Starting switches");
    for sw in t.get_switches() {
        if !devices.is_empty() && !devices.contains(&sw.get_name()) {
            log::trace!("Skipping switch {}", sw.get_name());
            continue;
        }

        if pid_path_is_alive(&sw.pid_path(&opts.working_dir))? {
            log::warn!("Switch {} is already started, skipping", sw.get_name());
            continue;
        }

        start_switch(&opts, sw)?;
    }

    // For namespaces we need a starter script in order to save
    // some information, such as the pid
    let script = crate::vde::Namespace::get_starter_script();
    let script_path = std::path::PathBuf::from(&opts.working_dir).join("ns_starter.sh");
    let mut file = fs::File::create(&script_path).context("Creating starter script")?;
    file.write(script)
        .context("Writing starter script into file")?;
    file.set_permissions(PermissionsExt::from_mode(0o755))
        .context("Setting permissions on starter script")?;

    drop(file);

    let script_path = script_path.to_str().unwrap().to_owned();
    log::trace!("Starting namespaces");
    for ns in t.get_namespaces() {
        if !devices.is_empty() && !devices.contains(&ns.get_name()) {
            log::trace!("Skipping namespace {}", ns.get_name());
            continue;
        }

        if pid_path_is_alive(&ns.pid_path(&opts.working_dir))? {
            log::warn!("Namespace {} is already started, skipping", ns.get_name());
            continue;
        }

        start_namespace(&opts, ns, &script_path, inline)?;
    }

    log::trace!("Starting cables");
    for conn in t.get_cables() {
        if !devices.is_empty() && !devices.contains(&conn.name) {
            log::trace!("Skipping cable {}", conn.name);
            continue;
        }

        if pid_path_is_alive(&conn.pid_path(&opts.working_dir))? {
            log::warn!(
                "Connection {} is already started, skipping",
                conn.get_name()
            );
            continue;
        }

        init_dir(conn.base_path(&opts.working_dir))
            .context(format!("Initializing base dir for {}", conn.name))?;

        if !conn.wirefilter && inline {
            bail!("Can't start cable inline if wirefilter is not specified")
        }

        start_cable(&opts, conn)?;
    }

    if inline {
        thread::sleep(std::time::Duration::new(1, 0));
        topology_attach(opts, devices[0].clone(), true)?;
    }

    Ok(())
}

fn start_switch(opts: &Options, sw: &crate::vde::Switch) -> Result<()> {
    let sw_name = sw.get_name();
    log::trace!("Starting switch {}", sw_name);
    init_dir(sw.base_path(&opts.working_dir))
        .context(format!("Initializing base dir for {}", sw_name))?;

    if sw.needs_config() {
        log::debug!("Switch needs configuration");
        let config = sw.get_config();
        log::trace!("config: {config:?}");
        let path = sw.config_path(&opts.working_dir);
        log::debug!("Writing configuration to {path}");
        fs::write(&path, config.join("\n"))
            .context(format!("Writing config file for {}", sw_name))?;
    }

    let cmd = sw.exec_command();
    log::debug!("Command: {}", cmd);
    let args = sw.exec_args(&opts.working_dir);
    log::debug!("Args: {:?}", args);

    exec(&cmd, &args).context(format!("Starting switch {}", sw_name))
}

fn start_namespace(
    opts: &Options,
    ns: &crate::vde::Namespace,
    script_path: &str,
    inline: bool,
) -> Result<()> {
    let ns_name = ns.get_name();
    log::trace!("Starting namespace {}", ns_name);
    init_dir(ns.base_path(&opts.working_dir))
        .context(format!("Initializing base dir for {}", ns_name))?;

    if ns.needs_config() {
        log::debug!("Switch needs configuration");
        let config = ns.get_config();
        log::trace!("config: {config:?}");
        let path = ns.config_path(&opts.working_dir);
        log::debug!("Writing configuration to {path}");
        fs::write(&path, config.join("\n"))
            .context(format!("Writing config file for {}", ns_name))?;
    }

    let cmd = ns.exec_command();
    log::debug!("Command: {}", cmd);
    let args = ns.exec_args(&opts.working_dir, script_path);
    log::debug!("Args: {:?}", args);

    if inline {
        // Start namespace in the current terminal
        exec_inline(&cmd, &args)
    } else {
        // Namespaces need to be started in a new terminal
        log::debug!("Terminal: {}", opts.terminal);
        log::debug!("Terminal args: {:?}", opts.terminal_args);
        exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
    }
    .context(format!("Starting namespace {}", ns_name))
}

fn start_cable(opts: &Options, cable: &crate::vde::Cable) -> Result<()> {
    log::trace!("Starting cable {}", cable.get_name());
    let cmd = cable.exec_command();
    let args = cable.exec_args(&opts.working_dir);

    if cable.needs_config() {
        log::trace!("Configuring cable {}", cable.get_name());
        let config = cable.get_config();
        let path = cable.config_path(&opts.working_dir);
        fs::write(&path, config.join("\n"))
            .context(format!("Writing config file for {}", cable.name))?;
    }

    log::debug!("Executing {cmd} {:?}", args);
    exec(&cmd, &args).context(format!("Starting cable {}", cable.name))
}

fn init(opts: &Options) -> Result<()> {
    if fs::exists(&opts.working_dir)? {
        // Should check if a pid file is present
        fs::remove_dir_all(&opts.working_dir)?;
    }
    fs::create_dir(&opts.working_dir)?;

    Ok(())
}

fn init_dir(path: String) -> Result<()> {
    if fs::exists(&path)? {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir(&path)?;

    Ok(())
}

/// Execute a command with args inside a terminal
fn exec_terminal(
    terminal: &str,
    terminal_args: &Vec<String>,
    cmd: &str,
    args: &Vec<String>,
) -> Result<()> {
    log::debug!("Executing: {terminal} {terminal_args:?} {cmd} {args:?}");

    process::Command::new(terminal)
        .args(terminal_args)
        .arg(cmd)
        .args(args)
        .spawn()
        .context(format!(
            "Executing terminal {terminal} with command '{cmd}'\nargs: {args:#?}"
        ))?;
    Ok(())
}

/// Execute a command with args
fn exec(cmd: &str, args: &Vec<String>) -> Result<()> {
    log::debug!("Executing: {cmd} {args:?}");
    process::Command::new(cmd)
        .args(args)
        .spawn()
        .context(format!("Executing commad '{cmd}'\nargs: {args:#?}"))?;
    Ok(())
}

/// This is a point of no return. Replace the current process with cmd. If it fails, it returns an error
fn exec_inline(cmd: &str, args: &Vec<String>) -> Result<()> {
    let err = process::Command::new(cmd).args(args).exec();

    // If we reach this point, the exec failed
    Err(anyhow!(
        "Executing command '{cmd}'\nargs: {args:#?}\nError: {err}"
    ))
}

pub fn write_topology(opts: Options, t: crate::vde::Topology) -> Result<()> {
    init(&opts).context("Initializing executor")?;

    write_raw_topology(
        opts.clone(),
        t.to_string().context("Converting topology to string")?,
    )
}

pub fn write_raw_topology(opts: Options, t: String) -> Result<()> {
    init(&opts).context("Initializing executor")?;

    let path = &format!("{}/topology", &opts.working_dir);
    fs::write(&path, t).context(format!("Writing topology on file {path}"))?;

    Ok(())
}

/// If None is provided as devices, all devices are printed in the status
pub fn topology_status(opts: Options, devices: Option<Vec<String>>, verbose: u8) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    println!("{}", "Topology status".bold());
    println!("{}:", "Namespaces".bold());
    for n in t.get_namespaces() {
        if let Some(devices) = &devices {
            if !devices.contains(&n.get_name().to_owned()) {
                continue;
            }
        }

        let path = n.pid_path(&opts.working_dir);
        let status = if pid_path_is_alive(&path)? {
            "alive".green()
        } else {
            "dead".red()
        };

        println!("- {} {}", n.get_name(), status);
        if verbose > 0 {
            for i in n.get_interfaces() {
                println!(
                    "\tinterface: {}\n\tip: {}",
                    i.get_name().bold(),
                    i.get_ip()
                        .clone()
                        .unwrap_or_else(|| "None".to_string())
                        .bold(),
                );
            }
        }
    }

    println!("\n{}:", "Switches".bold());

    for s in t.get_switches() {
        if let Some(devices) = &devices {
            if !devices.contains(&s.get_name().to_owned()) {
                continue;
            }
        }

        let path = s.pid_path(&opts.working_dir);
        let status = if pid_path_is_alive(&path)? {
            "alive".green()
        } else {
            "dead".red()
        };

        println!("- {} {}", s.get_name(), status);
        if verbose > 0 {
            println!(
                "\tports: {}\n\thub: {}",
                s.get_ports().to_string().bold(),
                s.is_hub().to_string().bold()
            );
        }

        if verbose > 1 {
            println!("\tconfig:");
            for l in s.get_config() {
                println!("\t  {}", l.bold());
            }
        }
    }

    println!("\n{}:", "Cables".bold());

    for conn in t.get_cables() {
        if let Some(devices) = &devices {
            if !devices.contains(&conn.name) {
                continue;
            }
        }

        let path = conn.pid_path(&opts.working_dir);
        let status = if pid_path_is_alive(&path)? {
            "alive".green()
        } else {
            "dead".red()
        };
        println!("- {} {}", conn.name, status);
        if verbose > 0 {
            let endp_a = conn.get_a();
            let endp_b = conn.get_b();
            println!(
                "\tendpoint_a: {} {} {}\n\tendpoint_b: {} {} {}\n\twirefilter: {}",
                endp_a.get_name().bold(),
                option_to_string(endp_a.get_port()).bold(),
                endp_a.get_protocol().to_string().bold(),
                endp_b.get_name().bold(),
                option_to_string(endp_b.get_port()).bold(),
                endp_b.get_protocol().to_string().bold(),
                conn.has_wirefilter().to_string().bold()
            );
        }

        if verbose > 1 {
            println!("\tconfig:");
            for l in conn.get_config() {
                println!("\t  {}", l.bold());
            }
        }
    }

    Ok(())
}

fn pid_path_is_alive(path: &str) -> Result<bool> {
    if !fs::exists(&path)? {
        return Ok(false);
    }
    let pid = fs::read_to_string(path)?;
    let pid = pid.trim();

    return Ok(pid_is_alive(pid));
}

fn pid_is_alive(pid: &str) -> bool {
    // To check if a pid is alive we could use the kill syscall.
    // Or we could use the ps command
    process::Command::new("ps")
        .arg("-p")
        .arg(pid)
        .output()
        .unwrap()
        .status
        .success()
}

/// If None is provided as devices, all devices are stopped
pub fn topology_stop(opts: &Options, devices: Option<Vec<String>>) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
        if let Some(devices) = &devices {
            if !devices.contains(&sw.get_name().to_owned()) {
                continue;
            }
        }

        let path = sw.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            // We could send a shutdown signal to the switch :)
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for ns in t.get_namespaces() {
        if let Some(devices) = &devices {
            if !devices.contains(&ns.get_name().to_owned()) {
                continue;
            }
        }

        let path = ns.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for conn in t.get_cables() {
        if let Some(devices) = &devices {
            if !devices.contains(&conn.name) {
                continue;
            }
        }

        let path = conn.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    Ok(())
}

pub fn topology_attach(opts: Options, device: String, inline: bool) -> Result<()> {
    log::trace!("Attaching to device {}", device);
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
        let sw_name = sw.get_name();
        if sw_name != &device {
            continue;
        }

        log::trace!("Attaching to switch {}", sw_name);

        let path = sw.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let pid = fs::read_to_string(&path)?.trim().parse().context(format!(
            "Internal error: can't parse pid for switch: {}",
            sw_name
        ))?;
        let cmd = sw.attach_command();
        let args = sw.attach_args(&opts.working_dir, pid);

        if inline {
            exec_inline(&cmd, &args).context("Executing attach command")?;
        } else {
            exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
                .context("Executing attach command")?;
        }
        return Ok(());
    }

    for ns in t.get_namespaces() {
        if ns.get_name() != &device {
            continue;
        }

        log::trace!("Attaching to namespace {}", ns.get_name());

        let path = ns.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let pid = fs::read_to_string(&path)?.trim().parse().context(format!(
            "Internal error: can't parse pid for namespace: {}",
            ns.get_name()
        ))?;
        let cmd = ns.attach_command();
        let args = ns.attach_args(&opts.working_dir, pid);

        if inline {
            exec_inline(&cmd, &args).context("Executing attach command")?;
        } else {
            exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
                .context("Executing attach command")?;
        }
        return Ok(());
    }

    for conn in t.get_cables() {
        if conn.name != device {
            continue;
        }

        log::trace!("Attaching to cable {}", conn.name);

        let path = conn.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let cmd = conn.attach_command()?;
        let args = conn.attach_args(&opts.working_dir)?;

        if inline {
            exec_inline(&cmd, &args).context("Executing attach command")?;
        } else {
            exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
                .context("Executing attach command")?;
        }
        return Ok(());
    }

    Err(anyhow!("Device not found"))
}

/// Execute a command inside a device. This genereally use vdecmd, but if a
/// namespace is provided, it uses nsenter
pub fn topology_exec(opts: Options, device: String, command: Vec<String>) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    let mut command = command;

    for sw in t.get_switches() {
        if sw.get_name() != &device {
            continue;
        }

        let path = sw.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let cmd = sw.exec_command_command();
        let args = sw.exec_command_args(&opts.working_dir, command.as_mut());

        exec_inline(&cmd, &args).context("Executing command inside switch")?;
        return Ok(());
    }

    for ns in t.get_namespaces() {
        if ns.get_name() != &device {
            continue;
        }

        let path = ns.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let pid = fs::read_to_string(&path)?.trim().parse().unwrap();
        let cmd = ns.exec_command_command();
        let args = ns.exec_command_args(&opts.working_dir, pid, command.as_mut());

        exec_inline(&cmd, &args).context("Executing command inside namespace")?;
        return Ok(());
    }

    for conn in t.get_cables() {
        if conn.name != device {
            continue;
        }

        let path = conn.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let cmd = conn.exec_command_command()?;
        let args = conn.exec_command_args(&opts.working_dir, command.as_mut())?;

        exec_inline(&cmd, &args).context("Executing command inside cable")?;
        return Ok(());
    }

    Err(anyhow!("Device not found"))
}

pub fn clear_topology(opts: &Options) -> Result<()> {
    fs::remove_dir_all(&opts.working_dir).context("Removing working directory")
}

fn option_to_string<T: ToString>(opt: Option<T>) -> String {
    match opt {
        Some(value) => value.to_string(),
        None => String::from(""),
    }
}
