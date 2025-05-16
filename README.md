# ImagiNet

A command line tool to configure and manage complex VDE network topologies.

ImagiNet is a *network software emulator* that leverages **VDE** (Virtual Distributed 
Internet) for the emulations. It's like GNS3 or PacketTracer but without the GUI 
and more limited functionalities.

This project was designed with the intent of helping teach the basics of 
networking. It was not designed to cover all VDE features, and it was not 
designed to create the most efficient topology.


## Index

- [Index](#index)
- [Compile](#compile)
- [Usage](#usage)
- [How to](#how-to)
- [Terminal configuration](#terminal-configuration)
- [Troubleshooting](#troubleshooting)
    - [Namespace not starting](#namespace-not-starting)
    - [Switch not starting](#switch-not-starting)
    - [Cable not starting](#cable-not-starting)
- [Internals](#internals)

## Compile

The project is written in Rust and must be compiled trough cargo.
```bash
cargo build --release
```
The executable will be in `target/release/`.

## Usage

> [!IMPORTANT]
> To be able to successfuly use ImagiNet you must install [vde-2](https://github.com/virtualsquare/vde-2),
> [vdeplug4](https://github.com/rd235/vdeplug4) and [vdens](https://github.com/rd235/vdens).

ImagiNet provides a very helpful command line interface. Simply run:
```
$ imaginet help
Create and manage VDE topologies

Usage: imaginet [OPTIONS] [COMMAND]

Commands:
  add     Add a device to the current topology
  attach  Attach to a device in the topology
  create  Create a topology from a yaml configuration
  clear   Stop and delete the current topology
  dump    Dump current raw configuration
  exec    Execute a command in a device
  import  Import a topology from a raw configuration file (generated with dump)
  rm      Remove a device from the topology
  start   Start devices in the current topology
  status  Status of running topology
  stop    Stop devices in the current topology
  help    Print this message or the help of the given subcommand(s)

Options:
  -b, --base-dir <BASE_DIR>  Base directory for all imaginet files
  -t, --terminal <TERMINAL>  Terminal to open when starting or attaching to a device
  -c, --config <CONFIG>      Path to global configuration file
  -v, --verbose...           Verbosity level. Can be used multiple times for increased verbosity
  -h, --help                 Print help
  -V, --version              Print version
```
Or `help` before a specific subcommand command:
```
$ imaginet help create
``Create a topology from a yaml configuration

Usage: imaginet create [OPTIONS] [CONFIG]

Arguments:
  [CONFIG]  Path to configuration file. If not provided, an empty topology is created

Options:
  -f, --force  Force the creation of a new topology, deleting the current one
  -h, --help   Print help
```

### How to 

1. First you need to define a network file in YAML format. Some examples can be found in `examples/`.
2. The create the topology from the file with:
```
$ imaginet create <CONFIG>
```
3. The you can monitor the status with:
```
$ imaginet status
```
4. You can start all devices with:
```
$ imaginet start
```
or start a specific device with:
```
$ imaginet start <DEVICE>
```

## Terminal configuration

TODO

## Troubleshooting

It's possible that something does not work properly. To start troubleshoot your 
problems you can increas the verbosity of the program by adding `-v` for INFO, 
`-vv` for DEBUG or `-vvv` for TRACE messages.

To debug some common issues, try looking at the following sections

### Namespace not starting

The first step is to try starting a simple namespace from a terminal. Can you 
open a terminal, type `vdens` and actually open the namespace?

If no errors are printend you are probably inside the namespace and you can 
check by typing `ip a` a seeing that you only have a `lo` interface.

If you can successfully start the namespace manually, you should try executing 
the command provided by ImagiNet. Start by creating an empty topology:
```
$ imaginet create
Topology created
```
You can now add the simplest namespace possible: the one without any interface:
```
$ imaginet add namespace ns1
```
You should now see a single namespace with the `status` command:
```
$ imaginet status
Topology status
Namespaces:
- ns1 dead

Switches:

Cables:
```
Then execute the following command to start the namespace in a very verbose way:
```
$ imaginet -vvv start ns1
```
You should see a line like:
```
[DEBUG imaginet::executor] Executing: foot [] vdens ["--hostname", "ns1", "-", "/tmp/imnet/ns_starter.sh", "/tmp/imnet/ns1/pid"]
```
This indicates that the command ImagiNet is trying to run is:
```
foot vdens --hostname ns1 - /tmp/imnet/ns_starter.sh /tmp/imnet/ns1/pid
```
- `foot` is my current terminal. You can use whatever terminal you prefer, as 
long as it can accept arguments to execute a program. Look at the 
[terminal configuration](#terminal-configuration) section.
- `[]` These are an array of arguments for configuring the foot terminal. Foot 
does not require arguments, so it's empty. Look at the 
[terminal configuration](#terminal-configuration) section if your terminal 
requires special arguments.
- `vdens --hostname ns1 - /tmp/imnet/ns_starter.sh /tmp/imnet/ns1/pid` This is 
the actual command to start the namespace. You should try to execute this 
command on a separate terminal to see if any errors are printed.

### Switch not starting

### Cable not starting

## Internals

TODO

Main:
    - Config: Parsing del yaml. Controlli vari ed eventuali + errori
    - VDE: Modulo responsabile di generare effettivamente la rete.
        Lo divido in moduli per ogni componente di vde e faccio in modo
        che sia generale la possibilità di generare la rete
