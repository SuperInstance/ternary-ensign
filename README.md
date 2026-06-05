# ternary-ensign: Specialist agent pattern with domain registry and bridging

An ensign is a domain specialist that handles tasks within its area of expertise. The registry loads and unloads specialists. The factory creates them on demand. The proxy manages API keys. The bridge maps domains to skills.

## Why This Exists

Not every agent needs to do everything. In a ternary ecosystem, specialists are loaded into rooms based on what the room needs — a navigation room gets a navigation ensign, a comms room gets a comms ensign. This crate formalizes that pattern, inspired by naval ensigns (junior officers with specific duties) and by the git-agent model where each room loads domain-specific skills.

## Core Concepts

- **Ensign (trait)** — A domain specialist. Implement `domain()` to declare your specialty and `handle(task)` to process tasks. Returns an `EnsignResult` (success/fail + output string).
- **EnsignRegistry** — A HashMap of domain → ensign. Load specialists in, unload them, dispatch tasks to the right domain.
- **EnsignFactory** — Registers builder functions (`Fn() -> Box<dyn Ensign>`) by domain name. Create ensigns on demand without knowing the concrete type.
- **EnsignProxy** — Manages API keys and session tokens per domain. Tracks whether a domain is "authenticated" (has both key and session). Does not make network calls.
- **EnsignBridge** — Maps domain names to skill names (like construct-core skills). Records an invocation log of (domain, task) pairs for auditing.
- **EchoEnsign / NullEnsign** — Built-in implementations for testing. EchoEnsign prefixes output with its domain; NullEnsign always fails.

## Quick Start

```toml
[dependencies]
ternary-ensign = "0.1"
```

```rust
use ternary_ensign::*;

// Create a registry and load a specialist
let mut reg = EnsignRegistry::new();
reg.load(Box::new(EchoEnsign::new("navigation")));

// Dispatch a task
let result = reg.dispatch("navigation", "plot course to sector 7");
assert!(result.success);
assert_eq!(result.output, "[navigation] plot course to sector 7");

// Use the factory for on-demand creation
let mut factory = EnsignFactory::new();
factory.register("comms", || Box::new(EchoEnsign::new("comms")));
let ensign = factory.create("comms").unwrap();
assert_eq!(ensign.domain(), "comms");
```

## API Overview

| Type | Description |
|------|-------------|
| `Ensign` (trait) | Domain specialist: `domain()` + `handle(task)`. |
| `EnsignResult` | success bool + output string. |
| `EchoEnsign` | Built-in: echoes task with domain prefix. |
| `NullEnsign` | Built-in: always returns error. |
| `EnsignRegistry` | HashMap of domain → ensign with dispatch. |
| `EnsignFactory` | Builder functions registered by domain, creates ensigns. |
| `EnsignProxy` | API key + session token management per domain. |
| `EnsignBridge` | Domain → skill mapping with invocation log. |

## How It Works

The registry pattern is straightforward: a `HashMap<String, Box<dyn Ensign>>` keyed by domain name. Load replaces any existing ensign for that domain (returning the old one). Dispatch looks up the domain and calls `handle`.

The factory stores closures that produce ensigns. This decouples creation from configuration — register a closure once, call `create("domain")` whenever you need a fresh instance.

The proxy manages two maps: keys and sessions. A domain is "authenticated" when it has both. Key rotation replaces the old key and returns it.

The bridge maps domain strings to skill strings and logs every invocation. It's a routing layer, not an execution layer — it tells you which skill to invoke, but doesn't invoke it.

## Known Limitations

- Ensign trait takes `&str` task input and returns string output — no structured data. For complex payloads, serialize to strings.
- EnsignRegistry is not thread-safe. Wrap in `Mutex` for concurrent access.
- EnsignProxy stores keys in plain memory. No encryption at rest.
- The bridge invocation log grows unbounded — in long-running simulations, clear it periodically or it will consume memory.
- Factory builder closures are boxed and type-erased, so you can't inspect what a factory will create without calling it.

## Use Cases

- **Room-specific skills** — Load a navigation ensign into navigation rooms, a combat ensign into arena rooms. Unload when the room changes purpose.
- **Plugin system** — Each plugin implements Ensign. The registry discovers and loads them at startup.
- **API gateway** — EnsignProxy tracks which external services have valid credentials. Dispatch only to authenticated domains.
- **Git-agent specialists** — Each git-agent is an ensign loaded per room. The bridge maps agent domains to construct-core skills.

## Ecosystem Context

`ternary-ensign` is independent of other ternary crates. It uses only `std`. Conceptually, ensigns are loaded into rooms from `ternary-room` and driven by agents from `ternary-agent`. The bridge connects to construct-core's skill system. In `ternary-world`, ensigns could serve as agent behaviors.

## License

MIT

## See Also
- **ternary-captain** — related fleet coordination
- **ternary-room** — related fleet coordination
- **ternary-channel** — related fleet coordination
- **ternary-beacon** — related fleet coordination
- **ternary-bridge** — related fleet coordination
- **ternary-protocol** — related fleet coordination

