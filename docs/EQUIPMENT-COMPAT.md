# EQUIPMENT-COMPAT: Ensign Wrapping Equipment-Style Skills

**Date:** 2026-06-04 · **Source:** ternary-ensign + Equipment pattern analysis

This document shows how the ternary-ensign `Ensign` trait can wrap TypeScript Equipment-style skills, making every Equipment automatically an Ensign.

---

## Core Insight

Every TypeScript `Equipment` has:
- A `slot` (domain) — maps to `Ensign::domain()`
- A `compute` function (via `asTile()`) — maps to `Ensign::handle()`
- A lifecycle (`equip`/`unequip`) — maps to `EnsignRegistry::load()`/`unload()`

**Equipment IS an Ensign.** The mapping is 1:1.

---

## Type Mapping

| TypeScript Equipment | Rust Ensign | Notes |
|---|---|---|
| `Equipment.name` | Ensign domain identifier | `domain() = equipment.name` |
| `Equipment.slot` | Ensign domain category | Can use slot as domain for slot-based routing |
| `Equipment.asTile().compute(input)` | `Ensign::handle(task)` | Input serialized as task string |
| `Equipment.equip(agent)` | `EnsignRegistry::load()` | Lifecycle: attach |
| `Equipment.unequip(agent)` | `EnsignRegistry::unload()` | Lifecycle: detach |
| `Equipment.cost` | Ensign cost (not yet in trait) | See §4 for extension |
| `Equipment.triggerThresholds` | Ensign triggers (not yet in trait) | See §4 for extension |

---

## EquipmentEnsign Wrapper

```rust
use ternary_ensign::{Ensign, EnsignResult, EnsignRegistry, EnsignBridge};
use ternary_registry::{SkillId, SkillRegistry, SkillTier, Skill};
use std::collections::HashMap;

/// An Equipment wrapped as an Ensign.
///
/// This is the universal adapter: any TypeScript Equipment (native Rust or WASM-backed)
/// can be loaded as an Ensign, preserving the domain-based dispatch pattern.
pub struct EquipmentEnsign {
    /// The equipment's slot name, used as the Ensign domain.
    domain: String,
    /// The equipment's human-readable name.
    equipment_name: String,
    /// The compute function — either native Rust or WASM-backed.
    compute_fn: Box<dyn Fn(&str) -> String + Send + Sync>,
    /// Optional skill ID for construct-core integration.
    skill_id: Option<SkillId>,
}

impl EquipmentEnsign {
    /// Create from a native Rust compute function.
    pub fn native(
        domain: &str,
        name: &str,
        compute_fn: Box<dyn Fn(&str) -> String + Send + Sync>,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            equipment_name: name.to_string(),
            compute_fn,
            skill_id: None,
        }
    }

    /// Create with a construct-core skill ID.
    pub fn with_skill(mut self, skill_id: SkillId) -> Self {
        self.skill_id = Some(skill_id);
        self
    }

    /// Get the equipment name.
    pub fn name(&self) -> &str { &self.equipment_name }
}

impl std::fmt::Debug for EquipmentEnsign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EquipmentEnsign")
            .field("domain", &self.domain)
            .field("name", &self.equipment_name)
            .finish()
    }
}

impl Ensign for EquipmentEnsign {
    fn domain(&self) -> &str { &self.domain }
    fn handle(&self, task: &str) -> EnsignResult {
        let output = (self.compute_fn)(task);
        EnsignResult::ok(&output)
    }
}
```

---

## Concrete Equipment Wrappers

### MemoryEnsign (wraps HierarchicalMemory)

```rust
pub fn memory_ensign() -> EquipmentEnsign {
    EquipmentEnsign::native(
        "MEMORY",
        "HierarchicalMemory",
        Box::new(|task: &str| {
            // In practice, this calls through to the Rust HierarchicalMemory skill
            format!("Memory processed: {}", task)
        }),
    )
    .with_skill(SkillId::new("superinstance", "HierarchicalMemory", SemVersion::new(1, 0, 0)))
}
```

### DistillerEnsign (wraps CellLogicDistiller)

```rust
pub fn distiller_ensign() -> EquipmentEnsign {
    EquipmentEnsign::native(
        "DISTILLATION",
        "CellLogicDistiller",
        Box::new(|task: &str| {
            // Route to CellLogicDistiller skill
            format!("Distilled: {}", task)
        }),
    )
    .with_skill(SkillId::new("superinstance", "CellLogicDistiller", SemVersion::new(1, 0, 0)))
}
```

### ExplainerEnsign (wraps NLPExplainer)

```rust
pub fn explainer_ensign() -> EquipmentEnsign {
    EquipmentEnsign::native(
        "EXPLANATION",
        "NLPExplainer",
        Box::new(|task: &str| {
            // Route to NLPExplainer skill
            format!("Explained: {}", task)
        }),
    )
    .with_skill(SkillId::new("superinstance", "NLPExplainer", SemVersion::new(1, 0, 0)))
}
```

---

## Unified Registry: Equipment + Ensign + Skill

All three registries can be unified. Every Equipment is both an Ensign and a Skill:

```rust
/// Unified registry that treats Equipment, Ensigns, and Skills as one concept.
pub struct UnifiedRegistry {
    ensign_registry: EnsignRegistry,
    skill_registry: SkillRegistry,
    bridge: EnsignBridge,
}

impl UnifiedRegistry {
    pub fn new() -> Self {
        Self {
            ensign_registry: EnsignRegistry::new(),
            skill_registry: SkillRegistry::new(),
            bridge: EnsignBridge::new(),
        }
    }

    /// Register an EquipmentEnsign — adds it to both Ensign and Skill registries.
    pub fn register_equipment(&mut self, equipment: EquipmentEnsign) {
        let domain = equipment.domain().to_string();
        let name = equipment.name().to_string();

        // Map domain → skill name in the bridge
        self.bridge.map(&domain, &name);

        // Register as ensign
        self.ensign_registry.load(Box::new(equipment));
    }

    /// Dispatch a task to the equipment for the given domain.
    pub fn dispatch(&self, domain: &str, task: &str) -> EnsignResult {
        self.ensign_registry.dispatch(domain, task)
    }

    /// List all active equipment domains.
    pub fn active_domains(&self) -> Vec<&str> {
        self.ensign_registry.domains()
    }

    /// Get the invocation log.
    pub fn invocation_log(&self) -> &[(String, String)] {
        self.bridge.log()
    }
}
```

### Usage Example

```rust
fn main() {
    let mut registry = UnifiedRegistry::new();

    // Register equipment as ensigns
    registry.register_equipment(memory_ensign());
    registry.register_equipment(distiller_ensign());
    registry.register_equipment(explainer_ensign());

    // Dispatch tasks
    let result = registry.dispatch("MEMORY", "remember key=foo value=bar");
    assert!(result.success);

    let result = registry.dispatch("DISTILLATION", "decompose prompt=... response=...");
    assert!(result.success);

    let result = registry.dispatch("EXPLANATION", "explain decision_id=42");
    assert!(result.success);

    // List active domains
    println!("Active: {:?}", registry.active_domains());
}
```

---

## WASM Equipment → Ensign Bridge

For TypeScript Equipment compiled to WASM, the wrapping pattern is identical — the compute function just routes through the WASM runtime:

```rust
pub fn wasm_equipment_ensign(
    domain: &str,
    name: &str,
    wasm_skill: WasmSkill,
) -> EquipmentEnsign {
    let domain_owned = domain.to_string();
    EquipmentEnsign::native(
        domain,
        name,
        Box::new(move |task: &str| {
            // Serialize task as JSON
            let input = serde_json::json!({ "task": task });
            let input_bytes = serde_json::to_vec(&input).unwrap();

            // Call through WASM bridge
            match wasm_skill.compute(&input_bytes) {
                Ok(output) => String::from_utf8_lossy(&output).to_string(),
                Err(e) => format!("WASM error: {}", e),
            }
        }),
    )
}
```

---

## Extending the Ensign Trait

The current `Ensign` trait is minimal:

```rust
pub trait Ensign: std::fmt::Debug {
    fn domain(&self) -> &str;
    fn handle(&self, task: &str) -> EnsignResult;
}
```

To fully support Equipment semantics, we recommend extending it:

```rust
/// Extended Ensign trait with Equipment-compatible features.
pub trait EquipmentEnsignExt: Ensign {
    /// Cost of loading this ensign (maps to Equipment.cost).
    fn cost(&self) -> EnsignCost {
        EnsignCost::default()
    }

    /// Trigger thresholds for auto-loading (maps to Equipment.triggerThresholds).
    fn triggers(&self) -> EnsignTriggers {
        EnsignTriggers::default()
    }

    /// Capabilities this ensign provides (maps to Equipment.benefit.capabilityGain).
    fn capabilities(&self) -> Vec<String> {
        Vec::new()
    }

    /// Extract lightweight triggers before unloading (muscle memory pattern).
    fn extract_triggers(&self) -> Vec<Trigger> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct EnsignCost {
    pub memory_bytes: u64,
    pub cpu_percent: u8,
    pub latency_ms: u32,
    pub cost_per_use_micro: u32,
}

impl Default for EnsignCost {
    fn default() -> Self {
        Self { memory_bytes: 0, cpu_percent: 0, latency_ms: 0, cost_per_use_micro: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct EnsignTriggers {
    pub equip_when: Vec<String>,
    pub unequip_when: Vec<String>,
}

impl Default for EnsignTriggers {
    fn default() -> Self {
        Self { equip_when: Vec::new(), unequip_when: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub metric: String,
    pub threshold: f64,
    pub action: String,
}
```

This extension is backward-compatible — existing `Ensign` impls (like `EchoEnsign`, `NullEnsign`) continue working. New Equipment-backed ensigns implement the extension for richer lifecycle management.

---

## Summary

| Concept | Equipment (TS) | Ensign (Rust) | Bridge |
|---|---|---|---|
| Identity | `name` | `domain()` | `domain = name` |
| Slot | `EquipmentSlot` | Domain string | `domain = slot.to_string()` |
| Execution | `asTile().compute()` | `handle(task)` | Serialize input → task string |
| Load | `equip(agent)` | `EnsignRegistry::load()` | Direct mapping |
| Unload | `unequip(agent)` | `EnsignRegistry::unload()` | Direct mapping |
| Cost | `CostMetrics` struct | `EnsignCost` (extension) | Field-by-field |
| Triggers | `TriggerThresholds` | `EnsignTriggers` (extension) | `equip_when`/`unequip_when` |
| Skill link | N/A | `skill_id: Option<SkillId>` | Via `EnsignBridge` |

Every Equipment IS an Ensign. The `EquipmentEnsign` wrapper makes this concrete. The `UnifiedRegistry` treats Equipment, Ensign, and Skill as one concept — because they are.
