//! Component resolver for dependency resolution.
//!
//! When a system component depends on other components (e.g., a TTS component
//! that needs a model component), the resolver provides a way to look up
//! available instances by type and name.

use crate::{SystemComponentType, SystemComponentInfo};
use std::collections::HashMap;

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
