[![Stand With Ukraine](https://raw.githubusercontent.com/vshymanskyy/StandWithUkraine/main/banner2-direct.svg)](https://stand-with-ukraine.pp.ua)

# Niri Per-Window Keyboard Layout Manager

Automatic keyboard layout switching for Niri - each window remembers its own keyboard layout.
> [!NOTE]
> This is a fork of https://github.com/coffebar/hyprland-per-window-layout.
> This is only an (unneeded) adaptation for Niri.
> Most credit for the original great project goes to [coffeebar](https://github.com/coffebar)

> [!WARNING]
> This behavior is already included in Niri by default:
> https://yalter.github.io/niri/Configuration%3A-Input.html#layout

## Features

- 🚀 **Zero configuration** - works out of the box
- 🧠 **Per-window memory** - each window maintains its layout
- ⚡ **Lightweight** - minimal resource usage (Rust)
- 🔧 **Optional configuration** - set default layouts per application

## Use Cases

- **Developers**: Code in English, chat in native language
- **Multilingual users**: Seamless switching between languages
- **Power users**: Consistent layouts across applications

**Requirements**: At least 2 keyboard layouts in niri/config.kdl

## Installation

### From Source

```bash
git clone https://github.com/wadsaek/niri-per-window-layout.git
cd niri-per-window-layout
cargo build --release
mkdir -p ~/.local/bin/
cp target/release/niri-per-window-layout ~/.local/bin/
```

Add to config.kdl:
```
spawn-at-startup "~/.local/bin/niri-per-window-layout"
```

## Configuration

Optional. See [configuration.md](configuration.md) for setting default layouts per application.

## Contributing

Bug reports and PRs are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Tested on niri v25.08
