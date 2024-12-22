use std::{fs, process, thread};
use anyhow::{Context, Result, anyhow};
use colored::Colorize;
//use core::time;

const WORKING_DIR: &str = "/tmp/imnet";
const TERMINAL: &str = "foot";
const NS_STARTER: &str = "./ns_starter.sh";

pub fn get_topology() -> Result<crate::vde::Topology> {
    let t = &fs::read_to_string(&format!("{}/topology", WORKING_DIR)).context(
        "Reading topology file failed"
    )?;
    let t = crate::vde::Topology::from_string(t);

    Ok(t)
}

pub fn topology_start() -> Result<()>{
    let t = get_topology().context("Getting topology failed")?;

    for sw in t.get_switches() {
        let sw_name = sw.get_name();
        init_dir(sw.base_path(WORKING_DIR)).context(format!(
            "Initializing base dir for {}", sw_name
        ))?;

        if sw.needs_config() {
            let config = sw.get_config();
            let path = sw.config_path(WORKING_DIR);
            fs::write(&path, config.join("\n")).context(format!(
                "Writing config file for {}", sw_name
            ))?;
        }

        let cmd = sw.exec_command();
        let args = sw.exec_args(WORKING_DIR);

        exec(&cmd, args).context(format!(
            "Starting switch {}", sw_name
        ))?;
    }

    for ns in t.get_namespaces() {
        let ns_name = ns.get_name();

        let cmd = ns.exec_command();
        let mut args = ns.exec_args(WORKING_DIR, NS_STARTER);

        args.insert(0, cmd);

        // Namespaces need to be started in a new terminal

        exec(TERMINAL, args).context(format!(
            "Starting namespace {}", ns_name
        ))?;

        // Need to configure the namespace
        thread::sleep(std::time::Duration::new(1, 0));
        // The following format i choosen by the ns_starter.sh script
        let pid = fs::read_to_string(&format!("{}/{}.pid", WORKING_DIR,
            ns.get_name())).context(format!(
                "Reading pid file for {}", ns.get_name()
            ))?.trim().to_owned();

        // I don't like the following part. It's too hardcoded
        for (i, el) in ns.get_interfaces().iter().enumerate() {

            let interface_name = el.get_name();
            ns_exec(&pid, &format!("ip link set vde{} name {}", &i, interface_name))
                .context(format!("Changin name to interface {} on {}", interface_name, ns_name))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, &format!("ip addr add {} dev {}", el.get_ip(), el.get_name()))
                .context(format!("Adding ip to interface {} on {}", interface_name, ns_name))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, &format!("ip link set {} up", el.get_name()))
                .context(format!("Bringing up interface {} on {}", interface_name, ns_name))?;
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, "ip link set lo up")
                .context(format!("Bringing up localhost interface on {}", ns_name))?;
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    for conn in t.get_connections() {
        init_dir(conn.base_path(WORKING_DIR)).context(format!(
            "Initializing base dir for {}", conn.name
        ))?;

        let cmd = conn.exec_command();
        let args = conn.exec_args(WORKING_DIR);

        exec(&cmd, args).context(format!(
            "Starting connection {}", conn.name
        ))?;
    }

    Ok(())
}

fn init() -> Result<()> {
    if fs::exists(&WORKING_DIR)? {
        // Should check if a pid file is present
        fs::remove_dir_all(&WORKING_DIR)?;
    }
    fs::create_dir(&WORKING_DIR)?;

    Ok(())
}

fn init_dir(path: String) -> Result<()> {
    if fs::exists(&path)? {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir(&path)?;

    Ok(())
}

fn exec(cmd: &str, args: Vec<String>) -> Result<()> {
    process::Command::new(cmd).args(args).spawn()?;
    Ok(())
}

fn ns_exec(pid: &str, command: &str) -> Result<()> {
    let cmd = "nsenter";
    let mut base_args = vec!(
        "-t".to_owned(), pid.to_owned(),
        "--preserve-credentials".to_owned(),
        "-U".to_owned(), "-n".to_owned(),
        "--keep-caps".to_owned(),
    );

    let args = command.split_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    base_args.extend(args);

    exec(cmd, base_args).context("Executing command in namespace failed")?;
    Ok(())
}

pub fn write_topology(t: crate::vde::Topology) -> Result<()> {
    init()?;

    let t = t.to_string();

    fs::write(&format!("{}/topology", WORKING_DIR), t)?;

    println!("{}", "--- Topology created ---\n".bold());

    Ok(())
}

pub fn topology_status() -> Result<()> {
    let t = get_topology()?;

    println!("{}", "--- Topology status ---".bold().blink());
    println!("{}:", "Namespaces".bold());
    for n in t.get_namespaces() {
        let path = n.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", n.get_name(), "alive".green());
        } else {
            println!("- {} {}", n.get_name(), "dead".red());
        }
    };

    println!("\n{}:", "Switches".bold());

    for s in t.get_switches() {
        let path = s.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", s.get_name(), "alive".green());
        } else {
            println!("- {} {}", s.get_name(), "dead".red());
        }
    };

    println!("\n{}:", "Connections".bold());

    for conn in t.get_connections() {
        let path = conn.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            println!("- {} {}", conn.name, "alive".green());
        } else {
            println!("- {} {}", conn.name, "dead".red());
        }
    };

    Ok(())
}

fn pid_path_is_alive(path: &str) -> Result<bool> {
    if !fs::exists(&path)? {
        return Ok(false);
    }
    let pid = fs::read_to_string(path)?;
    let pid = pid.trim();

    return Ok(pid_is_alive(pid))
}

fn pid_is_alive(pid: &str) -> bool {
    // To check if a pid is alive we could use the kill syscall.
    // Or we could use the ps command
    process::Command::new("ps").arg("-p").arg(pid).output().unwrap().status.success()
}

pub fn topology_stop() -> Result<()> {
    let t = get_topology()?;

    for sw in t.get_switches() {
        let path = sw.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            // We could send a shutdown signal to the switch :)
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for ns in t.get_namespaces() {
        let path = ns.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    for conn in t.get_connections() {
        let path = conn.pid_path(WORKING_DIR);
        if pid_path_is_alive(&path)? {
            let pid = fs::read_to_string(&path)?.trim().to_owned();
            process::Command::new("kill").arg(pid).spawn()?;
        }
    }

    Ok(())
}

pub fn attach(device: String) -> Result<()> {
    let t = get_topology()?;
    const DEAD_ERR: &str = "Device not alive";

    for sw in t.get_switches() {
        let sw_name = sw.get_name();
        if sw_name == &device {
            let path = sw.pid_path(WORKING_DIR);
            if pid_path_is_alive(&path)? {
                let pid = fs::read_to_string(&path)?.trim().parse().context(format!(
                    "Internal error: can't parse pid for switch: {}", sw_name
                ))?;
                let cmd = sw.attach_command();
                let mut args = sw.attach_args(WORKING_DIR, pid);

                args.insert(0, cmd);

                exec(TERMINAL, args).context("Executing attach command")?;
                return Ok(());
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    for ns in t.get_namespaces() {
        if ns.get_name() == &device {
            let path = ns.pid_path(WORKING_DIR);
            if pid_path_is_alive(&path)? {
                let pid = fs::read_to_string(&path)?.trim().parse().context(format!(
                    "Internal error: can't parse pid for namespace: {}", ns.get_name()
                ))?;
                let cmd = ns.attach_command();
                let mut args = ns.attach_args(WORKING_DIR, pid);
                args.insert(0, cmd);

                exec(TERMINAL, args).context("Executing attach command")?;
                return Ok(());
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    for conn in t.get_connections() {
        if conn.name == device {
            let path = conn.pid_path(WORKING_DIR);
            if pid_path_is_alive(&path)? {
                let cmd = conn.attach_command()?;
                let mut args = conn.attach_args(WORKING_DIR)?;

                args.insert(0, cmd);

                exec(TERMINAL, args).context("Executing attach command")?;
                return Ok(());
            } else {
                return Err(anyhow!(DEAD_ERR));
            }
        }
    }

    Err(anyhow!("Device not found"))
}
