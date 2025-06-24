# ImagiNet

A command line tool to configure and manage complex VDE network topologies.

ImagiNet is a *network software simulator* that leverages **VDE** (Virtual Distributed 
Internet). It's like GNS3 or PacketTracer but without the GUI 
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
- [Examples](#examples)
- [Wireshark](#wireshark)
- [Life of a](#life-of-a)
    - [Namespace](#namespace)
    - [Switch](#switch)
    - [Cable](#cable)
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
> You must also install vdeplug4 **after** vde-2, because the vde_plug executable
> provided by vde-2 must be overwritten by the new version provided by vdeplug4.
>
> In order to make the `exec` command works for namespaces, you must also install 
> the `nsenter` program present in the `util-linux` package. `nsenter` must be
> at minimum version 2.40, but it is recommended to use the latest version available

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

## Configuration

It's possible to define some configuration options in a global configuration 
file to avoid specifying them on the command line every time. Other options 
(i.e. terminal arguments) can not be specified on the command line and, if they 
are needed, they have to be specified in the configuration file. 

A template for a configuration file is present in the file `config.yaml`.
You can pass this file to ImagiNet with the option `--config` or you could put
it in `$HOME/.config/imaginet/config.yaml`.

### Terminal configuration

Some terminals (i.e. `gnome-terminal`) needs some arguments to function correctly.
The idea is that ImagiNet should be able to execute the terminal with some 
arguments and the arguments are the first program that the terminal will execute.

For `foot` no argument is necessary, but for `gnome-terminal` the following 
configuration is needed:
```
terminal:
  executable: gnome-terminal
  args: ["--"]
```

## Examples

Under the `examples/` directory you can find some examples of network topologies. 
This files should provide a good view of all configuration possibilities.

Given the didactic nature of the project, every example start with a simple
description of the topology and what you can expect from it. Then there is a
small section of what you can learn from the example. Then there is the last
section in which you can find what you have to do to make the example work, some
commands to run and some expected output.

## Wireshark

Wireshark is a very powerful tool to analyze network traffic. It is the perfect
tool to learn and analyze the network traffic generated by ImagiNet. You can
always start Wireshark inside a namespace with the following command:
```
$ imaginet exec <namespace> wireshark
```

You can't put wireshark between a cable like some other tools (e.g. GNS3), but
in real life you can't do that either. The only way to analyze the traffic is to
have wireshark at one end of the cable. 

To analyze traffic from more than two devices, you can start multiple istances or
you can use a switch configured as a hub. The switch will forward all the traffic
to all the ports, so you can analyze the traffic from all the devices connected
by looking only at one of them.

## Life of a

Every device have a different life cycle. Unlike something like GNS3, ImagiNet
DOES NOT save the state of the devices. This means that every time you start a 
device it will start from scratch. So be really careful when you are stopping a
device. If you need to save a particular configuration, write the commands 
somewhere safe.

### Namespace

Namespaces are active as long as at least one process is running inside them.
When you start a namespace, it will start a shell inside it. If you execute a
command inside the namespace, the command will keep the namespace active. The 
namespace will be stopped once every process inside it is stopped.
BUT as soon as the first process inside the namespace is stopped, all the
network interfaces inside the namespace will be removed. In this case the
namespace is in a inconsistent state and should be restarted.

### Switch

Switches are active as long as the process is running. When you start a switch,
it will start the process in the background that will keep the switch active.

If you enter the switch and type `shutdown`, the switch will be stopped.

### Cable

If a cable have wirefilter is able to stay "active" even if both ends are 
disconnected. If wirefilter is not set, the cable will be stopped when one of 
the ends is disconnected.

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
- ns1 inactive

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

The first step is to try starting a switch from a terminal. Can you 
open a terminal, type `vde_switch`, spam some new lines and actually see a shell
like the folliwing?
```
$ vde_switch


vde$ 
```

If you can successfully start the switch manually, you should try executing 
the command provided by ImagiNet. The steps for adding a switch are almost the
same as the ones used for adding a namespace. For a more detailed explanation
you should look at [Namespace not starting](#namespace-not-starting).
You should execute the following commans:
```
$ imaginet create
Topology created
$ imaginet add switch sw1
$ imaginet status
Topology status
Namespaces:

Switches:
- sw1 inactive

Cables:
$ imaginet -vvv start sw1
```
You should see a line like the following:
```
[DEBUG imaginet::executor] Executing: vde_switch ["--pidfile", "/tmp/imnet/sw1/pid", "--mgmt", "/tmp/imnet/sw1/mgmt", "--sock", "/tmp/imnet/sw1/sock", "--rcfile", "/tmp/imnet/sw1/config", "--numports", "32", "--daemon"]
```
All the arguments are specific to the switch. If you are interested, you can take
a look at the [Internals](#internals) section.
As with the namespace, you should try running the command manually and see if
any errors are printed. It could be helpful to run the command without the 
`--daemon` argument to see the output of the switch.

### Cable not starting

There are two types of cables: the ones with wirefilter and the ones without.
Let's start with the ones without wirefilter. Do you have vdeplug4 installed?
```
$ vde_plug
```
It's mandatory that the `vde_plug` executable is installed from vdeplug4 and not
from vde-2. Please check the [Usage](#usage) section to see how to install.

After you have verified that you have vdeplug4 installed, can you start a simple
point-to-point cable?
```
$ vde_plug ptp:///tmp/ptp1
```
This command does not print anything, but it "hangs". This is normal.

If you can successfully start the cable manually, you should try executing
the start command provided by ImagiNet but with the verbose flags:
```
$ imaginet -vvv start conn1
```
This will print something like:
```
[DEBUG imaginet::executor] Executing vde_plug ["ptp:///tmp/imnet/./ns1/eth0", "ptp:///tmp/imnet/./ns2/eth0", "--pidfile", "/tmp/imnet/conn1/pid", "--descr", "conn1", "--daemon"]
```
You should try running the command manually and see if any errors are printed.

#### Wirefilter

In order to use wirefilter, you need to have the `dpipe` executable installed from the `vdeplug4` package.

Wirefilter is an executable found in `vde-2`, you need to have this, too.
For both of them if you execute the command without arguments you should see the 
help message.

The debug verbosity for the start command is always your friend. Something like
the following is the command for a wirefilter cable, you should try running it
manually and see if any errors are printed:
```
[DEBUG imaginet::executor] Executing: dpipe ["--daemon", "--pidfile", "/tmp/imnet/conn1/pid", "vde_plug", "ptp:///tmp/imnet/./ns1/eth0", "=", "wirefilter", "--mgmt", "/tmp/imnet/conn1/mgmt", "--rcfile", "/tmp/imnet/conn1/config", "=", "vde_plug", "ptp:///tmp/imnet/./ns2/eth0"]
```

## Internals

All the internals of ImagiNet are documented in the `INTERNALS.md` file.
