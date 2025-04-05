# Welcome to my DHT22 Reader!
Don't run this code :)

# Setup

I am using a RPI 5 on Ubuntu 25.04. This is becuase of a dependency on libgpiod-dev >=2.2.

# Installing dependencies

## Install Rust

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install C dependencies 

```
sudo apt update
sudo apt install build-essential gcc clang libclang-dev libgpiod-dev autoconf automake libtool pkg-config autoconf-archive
```

# Regenerate bindings

```
cargo build --features generate-bindings
```

# Debugging

gpio lines are tricky to debug. Using the cli tools included in libgpiod-dev:


```
# Print activity on chip 0, line 4
gpionotify -c gpiochip0 -F '%U: Line: %o, event: %E' 4
```

This will produce:
```
$ gpionotify -c gpiochip0 -F '%U: Line: %o, event: %E' 4
2025-04-05T09:22:28.995295714Z: Line: 4, event: requested
2025-04-05T09:22:28.996478820Z: Line: 4, event: reconfigured
2025-04-05T09:22:28.996715893Z: Line: 4, event: released
```

More info at: https://libgpiod.readthedocs.io/en/latest/gpionotify.html
