use clap::Parser;
use core::time;
use std::{fs, process, thread, path::Path};
use serde::{Serialize, Deserialize};

/// ImagiNet
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to file
    #[arg(short, long)]
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Switch {
    name: String,
    vdeterm: bool,
    config: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Namespace {
    name: String,
    connected: String,
    ip: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    switch: Vec<Switch>,
    namespace: Vec<Namespace>
}

fn main() {
    let args = Args::parse();

    let file = fs::read_to_string(args.path);

    match file {
        Err(e) => {
            eprintln!("Error opening file {}", e);
            process::exit(1);
        }
        Ok(file) => {
            println!("{file}");
            let c: Config = serde_yaml::from_str(&file).unwrap();

            run_net(c);
        }
    } 
}

fn run_net(c: Config) {
    dbg!(&c);


    let path = "/tmp/rsnet";
    if fs::exists(&path).unwrap() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir(&path).unwrap();

    fs::copy("./configurator.sh", &format!("{path}/configurator.sh")).unwrap();

    for sw in c.switch {
        println!("Switch: {}", sw.name);

        let mgmt_path = format!("{path}/{}_mgmt", &sw.name);


        let sw_sock = format!("{path}/{}", &sw.name);
        let mut args = vec!("vde_switch", 
            "--sock", &sw_sock, 
            "--mgmt", &mgmt_path, 
            "-d");

        let sw_conf;
        if let Some(config_path) = sw.config {
            fs::copy(config_path, &format!("{path}/{}.conf", sw.name))
                .expect("Cannot find config file for switch");

            args.push("--rcfile");
            sw_conf = format!("{path}/{}.conf", &sw.name);
            args.push(&sw_conf);
        };

        let _ = process::Command::new("foot").args(args)
            .spawn();

        
        thread::sleep(time::Duration::new(1, 0));

        if sw.vdeterm {
            let _ = process::Command::new("foot").args(["vdeterm", &mgmt_path]).spawn().expect("Can't spwan vdeterm for switch");
        }
    }

    // Should check for socket, not wait :)
    thread::sleep(time::Duration::new(1, 0));

    for ns in c.namespace {
        println!("Switch: {}", ns.name);
        let _ = process::Command::new("foot").args(["vdens", &format!("vde:///{path}/{}", ns.connected), &format!("{path}/configurator.sh"), &format!("{path}/sconf_{}", ns.name)]).spawn();

        dbg!("HERE");


        let res = fs::write(&format!("{path}/sconf_{}", ns.name), format!("ip a a {} dev vde0\nip l set vde0 up\n", ns.ip).as_bytes());
        match res {
            Ok(_) => println!("File created"),
            Err(e) => eprintln!("{e}")
        };
    }
}
