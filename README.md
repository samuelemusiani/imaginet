# ImagiNet

A command line tool to configure and manage complex VDE network topologies.

## Compile

The project is written in Rust and must be compiled trough cargo.
```bash
cargo build --release
```
The executable will be in `target/release/`.

## Usage

ImagiNet provides a very helpful command line interface. Simply run:
```bash
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
```bash
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
```bash
$ imaginet create <CONFIG>
```
3. The you can monitor the status with:
```bash
$ imaginet status
```
4. You can start all devices with:
```bash
$ imaginet start
```
or start a specific device with:
```bash
$ imaginet start <DEVICE>
```

## Internals

TODO

Main:
    - Config: Parsing del yaml. Controlli vari ed eventuali + errori
    - VDE: Modulo responsabile di generare effettivamente la rete.
        Lo divido in moduli per ogni componente di vde e faccio in modo
        che sia generale la possibilità di generare la rete
