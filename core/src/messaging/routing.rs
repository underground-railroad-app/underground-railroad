//! Message routing through the trust network

use crate::{PersonId, Result};
use serde::{Deserialize, Serialize};

/// A route through the network (for relayed messages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Ordered list of nodes in the route
    pub nodes: Vec<PersonId>,

    /// Current position in route (which node we're at)
    pub current_index: usize,
}

impl Route {
    /// Create a new route
    pub fn new(nodes: Vec<PersonId>) -> Self {
        Self {
            nodes,
            current_index: 0,
        }
    }

    /// Create a direct route (no relays)
    pub fn direct(recipient: PersonId) -> Self {
        Self {
            nodes: vec![recipient],
            current_index: 0,
        }
    }

    /// Get the next node in the route
    pub fn next_hop(&self) -> Option<PersonId> {
        self.nodes.get(self.current_index).copied()
    }

    /// Get the final destination
    pub fn destination(&self) -> Option<PersonId> {
        self.nodes.last().copied()
    }

    /// Advance to next hop
    pub fn advance(&mut self) -> Result<()> {
        if self.current_index + 1 < self.nodes.len() {
            self.current_index += 1;
            Ok(())
        } else {
            Err(crate::Error::Invalid("Route exhausted".to_string()))
        }
    }

    /// Check if we've reached the destination
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.nodes.len() - 1
    }

    /// Get total number of hops
    pub fn hop_count(&self) -> usize {
        self.nodes.len() - 1
    }

    /// Check if route is direct (no relays)
    pub fn is_direct(&self) -> bool {
        self.nodes.len() == 1
    }
}

/// Message router - finds routes through trust network
pub struct MessageRouter {
    // Will integrate with TrustGraph to find relay paths
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {}
    }

    /// Find a route to a recipient
    ///
    /// For now, this returns a direct route.
    /// TODO: Integrate with TrustGraph to find relay paths when direct route unavailable.
    pub fn find_route(&self, _sender: PersonId, recipient: PersonId) -> Result<Route> {
        // Simple direct route for now
        Ok(Route::direct(recipient))
    }

    /// Find a relay route through intermediaries
    ///
    /// TODO: Use TrustGraph.find_path() to find trusted relay path
    pub fn find_relay_route(
        &self,
        _sender: PersonId,
        recipient: PersonId,
        _max_hops: usize,
    ) -> Result<Route> {
        // For now, return direct route
        // In future: find path through trust graph
        Ok(Route::direct(recipient))
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_route() {
        let recipient = PersonId::new();
        let route = Route::direct(recipient);

        assert!(route.is_direct());
        assert_eq!(route.hop_count(), 0);
        assert_eq!(route.next_hop(), Some(recipient));
        assert_eq!(route.destination(), Some(recipient));
    }

    #[test]
    fn test_multi_hop_route() {
        let node1 = PersonId::new();
        let node2 = PersonId::new();
        let node3 = PersonId::new();

        let mut route = Route::new(vec![node1, node2, node3]);

        assert!(!route.is_direct());
        assert_eq!(route.hop_count(), 2);

        assert_eq!(route.next_hop(), Some(node1));
        route.advance().unwrap();

        assert_eq!(route.next_hop(), Some(node2));
        route.advance().unwrap();

        assert_eq!(route.next_hop(), Some(node3));
        assert!(route.is_complete());

        assert_eq!(route.destination(), Some(node3));
    }

    #[test]
    fn test_route_exhaustion() {
        let node = PersonId::new();
        let mut route = Route::direct(node);

        // Already at destination
        let result = route.advance();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_router() {
        let router = MessageRouter::new();
        let sender = PersonId::new();
        let recipient = PersonId::new();

        let route = router.find_route(sender, recipient).unwrap();
        assert!(route.is_direct());
        assert_eq!(route.destination(), Some(recipient));
    }
}
