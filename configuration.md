# Configuration

## Reason to have configuration

This program can be used without a configuration file. But you may want to have a configuration file to:

- Set up a keyboard layout for a specific window classes

## Configuration file

Create a file
~/.config/niri-per-window-layout/options.toml

Example configuration file:

```toml
# layout_index => window classes list
# use `niri msg windows` to get ids
[[default_layouts]]
1 = [
    "org.telegram.desktop",
]
```

This example will set your second layout for the Telegram by default.

1 - is a layout index. In case of this input configuration:
```
input {
  keyboard {
    xkb {
      layout "us, ua, il"
    }
  }
}
```
*us* index is 0, *ua* index is 1, *il* index is 2.


```toml
# layout_index => window ids list
# use `niri msg windows` to get class names
[[default_layouts]]
1 = [
    "org.telegram.desktop",
    "discord",
]
2 = [
    "firefox",
]
```
