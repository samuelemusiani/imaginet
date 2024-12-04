use std::{fs, path::PathBuf, process, thread};
//use core::time;

const WORKING_DIR: &str = "/tmp/rsnet";
const TERMINAL: &str = "foot";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn start(t: crate::vde::Topology) -> Result<()>{
    init()?;

    for sw in t.get_switches() {
        let cmd = sw.exec_command();
        let args = sw.exec_args(WORKING_DIR);

        init_dir(sw.base_path(WORKING_DIR))?;

        exec(&cmd, args).unwrap();
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

    dbg!("Command: ", &cmd);
    dbg!("Args: ", &args);

    process::Command::new("vde_switch").args(args).spawn()?;

    Ok(())
}

//#[allow(dead_code)]
//pub fn run_net(c: crate::config::Config) {
//    dbg!(&c);
//
//
//    let path = "/tmp/rsnet";
//    if fs::exists(&path).unwrap() {
//        fs::remove_dir_all(&path).unwrap();
//    }
//    fs::create_dir(&path).unwrap();
//
//    fs::copy("./configurator.sh", &format!("{path}/configurator.sh")).unwrap();
//
//    for sw in c.switch {
//        println!("Switch: {}", sw.name);
//
//        let mgmt_path = format!("{path}/{}_mgmt", &sw.name);
//
//
//        let sw_sock = format!("{path}/{}", &sw.name);
//        let mut args = vec!("vde_switch", 
//            "--sock", &sw_sock, 
//            "--mgmt", &mgmt_path, 
//            "-d");
//
//        let sw_conf;
//        if let Some(config_path) = sw.config {
//            fs::copy(config_path, &format!("{path}/{}.conf", sw.name))
//                .expect("Cannot find config file for switch");
//
//            args.push("--rcfile");
//            sw_conf = format!("{path}/{}.conf", &sw.name);
//            args.push(&sw_conf);
//        };
//
//        let _ = process::Command::new("foot").args(args)
//            .spawn();
//
//
//        thread::sleep(time::Duration::new(1, 0));
//
//        if sw.vdeterm {
//            let _ = process::Command::new("foot").args(["vdeterm", &mgmt_path]).spawn().expect("Can't spwan vdeterm for switch");
//        }
//    }
//
//    // Should check for socket, not wait :)
//    thread::sleep(time::Duration::new(1, 0));
//
//    for ns in c.namespace {
//        println!("Switch: {}", ns.name);
//        let _ = process::Command::new("foot").args(["vdens", &format!("vde:///{path}/{}", ns.connected), &format!("{path}/configurator.sh"), &format!("{path}/sconf_{}", ns.name)]).spawn();
//
//        dbg!("HERE");
//
//        let res = fs::write(&format!("{path}/sconf_{}", ns.name), format!("ip a a {} dev vde0\nip l set vde0 up\n", ns.ip).as_bytes());
//        match res {
//            Ok(_) => println!("File created"),
//            Err(e) => eprintln!("{e}")
//        };
//
//        thread::sleep(time::Duration::new(1, 0));
//    }
//
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
