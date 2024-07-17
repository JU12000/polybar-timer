# About:

A configurable CLI timer which can be tailed to the command line. Built for use in Polybar but works in anything that accepts stdout.

## Installation:

### Requirements:
- [polybar](https://github.com/polybar/polybar)

1. Download the latest release and place `polybar-timer` in `~/.config/polybar/scripts/`.
2. Add it to your `polybar.config`

```ini
[module/polybar-timer]
type = custom/script
tail = true
exec = ~/.config/polybar/scripts/polybar-timer tail

; Configure these with your preferred arguments. A help interface is built in.
click-left = ~/.config/polybar/scripts/polybar-timer toggle
click-middle = ~/.config/polybar/scripts/polybar-timer cancel
click-right = ~/.config/polybar/scripts/polybar-timer new 25
scroll-up = ~/.config/polybar/scripts/polybar-timer increase 60
```
3. Don't forget to add the module to your bar!

### Configuration:

To view configuration options run `polybar-timer -h` and `polybar-timer [COMMAND] -h`

### Building:

1. Clone this repository
2. `cargo build` or `cargo build --release` to create the debug and release versions respectively.
3. Build results are in `target/[debug|release]/polybar-timer`
