# SPS

Linux/Windows/MacOS GUI Application to analyze stability-performance-scalability logs, powered by Rust, Svelte, DuckDB and WebKit!

Hand-Written FSM-powered, single-pass, zero-copy, multi-threaded parser that can purely parse logs totalling 1GB in 0.2s, and persists the entire result under a second.

# Usage

```bash
sps parse # Attempts to find sps logs in the current working directory, parses it and keeps the result in memory.
sps parse -d foo.db # Attempts to find sps logs in the current working directory, parses it and persists the results in foo.db
sps parse path/to/sps/directory -d foo.db # Attempts to parse path/to/sps/directory and persists the results in foo.db
sps launch # Launches the GUI, where you can parse a directory chosen from your file picker
sps launch -d foo.db # Launches the GUI with the results from the db
```

# Installation

## Ubuntu

1. Download the matching .deb for your Ubuntu version from [releases](https://github.com/bakageddy/sps/releases)
2. Execute the following command

```bash
sudo apt install ./sps_0.1.0_amd64_ubuntu-<version>.deb
```

3. Launch from command line with

```bash
sps launch
# or
sps
```

## MacOS M-Series

1. Download the .dmg file by copying the link from [releases](https://github.com/bakageddy/sps/releases) and executing:

```bash
curl -OL https://github.com/bakageddy/sps/releases/download/<tag>/sps_<version>_aarch64.dmg
```

2. Install either by using `open` in a terminal or with `finder`.

# Contributing

0. Strictly no AI on the rust based code, however the frontend is your playground
