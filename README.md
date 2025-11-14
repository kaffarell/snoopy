# snoopy

![snoopy](./logo/snoopy.png)

Snoop ARP traffic and insert neighbor entries on bridges for EVPN
advertisement. This functionality is similar to the `snooping` feature found in
CumulusNetworks' `neighmgrd` daemon.

## Why
In a typical EVPN setup, a bridge connects at least two ports: a VM (or
container) and a VXLAN interface that tunnels traffic to other nodes in the
EVPN zone. FRR only advertises neighbor entries that exist directly on the
bridge.

The issue: when pinging a VM on the same subnet but on a different node, the
bridge forwards traffic directly to the VXLAN interface without creating a
neighbor entry -- only an FDB entry is created on the VXLAN interface. The
bridge only creates neighbor entries when it directly receives ARP queries,
such as when traffic is destined for a different subnet where the bridge serves
as the gateway.

The solution is to monitor ARP requests originating from the VM and manually
install corresponding neighbor entries on the bridge. FRR will then detect
these neighbors and advertise them via EVPN Type-2 routes.

```
                  ┌────────────────────────┐  ┌──────────────────────┐  
                  │ Control Plane          │  │ Data Plane           │  
                  │                        │  │                      │  
                  │                        │  │                      │  
┌─────────────────│────────────────────────│──│──────────────────────│─┐
│ Node1           │                        │  │                      │ │
│                 │                        │  │                      │ │
│                 │                        │  │                      │ │
│  ┌────────────┐ │        ┌──────┐        │  │  ┌───────────────┐   │ │
│  │VM interface├─┼───────►│bridge│◄───────┼──┼──┤VXLAN interface│   │ │
│  └────────────┘ │        └──┬───┘        │  │  └──────┬────────┘   │ │
│   192.168.1.20  │           │            │  │         │            │ │
│                 │           │            │  │         │            │ │
│                 │           │            │  │         │            │ │
│                 │           │            │  │         │            │ │
└─────────────────│───────────┼────────────│──│─────────┼────────────│─┘
                  │           │            │  │         │            │  
                  │           │            │  │         │            │  
                  │           │BGP EVPN    │  │         │VXLAN tunnel│  
                  │           │            │  │         │            │  
┌─────────────────│───────────┼────────────│──│─────────┼────────────│─┐
│ Node2           │           │            │  │         │            │ │
│                 │           │            │  │         │            │ │
│                 │           │            │  │         │            │ │
│   192.168.1.10  │           │            │  │         │            │ │
│  ┌────────────┐ │        ┌──┴───┐        │  │   ┌─────┴─────────┐  │ │
│  │VM interface├─┼───────►│bridge│◄───────┼──┼───┤VXLAN interface│  │ │
│  └────────────┘ │        └──────┘        │  │   └───────────────┘  │ │
│                 │                        │  │                      │ │
│        │        │           ▲            │  │           ▲          │ │
│        │        │           │            │  │           │          │ │
└────────┼────────│───────────┼────────────│──│───────────┼──────────│─┘
         │        │           │            │  │           │          │  
         │        └───────────┼────────────┘  └───────────┼──────────┘  
         │                    │                           │             
         │                    │                           │             
         │                    │                           │             
         │                    │                           │             
         │                                                │             
         │ ARP Request     Intercept ARP       ARP request│             
         └──────────────►  msg here and    ───────────────┘             
                           inject into bridge
```

 Traffic Flow Example (VM on Node2 pings VM on Node1):

 1. VM (192.168.1.10) sends ARP request "Who has 192.168.1.20?"
    * Bridge forwards to VXLAN interface (creates FDB entry on VXLAN port only)
    * VXLAN tunnels to Node2

 2. WITHOUT snoopy:
    - Bridge has NO neighbor entry for 192.168.1.10
    - FRR doesn't advertise 192.168.1.10 via EVPN
    - Other nodes don't know about this VM (and will flood packets to all other Nodes)
    - So: frames skip the "bridge" and go directly from "VM interface" to
      "VXLAN interface"

 3. WITH snoopy:
    - Snoopy detects ARP request from VM
    - Creates neighbor entry on bridge: 192.168.1.10 -> VM's MAC
    - FRR sees neighbor entry and advertises EVPN Type-2 route
    - Full EVPN fabric awareness achieved
    - So: we intercept the frames going from "VM interface" to "VXLAN
      interface" and add them manually to bridge

## Prerequisites

1. stable rust toolchains: `rustup toolchain install stable`
1. nightly rust toolchains: `rustup toolchain install nightly --component rust-src`
1. bpf-linker: `cargo install bpf-linker` (`--no-default-features` on macOS)

## Build & Run

Use `cargo build`, `cargo check`, etc. as normal. Run your program with:

```shell
cargo run --release
```

Cargo build scripts are used to automatically build the eBPF correctly and include it in the
program.

## Development

Aya requires rust nightly, so this won't work with the normal debian toolchain
we use. Maybe someone can figure out something smarter, but I simply have a
normal stable + nightly toolchain (installed with rustup), which I selective
activate by prefixing commands with `PATH=~/.cargo/bin/:$PATH` (I have a funky
fish script which does it for me and prefixes every command with this
override).

## Todo
- [ ] Snoop replies instead of requests (This protects us a bit against arp-spoofing because of arp_announce)
