use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::Write;
use std::os::unix::{fs::PermissionsExt, process::CommandExt}; // Used for exec(), and permissions set on the namespace starter script
use std::{fs, process, thread};

const ERR_DEAD_DEVICE: &str = "Device not alive";

#[derive(Clone)]
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

/// If None is provided as devices, all devices are started
pub fn topology_start(opts: Options, devices: Option<Vec<String>>) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
        if let Some(devices) = &devices {
            if !devices.contains(&sw.get_name().to_owned()) {
                continue;
            }
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
    for ns in t.get_namespaces() {
        if let Some(devices) = &devices {
            if !devices.contains(&ns.get_name().to_owned()) {
                continue;
            }
        }

        start_namespace(&opts, ns, &script_path)?;
        configure_namespace(&opts, ns)?;
    }

    for conn in t.get_connections() {
        if let Some(devices) = &devices {
            if !devices.contains(&conn.name) {
                continue;
            }
        }

        init_dir(conn.base_path(&opts.working_dir))
            .context(format!("Initializing base dir for {}", conn.name))?;
        configure_connection(&opts, conn)?;
    }

    Ok(())
}

fn start_switch(opts: &Options, sw: &crate::vde::Switch) -> Result<()> {
    let sw_name = sw.get_name();
    init_dir(sw.base_path(&opts.working_dir))
        .context(format!("Initializing base dir for {}", sw_name))?;

    if sw.needs_config() {
        let config = sw.get_config();
        let path = sw.config_path(&opts.working_dir);
        fs::write(&path, config.join("\n"))
            .context(format!("Writing config file for {}", sw_name))?;
    }

    let cmd = sw.exec_command();
    let args = sw.exec_args(&opts.working_dir);

    exec(&cmd, &args).context(format!("Starting switch {}", sw_name))
}

fn start_namespace(opts: &Options, ns: &crate::vde::Namespace, script_path: &str) -> Result<()> {
    let ns_name = ns.get_name();

    let cmd = ns.exec_command();
    let args = ns.exec_args(&opts.working_dir, script_path);

    // Namespaces need to be started in a new terminal

    exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
        .context(format!("Starting namespace {}", ns_name))
}

fn configure_namespace(opts: &Options, ns: &crate::vde::Namespace) -> Result<()> {
    // Need to configure the namespace
    let ns_name = ns.get_name();

    thread::sleep(std::time::Duration::new(1, 0));
    // The following format i choosen by the ns_starter.sh script
    let path = format!("{}/{}.pid", &opts.working_dir, ns.get_name());
    let pid = fs::read_to_string(&path)
        .context(format!("Reading pid file for {}", ns.get_name()))?
        .trim()
        .parse()
        .unwrap();

    let cmd = ns.exec_command_command();
    let ns_exec = |command: &str| -> Result<()> {
        let mut args = command
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        let args = ns.exec_command_args(&opts.working_dir, pid, args.as_mut());

        exec(&cmd, &args)
    };

    for (i, el) in ns.get_interfaces().iter().enumerate() {
        let interface_name = el.get_name();

        let v = vec![
            format!("ip link set {} name vde{}", interface_name, i),
            format!("ip addr add {} dev vde{}", el.get_ip(), i),
            format!("ip link set vde{} up", i),
        ];

        for command in v {
            ns_exec(&command).context(format!(
                "Executing command '{}' on interface {} on {}",
                command, interface_name, ns_name
            ))?;
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    Ok(())
}

fn configure_connection(opts: &Options, conn: &crate::vde::Connection) -> Result<()> {
    let cmd = conn.exec_command();
    let args = conn.exec_args(&opts.working_dir);

    if conn.needs_config() {
        let config = conn.get_config();
        let path = conn.config_path(&opts.working_dir);
        fs::write(&path, config.join("\n"))
            .context(format!("Writing config file for {}", conn.name))?;
    }

    exec(&cmd, &args).context(format!("Starting connection {}", conn.name))
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
    let t = t.to_string().context("Converting topology to string")?;

    let path = &format!("{}/topology", &opts.working_dir);
    fs::write(&path, t).context(format!("Writing topology on file {path}"))?;

    println!("{}", "--- Topology created ---\n".bold());

    Ok(())
}

/// If None is provided as devices, all devices are printed in the status
pub fn topology_status(opts: Options, devices: Option<Vec<String>>) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    println!("{}", "--- Topology status ---".bold());
    println!("{}:", "Namespaces".bold());
    for n in t.get_namespaces() {
        if let Some(devices) = &devices {
            if !devices.contains(&n.get_name().to_owned()) {
                continue;
            }
        }

        let path = n.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", n.get_name(), "alive".green());
        } else {
            println!("- {} {}", n.get_name(), "dead".red());
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
        if pid_path_is_alive(&path)? {
            println!("- {} {}", s.get_name(), "alive".green());
        } else {
            println!("- {} {}", s.get_name(), "dead".red());
        }
    }

    println!("\n{}:", "Connections".bold());

    for conn in t.get_connections() {
        if let Some(devices) = &devices {
            if !devices.contains(&conn.name) {
                continue;
            }
        }

        let path = conn.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", conn.name, "alive".green());
        } else {
            println!("- {} {}", conn.name, "dead".red());
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
pub fn topology_stop(opts: Options, devices: Option<Vec<String>>) -> Result<()> {
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

    for conn in t.get_connections() {
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
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
        let sw_name = sw.get_name();
        if sw_name == &device {
            continue;
        }

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

    for conn in t.get_connections() {
        if conn.name != device {
            continue;
        }

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

    for conn in t.get_connections() {
        if conn.name != device {
            continue;
        }

        let path = conn.pid_path(&opts.working_dir);
        if !pid_path_is_alive(&path)? {
            return Err(anyhow!(ERR_DEAD_DEVICE));
        }

        let cmd = conn.exec_command_command()?;
        let args = conn.exec_command_args(&opts.working_dir, command.as_mut())?;

        exec_inline(&cmd, &args).context("Executing command inside connection")?;
        return Ok(());
    }

    Err(anyhow!("Device not found"))
}
