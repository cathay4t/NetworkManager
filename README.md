# NetworkManager Rust Rewrite

## Binaries and Libraries

 * `cli`: CLI tool for communicating with daemon -- `nmc`
 * `daemon`: The daemon -- `NetworkManager`
 * `nm`: Rust crate for daemon communication and daemon free actions
 * `python-libnm`: Python APi for daemon communication
 * `libnm-plugin`: Rust crate for plugin interface

## Features
 * Daemon free mode
 * Simply plugin design
 * Native support of [Nmstate][nmstate_url] schema

## License
 * The daemon `NetworkManager` is licensed under `GPL-3.0-or-later`.
 * The CLI `nmc` is licensed under `GPL-3.0-or-later`.
 * Others are licensed under 'Apache-2.0' license.

Please check `LICENSE-GPL` and `LICENSE-APACHE` files for detail.

## Run Server

```bash
env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo' \
    cargo run --bin NetworkManager
```

## Run Client

```bash
env CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo' \
    cargo run --bin nmc
```

[nmstate_url]: https://nmstate.io/
