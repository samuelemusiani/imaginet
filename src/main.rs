use clap::Parser;
use core::time;
use std::{fs, process, thread};
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
    vdeterm: bool
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

        let _ = process::Command::new("foot").args(["vde_switch", 
            "--sock", &format!("{path}/{}", &sw.name), 
            "--mgmt", &mgmt_path, 
            "-d"])
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
