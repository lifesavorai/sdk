//! Component resolver for dependency resolution.
//!
//! When a system component depends on other components (e.g., a TTS component
//! that needs a model component), the resolver provides a way to look up
//! available instances by type and name.

use crate::{SystemComponentType, SystemComponentInfo};
use std::collections::HashMap;

/// A generic alias resolver that maps alias names to concrete values.
///
/// Used by TTS/STT components to resolve voice aliases (e.g., "default" → "en-US-Neural2-F")
/// and by other components for similar name-to-value mappings.
#[derive(Debug, Clone)]
pub struct AliasResolver<T: Clone> {
    aliases: HashMap<String, T>,
}

impl<T: Clone + ToString> AliasResolver<T> {
    /// Create a new alias resolver from a map of alias → value.
    pub fn new(aliases: HashMap<String, T>) -> Self {
        Self { aliases }
    }

    /// Resolve an alias to its value, or return the input as-is if no alias matches.
    pub fn resolve_or_passthrough(&self, key: &str) -> String {
        match self.aliases.get(key) {
            Some(value) => value.to_string(),
            None => key.to_string(),
        }
    }

    /// Resolve an alias to its value, or return None if no alias matches.
    pub fn resolve(&self, key: &str) -> Option<&T> {
        self.aliases.get(key)
    }

    /// Check if an alias exists.
    pub fn has_alias(&self, key: &str) -> bool {
        self.aliases.contains_key(key)
    }

    /// List all registered alias names.
    pub fn aliases(&self) -> Vec<&str> {
        self.aliases.keys().map(|s| s.as_str()).collect()
    }
}

/// A resolved component reference with connection details.
#[derive(Debug, Clone)]
pub struct ResolvedComponent {
    /// The component's unique instance ID.
    pub instance_id: String,
    /// The component type.
    pub component_type: SystemComponentType,
    /// The component's display name.
    pub name: String,
    /// Operations exposed by this component.
    pub operations: Vec<String>,
    /// Whether the component is currently healthy.
    pub healthy: bool,
}

/// Resolver for discovering and connecting to other system components.
#[derive(Debug, Clone, Default)]
pub struct ComponentResolver {
    registry: HashMap<String, ResolvedComponent>,
}

impl ComponentResolver {
    /// Create a new empty resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a component in the resolver.
    pub fn register(&mut self, instance_id: &str, component: ResolvedComponent) {
        self.registry.insert(instance_id.to_string(), component);
    }

    /// Look up a component by instance ID.
    pub fn get(&self, instance_id: &str) -> Option<&ResolvedComponent> {
        self.registry.get(instance_id)
    }

    /// Find all components of a given type.
    pub fn find_by_type(&self, component_type: SystemComponentType) -> Vec<&ResolvedComponent> {
        self.registry
            .values()
            .filter(|c| c.component_type == component_type)
            .collect()
    }

    /// Find a component that exposes a specific operation.
    pub fn find_by_operation(&self, operation: &str) -> Vec<&ResolvedComponent> {
        self.registry
            .values()
            .filter(|c| c.operations.iter().any(|op| op == operation))
            .collect()
    }

    /// List all registered component instance IDs.
    pub fn list_instances(&self) -> Vec<&str> {
        self.registry.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a component from the resolver.
    pub fn unregister(&mut self, instance_id: &str) -> Option<ResolvedComponent> {
        self.registry.remove(instance_id)
    }
}
