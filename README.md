# Device services and orchestrator

This Cargo workspace contains two independently runnable services:

- `controller`: device service on `127.0.0.1:3000`, exposing `GET /device/name`.
- `orchestrator`: routes requests by device ID on `127.0.0.1:3001`, exposing
  `GET /devices/{id}/name`.

Both services have API, core, and infrastructure layers. The orchestrator's core
owns the `DeviceService` interface and selects a registered client. Its HTTP
infrastructure implements that interface, calling the selected service's
`/device/name`. `main.rs` loads configuration and wires the layers together.
The controller's code lives in `controller/src/`; the orchestrator's code lives
in `orchestrator/src/`. The root manifest only defines the workspace.
The controller currently uses a fake device.

## Run

From the repository root, start the device service:

```sh
cargo run -p controller
```

In another terminal, start the orchestrator:

```sh
cargo run -p orchestrator
```

Then request the configured device:

```sh
curl http://127.0.0.1:3001/devices/local/name
# {"name":"Fake Device"}
```

## Registry configuration

`CONTROLLER_ADDR` sets each controller's listen address (default
`127.0.0.1:3000`). To run two controllers locally, start these commands in
separate terminals:

```sh
CONTROLLER_ADDR=127.0.0.1:3000 cargo run -p controller
CONTROLLER_ADDR=127.0.0.1:3002 cargo run -p controller
```

`DEVICE_REGISTRY` selects a JSON registry file. It defaults to
`orchestrator/devices.json`, relative to the process's working directory.
`ORCHESTRATOR_ADDR` sets the listen address (default `127.0.0.1:3001`).

```sh
DEVICE_REGISTRY=/path/to/devices.json ORCHESTRATOR_ADDR=127.0.0.1:4000 cargo run -p orchestrator
```

Register the instances in that registry file, for example:

```json
[
  { "id": "local", "url": "http://127.0.0.1:3000" },
  { "id": "second", "url": "http://127.0.0.1:3002" }
]
```

Restart the orchestrator after changing its registry, then request either
`/devices/local/name` or `/devices/second/name`. Both controllers currently
return `Fake Device` because they use the same fake-device implementation.

IDs must be unique, nonempty, and contain only ASCII letters, digits, `_`, or `-`.
URLs must use plain HTTP; HTTPS, credentials, query strings, and fragments are
rejected. Optional path prefixes are supported: `http://host:3000/lab` calls
`http://host:3000/lab/device/name`. An empty registry is allowed. Invalid
configuration fails at startup; configuration changes require a restart.
Each device service must expose the existing `GET /device/name` JSON contract.

## Responses

| Status | JSON response | Meaning |
| --- | --- | --- |
| 200 | `{"name":"Fake Device"}` | Selected device's name |
| 404 | `{"error":"unknown device"}` | ID is not registered |
| 502 | `{"error":"device service request failed"}` | Connection failure, non-success status, malformed JSON, or body over 64 KiB |
| 504 | `{"error":"device service timed out"}` | Complete downstream call exceeded five seconds |

The orchestrator does not follow redirects or retry calls. The timeout covers
DNS resolution, connection setup, headers, and body reading.

API contracts: [controller](controller/contracts/openapi.yaml) and
[orchestrator](orchestrator/contracts/openapi.yaml).

## Verify

```sh
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```
