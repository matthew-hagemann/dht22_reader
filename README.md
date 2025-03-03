# Welcome to my DHT22 Reader!
Don't run this code :)

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

### Install libgpiod

```
git clone https://github.com/brgl/libgpiod.git
cd libgpiod
```

### Generate configuration scripts 

```
./autogen.sh
```

### Prep the build system, set installtion to `/usr`

```
./configure --prefix=/usr
```

### Compile, use all cores

```
make -j$(nproc)
```

### Install the binaries

```
sudo make install
```
