# Nebula Keybind Menu

A small Rust TUI for browsing keybinds. 

## Layout
- `Cargo.toml` / `Cargo.lock`: Rust crate definition.
- `src/main.rs`: All app logic (UI, event loop, config loading).
- `config.toml`: Example keybind list.

## Configuration
The app loads keybinds in this order:
1. `$XDG_CONFIG_HOME/nebula-keybind-menu/config.toml`
2. `~/.config/nebula-keybind-menu/config.toml`
3. `/usr/share/nebula-keybind-menu/config.toml`
4. Built-in defaults in `src/main.rs`

TOML format:

```toml
[[keybinds]]
keys = "SUPER + SPACE"
name = "Launcher"
desc = "Open app launcher"
```

## Build & Run
From this directory:

```bash
cargo build
cargo run
```

## Install
From this directory (local build):

```bash
makepkg -si
```

With the Nebula repo configured:

```bash
sudo pacman -S nebula-keybind-menu
```

## Controls
- Type to search
- `Esc` or `Ctrl+c` to quit
