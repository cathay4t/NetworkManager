# NetworkManager Rust Rewrite

## Binaries and Libraries

 * `nmcli`: CLI tool for communicating with daemon
 * `NetworkManager`: The daemon
 * `nm`: Rust crate for IPC communicating with NetworkManager daemon
 * `nmstate`: Schema for desired state
 * `nmstatectl`: Nmstate backward compatibility CLI

## Features
 * Daemon Free Mode
 * Simply Plugin Design

## Run Server

```bash
env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo' \
    cargo run --bin NetworkManager
```

## Run Client

```bash
env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo' \
    cargo run --bin nmcli
```
