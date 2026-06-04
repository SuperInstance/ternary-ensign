# Future Integration: ternary-ensign

## Current State

ternary-ensign implements the specialist agent pattern inspired by naval ensigns. The `Ensign` trait defines `domain()` and `handle(task) → EnsignResult`. `EnsignRegistry` manages load/unload of ensigns by domain with `dispatch()` routing. `EnsignFactory` creates ensigns on demand via registered builder functions. `EnsignProxy` manages API keys per domain with key rotation, session binding, and `is_authenticated()` checks. `EnsignBridge` maps ensign domains to construct-core skill names with invocation logging.

## Integration Opportunities

### Git-Agent Army (Primary Use Case)

The `EnsignFactory` pattern enables a git-agent army: register builders for every specialist domain, then create ensigns on demand when agents enter rooms. A fleet of `EchoEnsign`-like specialists (but with real domain logic) handles tasks:

- **engine-monitor ensign**: `domain() = "engine"`, `handle()` runs ternary-kalman state estimation
- **fleet-coord ensign**: `domain() = "fleet"`, `handle()` runs ternary-consensus agreement protocol
- **compiler ensign**: `domain() = "compiler"`, `handle()` runs ternary-compiler strategy compilation

`EnsignFactory::register()` stores the builder, `create()` instantiates on room entry, `EnsignRegistry::unload()` removes on room exit.

### EnsignProxy → PLATO API Key Flow

`EnsignProxy` exactly models the PLATO proxy pattern from ROOM-AS-CODESPACE-ARCHITECTURE.md:

1. PLATO holds master API keys (`EnsignProxy::set_key("llm", master_key)`)
2. When a Codespace room is created, PLATO provisions a proxy endpoint (`EnsignProxy::bind_session("llm", session_token)`)
3. The ensign checks `is_authenticated("llm")` before making LLM calls
4. Key rotation (`rotate_key()`) refreshes compromised keys without restarting the room

Current `EnsignProxy` is in-memory. For production, it needs network transport: `EnsignProxy::forward_request(domain, request) → Response` that calls through to PLATO's proxy endpoint.

### EnsignBridge → construct-core Skills

`EnsignBridge::map("engine", "ternary_kalman_skill")` connects ensign domains to `ternary-registry` skill IDs. When `EnsignBridge::invoke()` is called, it routes to the construct-core `load_skill()` / `query_owned()` pipeline. The bridge's invocation log (`log()`) provides provenance tracking — every skill invocation is recorded with domain and task.

### Muscle Memory / Trigger Extraction

The ensign pattern needs trigger extraction on unload. When `EnsignRegistry::unload("engine")` is called, the ensign should extract lightweight thresholds that fire when the specialist should be reloaded. Adding `trait Ensign: extract_triggers(&self) → Vec<Trigger>` where `Trigger { metric, threshold, action }` enables the muscle-memory pattern from the architecture doc.

## Potential in Mature Systems

In a mature room-as-codespace deployment, every room loads 1-3 ensigns on entry and unloads them on exit. The `EnsignRegistry` tracks active specialists. `EnsignFactory` provides lazy instantiation — ensigns are only built when needed, not pre-loaded. The `EnsignBridge` ensures every ensign invocation goes through the construct-core skill system, maintaining the hardware tier abstraction. The invocation log becomes the audit trail for compliance and debugging.

## Cross-Pollination Ideas

- **EnsignRegistry → ternary-registry SkillRegistry**: Unify the two registries. Every `Ensign` IS a `Skill` with a `domain` capability. `EnsignRegistry::load()` = `SkillRegistry::register()`.
- **EnsignFactory → linguistic-polyformalism**: Use the 7 constraint types (Boundary, Pattern, Process Shape, etc.) to auto-generate ensign capability declarations. Each ensign's `domain()` declares which constraint types it understands.
- **EnsignProxy → ternary-protocol**: API key delivery via ternary-protocol messages. Key rotation = broadcast Suppress to old key, Signal to new key.

## Dependencies for Next Steps

1. `Ensign::extract_triggers()` method addition to the trait
2. `EnsignProxy::forward_request()` with network transport
3. `EnsignBridge` → `SkillRegistry` unification
4. Real domain ensigns (engine-monitor, fleet-coord, compiler) implementing the trait
5. Cost/benefit scoring: `Ensign::cost() → EnsignCost` for budget management
