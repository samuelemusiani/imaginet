use std::{fs, path::PathBuf, process, thread};
//use core::time;

const WORKING_DIR: &str = "/tmp/rsnet";
const TERMINAL: &str = "foot";
const NS_STARTER: &str = "./ns_starter.sh";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn start(t: crate::vde::Topology) -> Result<()>{
    init()?;

    for sw in t.get_switches() {
        init_dir(sw.base_path(WORKING_DIR))?;

        if sw.needs_config() {
            let config = sw.get_config();
            let path = sw.config_path(WORKING_DIR);
            fs::write(&path, config.join("\n"))?;
        }

        let cmd = sw.exec_command();
        let args = sw.exec_args(WORKING_DIR);

        exec(&cmd, args).unwrap();
    }

    for ns in t.get_namespaces() {
        let cmd = ns.exec_command();
        let mut args = ns.exec_args(WORKING_DIR, NS_STARTER);

        args.insert(0, cmd);

        // Namespaces need to be started in a new terminal

        exec(TERMINAL, args).unwrap();

        // Need to configure the namespace
        thread::sleep(std::time::Duration::new(1, 0));
        // The following format i choosen by the ns_starter.sh script
        let pid = fs::read_to_string(&format!("{}/{}.pid", WORKING_DIR, 
            ns.get_name()))?.trim().to_owned();

        // I don't like the following part. It's too hardcoded
        for (i, el) in ns.get_interfaces().iter().enumerate() {

            ns_exec(&pid, &format!("ip link set vde{} name {}", &i, el.get_name())).unwrap();
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, &format!("ip addr add {} dev {}", el.get_ip(), el.get_name())).unwrap();
            thread::sleep(std::time::Duration::from_millis(100));

            ns_exec(&pid, &format!("ip link set {} up", el.get_name())).unwrap();
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    dbg!("HERE");

    for conn in t.get_connections() {
        let cmd = conn.exec_command();
        let args = conn.exec_args(WORKING_DIR);

        exec(&cmd, args).unwrap();
    }

    dbg!("HERE 2");

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

    dbg!(&cmd);
    dbg!(&args);

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

    exec(cmd, base_args).unwrap();
    Ok(())
}

//#[allow(dead_code)]
//pub fn run_net(c: crate::config::Config) {
//    for conn in c.connections {
//        let cp1 = format!("{path}/{}", conn.a);
//        let cp2 = format!("{path}/{}", conn.b);
//
//        let mut args = vec!("vde_plug", &cp1,);
//
//        let wrp = format!("{path}/wr_{}_mng", conn.name);
//        if let Some(_) = conn.wirefilter {
//            args.append(&mut vec!("=", "wirefilter", "-M", &wrp, "="));
//        } else {
//            args.push("=");
//        }
//
//        let mut conn2 = vec!("vde_plug", &cp2);
//        args.append(&mut conn2);
//
//        // Without the need of wirefilter we could probably only
//        // use vde_plug without dpipe. There is a performance hit?
//        let _ = process::Command::new("dpipe")
//            .args(args).spawn();
//
//        if let Some(_) = conn.wirefilter {
//            let _ = process::Command::new("foot")
//                .args(["vdeterm", &wrp]).spawn();
//        }
//    }
//}
