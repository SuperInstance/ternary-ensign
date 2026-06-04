#![forbid(unsafe_code)]

//! Specialist agent pattern inspired by naval ensigns.
//!
//! An `Ensign` is a domain specialist that can be loaded into a room on demand.
//! The `EnsignRegistry` tracks available specialists. `EnsignFactory` creates
//! them. `EnsignProxy` manages API keys via an external session. `EnsignBridge`
//! connects to a construct-core skills system.

use std::collections::HashMap;

// ── Ensign Trait ───────────────────────────────────────────────────────────

/// A domain specialist that can handle tasks of a given type.
///
/// Implementations encapsulate domain knowledge and return results as strings.
pub trait Ensign: std::fmt::Debug {
    /// The domain this specialist covers (e.g. "navigation", "comms").
    fn domain(&self) -> &str;

    /// Handle a task within this specialist's domain.
    fn handle(&self, task: &str) -> EnsignResult;
}

/// Result from an ensign handling a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsignResult {
    pub success: bool,
    pub output: String,
}

impl EnsignResult {
    pub fn ok(output: &str) -> Self {
        Self { success: true, output: output.to_string() }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, output: msg.to_string() }
    }
}

// ── Built-in Ensigns ──────────────────────────────────────────────────────

/// A generic ensign that echoes tasks back with its domain prefixed.
#[derive(Debug)]
pub struct EchoEnsign {
    domain: String,
}

impl EchoEnsign {
    pub fn new(domain: &str) -> Self {
        Self { domain: domain.to_string() }
    }
}

impl Ensign for EchoEnsign {
    fn domain(&self) -> &str {
        &self.domain
    }

    fn handle(&self, task: &str) -> EnsignResult {
        EnsignResult::ok(&format!("[{}] {}", self.domain, task))
    }
}

/// An ensign that always fails — useful as a placeholder.
#[derive(Debug)]
pub struct NullEnsign {
    domain: String,
}

impl NullEnsign {
    pub fn new(domain: &str) -> Self {
        Self { domain: domain.to_string() }
    }
}

impl Ensign for NullEnsign {
    fn domain(&self) -> &str {
        &self.domain
    }

    fn handle(&self, _task: &str) -> EnsignResult {
        EnsignResult::err("not implemented")
    }
}

// ── Ensign Registry ────────────────────────────────────────────────────────

/// A registry of available ensigns that can be loaded and unloaded by domain.
#[derive(Debug)]
pub struct EnsignRegistry {
    ensigns: HashMap<String, Box<dyn Ensign>>,
}

impl EnsignRegistry {
    pub fn new() -> Self {
        Self { ensigns: HashMap::new() }
    }

    /// Register an ensign. Replaces any existing ensign for the same domain.
    /// Returns the previous ensign if one existed.
    pub fn load(&mut self, ensign: Box<dyn Ensign>) -> Option<Box<dyn Ensign>> {
        self.ensigns.insert(ensign.domain().to_string(), ensign)
    }

    /// Remove an ensign by domain. Returns true if it was present.
    pub fn unload(&mut self, domain: &str) -> bool {
        self.ensigns.remove(domain).is_some()
    }

    /// Look up an ensign by domain.
    pub fn get(&self, domain: &str) -> Option<&dyn Ensign> {
        self.ensigns.get(domain).map(|b| b.as_ref())
    }

    /// Dispatch a task to the ensign for the given domain.
    /// Returns an error result if no ensign is registered for that domain.
    pub fn dispatch(&self, domain: &str, task: &str) -> EnsignResult {
        match self.ensigns.get(domain) {
            Some(e) => e.handle(task),
            None => EnsignResult::err(&format!("no ensign for domain '{}'", domain)),
        }
    }

    /// List all registered domains.
    pub fn domains(&self) -> Vec<&str> {
        self.ensigns.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered ensigns.
    pub fn len(&self) -> usize {
        self.ensigns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ensigns.is_empty()
    }
}

impl Default for EnsignRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Ensign Factory ─────────────────────────────────────────────────────────

/// A factory that creates ensigns on demand by domain name.
///
/// Register builder functions, then create ensigns by name.
pub struct EnsignFactory {
    builders: HashMap<String, Box<dyn Fn() -> Box<dyn Ensign>>>,
}

impl EnsignFactory {
    pub fn new() -> Self {
        Self { builders: HashMap::new() }
    }

    /// Register a builder function for a domain.
    pub fn register<F>(&mut self, domain: &str, builder: F)
    where
        F: Fn() -> Box<dyn Ensign> + 'static,
    {
        self.builders.insert(domain.to_string(), Box::new(builder));
    }

    /// Create an ensign for the given domain. Returns None if no builder registered.
    pub fn create(&self, domain: &str) -> Option<Box<dyn Ensign>> {
        self.builders.get(domain).map(|b| b())
    }

    /// List domains with registered builders.
    pub fn available(&self) -> Vec<&str> {
        self.builders.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for EnsignFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EnsignFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnsignFactory")
            .field("domains", &self.builders.keys().collect::<Vec<_>>())
            .finish()
    }
}

// ── Ensign Proxy ───────────────────────────────────────────────────────────

/// API key management via an external session.
///
/// The proxy stores API keys per domain and can rotate them. It does NOT make
/// network calls — it only manages key state. The "session" is a string token
/// that represents an authenticated external session.
#[derive(Debug, Clone)]
pub struct EnsignProxy {
    keys: HashMap<String, String>,
    sessions: HashMap<String, String>, // domain -> session token
}

impl EnsignProxy {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    /// Store an API key for a domain.
    pub fn set_key(&mut self, domain: &str, key: &str) {
        self.keys.insert(domain.to_string(), key.to_string());
    }

    /// Retrieve the API key for a domain.
    pub fn get_key(&self, domain: &str) -> Option<&str> {
        self.keys.get(domain).map(|s| s.as_str())
    }

    /// Rotate the API key: replaces the old key with a new one.
    /// Returns the old key if one existed.
    pub fn rotate_key(&mut self, domain: &str, new_key: &str) -> Option<String> {
        self.keys.insert(domain.to_string(), new_key.to_string())
    }

    /// Remove an API key for a domain.
    pub fn remove_key(&mut self, domain: &str) -> Option<String> {
        self.keys.remove(domain)
    }

    /// Bind a session token to a domain.
    pub fn bind_session(&mut self, domain: &str, token: &str) {
        self.sessions.insert(domain.to_string(), token.to_string());
    }

    /// Get the session token for a domain.
    pub fn get_session(&self, domain: &str) -> Option<&str> {
        self.sessions.get(domain).map(|s| s.as_str())
    }

    /// Check if a domain has both a key and an active session.
    pub fn is_authenticated(&self, domain: &str) -> bool {
        self.keys.contains_key(domain) && self.sessions.contains_key(domain)
    }

    /// Number of domains with keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
}

impl Default for EnsignProxy {
    fn default() -> Self {
        Self::new()
    }
}

// ── Ensign Bridge ──────────────────────────────────────────────────────────

/// Connects ensigns to a skills system.
///
/// A skill is a named capability (like a construct-core skill) that can be
/// invoked. The bridge maps domain names to skill names and routes invocations.
#[derive(Debug, Clone)]
pub struct EnsignBridge {
    mappings: HashMap<String, String>, // domain -> skill_name
    invoked: Vec<(String, String)>,     // log of (domain, task) invocations
}

impl EnsignBridge {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            invoked: Vec::new(),
        }
    }

    /// Map a domain to a skill name.
    pub fn map(&mut self, domain: &str, skill: &str) {
        self.mappings.insert(domain.to_string(), skill.to_string());
    }

    /// Remove a mapping.
    pub fn unmap(&mut self, domain: &str) -> bool {
        self.mappings.remove(domain).is_some()
    }

    /// Get the skill name for a domain.
    pub fn skill_for(&self, domain: &str) -> Option<&str> {
        self.mappings.get(domain).map(|s| s.as_str())
    }

    /// Invoke a skill for a domain. Records the invocation in the log.
    /// Returns the skill name if mapped, or None.
    pub fn invoke(&mut self, domain: &str, task: &str) -> Option<&str> {
        if self.mappings.contains_key(domain) {
            self.invoked.push((domain.to_string(), task.to_string()));
            self.mappings.get(domain).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Get the invocation log.
    pub fn log(&self) -> &[(String, String)] {
        &self.invoked
    }

    /// Number of mapped domains.
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    /// Number of invocations.
    pub fn invocation_count(&self) -> usize {
        self.invoked.len()
    }
}

impl Default for EnsignBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_ensign() {
        let e = EchoEnsign::new("navigation");
        assert_eq!(e.domain(), "navigation");
        let result = e.handle("plot course");
        assert!(result.success);
        assert_eq!(result.output, "[navigation] plot course");
    }

    #[test]
    fn null_ensign() {
        let e = NullEnsign::new("placeholder");
        let result = e.handle("anything");
        assert!(!result.success);
        assert_eq!(result.output, "not implemented");
    }

    #[test]
    fn ensign_result_constructors() {
        let ok = EnsignResult::ok("done");
        assert!(ok.success);
        let err = EnsignResult::err("fail");
        assert!(!err.success);
    }

    #[test]
    fn registry_load_unload() {
        let mut reg = EnsignRegistry::new();
        reg.load(Box::new(EchoEnsign::new("nav")));
        assert_eq!(reg.len(), 1);
        assert!(reg.unload("nav"));
        assert!(reg.is_empty());
        assert!(!reg.unload("nav")); // already gone
    }

    #[test]
    fn registry_dispatch_success() {
        let mut reg = EnsignRegistry::new();
        reg.load(Box::new(EchoEnsign::new("comms")));
        let result = reg.dispatch("comms", "send message");
        assert!(result.success);
        assert_eq!(result.output, "[comms] send message");
    }

    #[test]
    fn registry_dispatch_missing_domain() {
        let reg = EnsignRegistry::new();
        let result = reg.dispatch("missing", "task");
        assert!(!result.success);
    }

    #[test]
    fn registry_domains() {
        let mut reg = EnsignRegistry::new();
        reg.load(Box::new(EchoEnsign::new("a")));
        reg.load(Box::new(EchoEnsign::new("b")));
        let mut domains = reg.domains();
        domains.sort();
        assert_eq!(domains, vec!["a", "b"]);
    }

    #[test]
    fn registry_get() {
        let mut reg = EnsignRegistry::new();
        reg.load(Box::new(EchoEnsign::new("x")));
        assert!(reg.get("x").is_some());
        assert!(reg.get("y").is_none());
    }

    #[test]
    fn factory_create() {
        let mut factory = EnsignFactory::new();
        factory.register("nav", || Box::new(EchoEnsign::new("nav")));
        let ensign = factory.create("nav").unwrap();
        assert_eq!(ensign.domain(), "nav");
        assert!(factory.create("missing").is_none());
    }

    #[test]
    fn factory_available() {
        let mut factory = EnsignFactory::new();
        factory.register("a", || Box::new(EchoEnsign::new("a")));
        factory.register("b", || Box::new(EchoEnsign::new("b")));
        let mut avail = factory.available();
        avail.sort();
        assert_eq!(avail, vec!["a", "b"]);
    }

    #[test]
    fn proxy_key_management() {
        let mut proxy = EnsignProxy::new();
        proxy.set_key("api", "key123");
        assert_eq!(proxy.get_key("api"), Some("key123"));
        let old = proxy.rotate_key("api", "key456");
        assert_eq!(old, Some("key123".to_string()));
        assert_eq!(proxy.get_key("api"), Some("key456"));
        proxy.remove_key("api");
        assert_eq!(proxy.get_key("api"), None);
    }

    #[test]
    fn proxy_session_and_auth() {
        let mut proxy = EnsignProxy::new();
        assert!(!proxy.is_authenticated("api"));
        proxy.set_key("api", "k");
        assert!(!proxy.is_authenticated("api"));
        proxy.bind_session("api", "tok");
        assert!(proxy.is_authenticated("api"));
        assert_eq!(proxy.get_session("api"), Some("tok"));
    }

    #[test]
    fn proxy_key_count() {
        let mut proxy = EnsignProxy::new();
        proxy.set_key("a", "1");
        proxy.set_key("b", "2");
        assert_eq!(proxy.key_count(), 2);
    }

    #[test]
    fn bridge_map_invoke() {
        let mut bridge = EnsignBridge::new();
        bridge.map("nav", "navigate_skill");
        assert_eq!(bridge.skill_for("nav"), Some("navigate_skill"));
        let result = bridge.invoke("nav", "go north");
        assert_eq!(result, Some("navigate_skill"));
        assert_eq!(bridge.invocation_count(), 1);
    }

    #[test]
    fn bridge_invoke_unmapped() {
        let mut bridge = EnsignBridge::new();
        assert!(bridge.invoke("missing", "task").is_none());
        assert_eq!(bridge.invocation_count(), 0);
    }

    #[test]
    fn bridge_log() {
        let mut bridge = EnsignBridge::new();
        bridge.map("x", "skill_x");
        bridge.invoke("x", "task1");
        bridge.invoke("x", "task2");
        assert_eq!(bridge.log().len(), 2);
        assert_eq!(bridge.log()[0].0, "x");
        assert_eq!(bridge.log()[1].1, "task2");
    }

    #[test]
    fn bridge_unmap() {
        let mut bridge = EnsignBridge::new();
        bridge.map("a", "s");
        assert!(bridge.unmap("a"));
        assert!(!bridge.unmap("a"));
        assert_eq!(bridge.mapping_count(), 0);
    }

    #[test]
    fn registry_replace() {
        let mut reg = EnsignRegistry::new();
        reg.load(Box::new(EchoEnsign::new("d")));
        let old = reg.load(Box::new(NullEnsign::new("d")));
        assert!(old.is_some());
        let result = reg.dispatch("d", "test");
        assert!(!result.success); // NullEnsign now
    }

    #[test]
    fn factory_default() {
        let f = EnsignFactory::default();
        assert!(f.available().is_empty());
    }

    #[test]
    fn proxy_default() {
        let p = EnsignProxy::default();
        assert_eq!(p.key_count(), 0);
    }

    #[test]
    fn bridge_default() {
        let b = EnsignBridge::default();
        assert_eq!(b.mapping_count(), 0);
    }
}
