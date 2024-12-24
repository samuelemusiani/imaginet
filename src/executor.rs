use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::io::Write;
use std::os::unix::{fs::PermissionsExt, process::CommandExt}; // Used for exec(), and permissions set on the namespace starter script
use std::{fs, process, thread};
//use core::time;

const NS_STARTER: &str = "./ns_starter.sh";

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

pub fn topology_start(opts: Options) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
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

        exec(&cmd, &args).context(format!("Starting switch {}", sw_name))?;
    }

    // For namespaces we need a starter script in order to save
    // some information, such as the pid
    let script = crate::vde::Namespace::get_starter_script();
    let script_path = std::path::PathBuf::from(&opts.working_dir).join(NS_STARTER);
    let mut file = fs::File::create(&script_path).context("Creating starter script")?;
    file.write(script)
        .context("Writing starter script into file")?;
    file.set_permissions(PermissionsExt::from_mode(0o755))
        .context("Setting permissions on starter script")?;

    for ns in t.get_namespaces() {
        let ns_name = ns.get_name();

        let cmd = ns.exec_command();
        let args = ns.exec_args(&opts.working_dir, script_path.to_str().unwrap());

        // Namespaces need to be started in a new terminal

        exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
            .context(format!("Starting namespace {}", ns_name))?;

        // Need to configure the namespace
        thread::sleep(std::time::Duration::new(1, 0));
        // The following format i choosen by the ns_starter.sh script
        let path = format!("{}/{}.pid", &opts.working_dir, ns.get_name());
        let pid = fs::read_to_string(&path)
            .context(format!("Reading pid file for {}", ns.get_name()))?
            .trim()
            .to_owned();

        // I don't like the following part. It's too hardcoded
        for (i, el) in ns.get_interfaces().iter().enumerate() {
            let interface_name = el.get_name();
            ns_exec(
                &pid,
                &format!("ip link set vde{} name {}", &i, interface_name),
            )
            .context(format!(
                "Changin name to interface {} on {}",
                interface_name, ns_name
            ))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(
                &pid,
                &format!("ip addr add {} dev {}", el.get_ip(), el.get_name()),
            )
            .context(format!(
                "Adding ip to interface {} on {}",
                interface_name, ns_name
            ))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, &format!("ip link set {} up", el.get_name())).context(format!(
                "Bringing up interface {} on {}",
                interface_name, ns_name
            ))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, "ip link set lo up")
                .context(format!("Bringing up localhost interface on {}", ns_name))?;
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    for conn in t.get_connections() {
        init_dir(conn.base_path(&opts.working_dir))
            .context(format!("Initializing base dir for {}", conn.name))?;

        let cmd = conn.exec_command();
        let args = conn.exec_args(&opts.working_dir);

        if conn.needs_config() {
            let config = conn.get_config();
            let path = conn.config_path(&opts.working_dir);
            fs::write(&path, config.join("\n"))
                .context(format!("Writing config file for {}", conn.name))?;
        }

        exec(&cmd, &args).context(format!("Starting connection {}", conn.name))?;
    }

    Ok(())
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

/// Execute a command inside a namespace identified by pid
fn ns_exec(pid: &str, command: &str) -> Result<()> {
    let cmd = "nsenter";
    let mut base_args = vec![
        "-t".to_owned(),
        pid.to_owned(),
        "--preserve-credentials".to_owned(),
        "-U".to_owned(),
        "-n".to_owned(),
        "--keep-caps".to_owned(),
    ];

    let args = command
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    base_args.extend(args);

    exec(cmd, &base_args).context(format!(
        "Executing command {cmd} in namespace failed.\n{base_args:#?}"
    ))?;
    Ok(())
}

pub fn write_topology(opts: Options, t: crate::vde::Topology) -> Result<()> {
    init(&opts).context("Initializing executor")?;
    let t = t.to_string().context("Converting topology to string")?;

    let path = &format!("{}/topology", &opts.working_dir);
    fs::write(&path, t).context(format!("Writing topology on file {path}"))?;

    println!("{}", "--- Topology created ---\n".bold());

    Ok(())
}

pub fn topology_status(opts: Options) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    println!("{}", "--- Topology status ---".bold());
    println!("{}:", "Namespaces".bold());
    for n in t.get_namespaces() {
        let path = n.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", n.get_name(), "alive".green());
        } else {
            println!("- {} {}", n.get_name(), "dead".red());
        }
    }

    println!("\n{}:", "Switches".bold());

    for s in t.get_switches() {
        let path = s.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", s.get_name(), "alive".green());
        } else {
            println!("- {} {}", s.get_name(), "dead".red());
        }
    }

    println!("\n{}:", "Connections".bold());

    for conn in t.get_connections() {
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

pub fn topology_stop(opts: Options) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;

    for sw in t.get_switches() {
        let path = sw.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            // We could send a shutdown signal to the switch :)
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for ns in t.get_namespaces() {
        let path = ns.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for conn in t.get_connections() {
        let path = conn.pid_path(&opts.working_dir);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    Ok(())
}

pub fn attach(opts: Options, device: String, inline: bool) -> Result<()> {
    let t = get_topology(&opts).context("Gettin topology")?;
    const DEAD_ERR: &str = "Device not alive";

    for sw in t.get_switches() {
        let sw_name = sw.get_name();
        if sw_name == &device {
            let path = sw.pid_path(&opts.working_dir);
            if pid_path_is_alive(&path)? {
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
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    for ns in t.get_namespaces() {
        if ns.get_name() == &device {
            let path = ns.pid_path(&opts.working_dir);
            if pid_path_is_alive(&path)? {
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
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    for conn in t.get_connections() {
        if conn.name == device {
            let path = conn.pid_path(&opts.working_dir);
            if pid_path_is_alive(&path)? {
                let cmd = conn.attach_command()?;
                let args = conn.attach_args(&opts.working_dir)?;

                if inline {
                    exec_inline(&cmd, &args).context("Executing attach command")?;
                } else {
                    exec_terminal(&opts.terminal, &opts.terminal_args, &cmd, &args)
                        .context("Executing attach command")?;
                }
                return Ok(());
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    Err(anyhow!("Device not found"))
}
