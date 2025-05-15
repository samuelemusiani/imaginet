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
  attach  Attach to a device in a topology
  create  Create a topology
  start   Start a topology
  status  Status of running topology
  stop    Stop a topology
  exec    Execute a command in a device
  help    Print this message or the help of the given subcommand(s)

Options:
  -b, --base-dir <BASE_DIR>  Base directory for all imaginet files
  -t, --terminal <TERMINAL>  Terminal to open when starting or attaching to a device
  -c, --conifg <CONIFG>      Path to configuration file
  -h, --help                 Print help
  -V, --version              Print version
```
Or `help` before a specific subcommand command:
```bash
$ imaginet help create
Create a topology

Usage: imaginet create <CONFIG>

Arguments:
  <CONFIG>  Path to configuration file

Options:
  -h, --help  Print help
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
