# rust-sandbox

Build natively in WSL or cross-build for ARM64 Linux
(`shalex@192.168.3.94`, glibc 2.39).

## Cross-build setup in WSL

```sh
sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu libc6-dev-arm64-cross
rustup target add aarch64-unknown-linux-gnu
```

## Build

In RustRover, use the WSL Rust toolchain, select a configuration below,
and click **Run**. Reload the project if a configuration is missing.

| RustRover configuration | Cargo command | Output |
| --- | --- | --- |
| Native build | `cargo build --release` | `target/release/rust-sandbox` |
| Cross build ARM64 Linux | `cargo build --release --target aarch64-unknown-linux-gnu` | `target/aarch64-unknown-linux-gnu/release/rust-sandbox` |

Native means the selected toolchain's host (x86-64 Linux for this WSL setup).
`.cargo/config.toml` sets only the ARM64 linker; it does not change the default
build target. Use `cargo run` to build and run locally.

The current ARM64 binary requires glibc 2.34. After toolchain or dependency updates,
check with `readelf --version-info` that it still requires no version above
the server's 2.39.

## Copy and run

```sh
scp -F /dev/null target/aarch64-unknown-linux-gnu/release/rust-sandbox shalex@192.168.3.94:~/rust-sandbox
ssh -F /dev/null shalex@192.168.3.94 './rust-sandbox'
```

SSH authentication must be configured. `-F /dev/null` bypasses the broken system
SSH configuration in this WSL environment; omit it once that is fixed.

## Deploy a debug build from RustRover

`scripts/deploy.sh` uploads the existing ARM64 debug executable to
`shalex@192.168.3.94:/home/shalex/rust-sandbox/rust-sandbox`. The remote directory
must exist. Stop the previous debug process before replacing its executable.
After uploading, the script starts `gdbserver --once :2345 ./rust-sandbox` in
the background on the target and waits up to 10 seconds for its listening
message. Startup failure stops the before-launch task.
The script uses batch-mode SSH and requires a key or agent available inside WSL;
it cannot use RustRover's saved SSH password.

For one-time SSH authentication setup, run in a WSL terminal:

```sh
bash scripts/deploy.sh --setup-ssh
```

This reuses `~/.ssh/id_ed25519` or interactively creates it if absent, installs
the public key using your remote password, and verifies batch-mode login.
Existing key files are never overwritten. If the key has a passphrase, its SSH
agent must also be accessible to the RustRover External Tool.

Add an External Tool in RustRover with:

- **Program:** `C:\Windows\System32\wsl.exe`
- **Arguments:** `-d Ubuntu-26.04 -- bash /home/shalex/dev/projects/rust-sandbox/scripts/deploy.sh`
- **Working directory:** `C:\Windows`
- Enable **Open console for tool output**.

In `remote-debug` → **Before launch**, put **Run configuration: cross-dev** first,
then **Run External tool** with this tool. Once the server is ready, the tool
returns so RustRover can connect its GDB client. Do not start another gdbserver
manually on port 2345.

Target stdout/stderr and gdbserver messages are written to
`/home/shalex/rust-sandbox/gdbserver.log` (replaced each launch). To view them in
RustRover's SSH terminal, run:

```sh
tail -F /home/shalex/rust-sandbox/gdbserver.log
```