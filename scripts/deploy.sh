#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$project_dir/target/aarch64-unknown-linux-gnu/debug/rust-sandbox"
remote_host="shalex@192.168.3.94"
destination="$remote_host:/home/shalex/rust-sandbox/rust-sandbox"

# Run this mode once in a WSL terminal, where password prompts are available.
if [[ "${1:-}" == "--setup-ssh" && $# -eq 1 ]]; then
    key="$HOME/.ssh/id_ed25519"
    if [[ ! -e "$key" && ! -e "$key.pub" ]]; then
        mkdir -p -- "$HOME/.ssh"
        chmod 700 -- "$HOME/.ssh"
        ssh-keygen -t ed25519 -f "$key"
    elif [[ ! -f "$key" || ! -f "$key.pub" ]]; then
        printf 'Incomplete SSH key pair at %s. Restore the missing file; existing files will not be overwritten.\n' "$key" >&2
        exit 1
    fi

    printf 'Installing the public key on %s. Enter your remote password if prompted.\n' "$remote_host"
    ssh-copy-id -F /dev/null -i "$key.pub" "$remote_host"
    if ssh -F /dev/null -o BatchMode=yes -o ConnectTimeout=10 "$remote_host" 'echo Connected'; then
        printf 'SSH setup complete. Run this script without arguments to deploy.\n'
    else
        printf 'Noninteractive SSH failed. If your key has a passphrase, load it into an SSH agent accessible to RustRover (ssh-add ~/.ssh/id_ed25519), then retry.\n' >&2
        exit 1
    fi
    exit 0
elif [[ $# -ne 0 ]]; then
    printf 'Usage: %s [--setup-ssh]\n' "$0" >&2
    exit 2
fi

if [[ ! -f "$binary" ]]; then
    printf 'Missing debug binary: %s\nRun cross-dev or cargo build --target aarch64-unknown-linux-gnu first.\n' "$binary" >&2
    exit 1
fi

printf 'Uploading %s to %s\n' "$binary" "$destination"
# Match the README workaround for this WSL environment's SSH configuration.
# Batch mode requires an SSH key/agent; IDE-saved passwords are not available.
if scp -F /dev/null -B -o ConnectTimeout=10 "$binary" "$destination"; then
    printf 'Deployment complete.\n'
else
    printf 'Deployment failed. For SSH authentication setup, run: bash "%s" --setup-ssh\n' "${BASH_SOURCE[0]}" >&2
    printf 'Also check the destination directory and that the remote binary is not running.\n' >&2
    exit 1
fi

printf 'Starting gdbserver on %s:2345...\n' "$remote_host"
if ! ssh -F /dev/null -o BatchMode=yes -o ConnectTimeout=10 "$remote_host" \
    'bash -s -- /home/shalex/rust-sandbox' <<'REMOTE'
set -euo pipefail
cd -- "$1"
command -v gdbserver >/dev/null || {
    printf 'gdbserver is not installed on the target.\n' >&2
    exit 1
}
chmod +x ./rust-sandbox

# Detach from SSH so the before-launch task can finish and GDB can connect.
# Do not probe the TCP port: that would consume the debugger connection.
LC_ALL=C nohup gdbserver --once :2345 ./rust-sandbox \
    >gdbserver.log 2>&1 </dev/null &
server_pid=$!
for ((attempt = 0; attempt < 100; attempt++)); do
    if ! kill -0 "$server_pid" 2>/dev/null; then
        cat gdbserver.log >&2
        printf 'gdbserver exited before becoming ready.\n' >&2
        exit 1
    fi
    if grep -q '^Listening on port 2345' gdbserver.log; then
        printf 'gdbserver is ready on port 2345 (PID %s).\n' "$server_pid"
        exit 0
    fi
    sleep 0.1
done
kill "$server_pid" 2>/dev/null || true
cat gdbserver.log >&2
printf 'Timed out waiting for gdbserver to listen on port 2345.\n' >&2
exit 1
REMOTE
then
    printf 'Remote debugger startup failed. Check the log and whether port 2345 is already in use.\n' >&2
    exit 1
fi
printf 'Ready for RustRover. Target output: /home/shalex/rust-sandbox/gdbserver.log\n'

