//! Veilid private routes for anonymous communication

use crate::{PersonId, Result};
use serde::{Deserialize, Serialize};

/// A Veilid private route (anonymous multi-hop path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateRoute {
    /// Route ID (blob that identifies this route)
    pub id: RouteId,

    /// Route blob (opaque data for Veilid)
    /// This contains the encrypted route information
    route_blob: Vec<u8>,

    /// When was this route created?
    pub created_at: crate::CoarseTimestamp,

    /// Is this route still valid?
    pub valid: bool,
}

/// Route identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouteId(pub String);

impl PrivateRoute {
    /// Create a new private route from Veilid route data
    pub fn new(id: RouteId, route_blob: Vec<u8>) -> Self {
        Self {
            id,
            route_blob,
            created_at: crate::CoarseTimestamp::now(),
            valid: true,
        }
    }

    /// Get route blob (for sending to Veilid)
    pub fn route_blob(&self) -> &[u8] {
        &self.route_blob
    }

    /// Invalidate this route (e.g., compromised)
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Check if route is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Export route for sharing (this is what goes in QR code)
    pub fn export(&self) -> Result<String> {
        // Encode route blob as base64
        Ok(base64::encode(&self.route_blob))
    }

    /// Import route from exported data
    pub fn import(id: RouteId, encoded: &str) -> Result<Self> {
        let route_blob = base64::decode(encoded)
            .map_err(|e| crate::Error::Serialization(format!("Base64 decode failed: {}", e)))?;

        Ok(Self::new(id, route_blob))
    }
}

/// Route manager - manages private routes for personas
pub struct RouteManager {
    /// Routes by persona
    routes: std::collections::HashMap<PersonId, PrivateRoute>,
}

impl RouteManager {
    /// Create a new route manager
    pub fn new() -> Self {
        Self {
            routes: std::collections::HashMap::new(),
        }
    }

    /// Add a route for a persona
    pub fn add_route(&mut self, persona: PersonId, route: PrivateRoute) {
        self.routes.insert(persona, route);
    }

    /// Get route for a persona
    pub fn get_route(&self, persona: PersonId) -> Option<&PrivateRoute> {
        self.routes.get(&persona)
    }

    /// Remove route for a persona
    pub fn remove_route(&mut self, persona: PersonId) -> Option<PrivateRoute> {
        self.routes.remove(&persona)
    }

    /// List all routes
    pub fn list_routes(&self) -> Vec<(PersonId, &PrivateRoute)> {
        self.routes.iter().map(|(p, r)| (*p, r)).collect()
    }

    /// Count routes
    pub fn count(&self) -> usize {
        self.routes.len()
    }

    /// Invalidate all routes (emergency rotation)
    pub fn invalidate_all(&mut self) {
        for route in self.routes.values_mut() {
            route.invalidate();
        }
    }
}

impl Default for RouteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_route() {
        let id = RouteId("test-route".to_string());
        let blob = vec![1, 2, 3, 4, 5];

        let route = PrivateRoute::new(id.clone(), blob.clone());

        assert_eq!(route.id, id);
        assert_eq!(route.route_blob(), &blob);
        assert!(route.is_valid());
    }

    #[test]
    fn test_route_invalidation() {
        let id = RouteId("test".to_string());
        let mut route = PrivateRoute::new(id, vec![1, 2, 3]);

        assert!(route.is_valid());

        route.invalidate();
        assert!(!route.is_valid());
    }

    #[test]
    fn test_route_export_import() {
        let id = RouteId("test".to_string());
        let blob = vec![1, 2, 3, 4, 5];
        let route = PrivateRoute::new(id.clone(), blob);

        let exported = route.export().unwrap();
        assert!(!exported.is_empty());

        let imported = PrivateRoute::import(id.clone(), &exported).unwrap();
        assert_eq!(imported.route_blob(), route.route_blob());
    }

    #[test]
    fn test_route_manager() {
        let mut manager = RouteManager::new();

        let persona = PersonId::new();
        let route = PrivateRoute::new(RouteId("test".to_string()), vec![1, 2, 3]);

        assert_eq!(manager.count(), 0);

        manager.add_route(persona, route.clone());
        assert_eq!(manager.count(), 1);

        let retrieved = manager.get_route(persona);
        assert!(retrieved.is_some());

        manager.remove_route(persona);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_invalidate_all() {
        let mut manager = RouteManager::new();

        let p1 = PersonId::new();
        let p2 = PersonId::new();

        manager.add_route(p1, PrivateRoute::new(RouteId("r1".to_string()), vec![1]));
        manager.add_route(p2, PrivateRoute::new(RouteId("r2".to_string()), vec![2]));

        // All routes valid
        assert!(manager.get_route(p1).unwrap().is_valid());
        assert!(manager.get_route(p2).unwrap().is_valid());

        // Invalidate all
        manager.invalidate_all();

        // All routes now invalid
        assert!(!manager.get_route(p1).unwrap().is_valid());
        assert!(!manager.get_route(p2).unwrap().is_valid());
    }
}
