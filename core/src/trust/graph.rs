//! Trust graph - managing the web of trust relationships

use crate::{PersonId, TrustLevel};
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

/// A graph representing trust relationships between people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGraph {
    /// Adjacency list: person -> set of trusted people
    edges: HashMap<PersonId, HashMap<PersonId, TrustLevel>>,

    /// Cache of trust paths (for performance)
    #[serde(skip)]
    path_cache: HashMap<(PersonId, PersonId), Option<TrustPath>>,
}

/// A path of trust from one person to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPath {
    /// List of people in the path (including start and end)
    pub nodes: Vec<PersonId>,

    /// Trust levels along the path
    pub trust_levels: Vec<TrustLevel>,

    /// Overall path strength (minimum trust level in path)
    pub strength: TrustLevel,

    /// Number of hops
    pub hops: usize,
}

impl TrustGraph {
    /// Create a new empty trust graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            path_cache: HashMap::new(),
        }
    }

    /// Add a trust relationship
    pub fn add_trust(&mut self, from: PersonId, to: PersonId, level: TrustLevel) {
        self.edges
            .entry(from)
            .or_insert_with(HashMap::new)
            .insert(to, level);

        // Invalidate cache when graph changes
        self.path_cache.clear();
    }

    /// Remove a trust relationship
    pub fn remove_trust(&mut self, from: PersonId, to: PersonId) {
        if let Some(trusted) = self.edges.get_mut(&from) {
            trusted.remove(&to);
        }

        // Invalidate cache
        self.path_cache.clear();
    }

    /// Get trust level from one person to another (direct)
    pub fn get_trust(&self, from: PersonId, to: PersonId) -> Option<TrustLevel> {
        self.edges.get(&from)?.get(&to).copied()
    }

    /// Get all people trusted by someone
    pub fn get_trusted(&self, person: PersonId) -> Vec<(PersonId, TrustLevel)> {
        self.edges
            .get(&person)
            .map(|trusted| trusted.iter().map(|(p, t)| (*p, *t)).collect())
            .unwrap_or_default()
    }

    /// Get all people who trust someone
    pub fn get_trusters(&self, person: PersonId) -> Vec<(PersonId, TrustLevel)> {
        let mut trusters = Vec::new();

        for (truster, trusted) in &self.edges {
            if let Some(level) = trusted.get(&person) {
                trusters.push((*truster, *level));
            }
        }

        trusters
    }

    /// Find shortest trust path between two people (BFS)
    pub fn find_path(&mut self, from: PersonId, to: PersonId, max_hops: usize) -> Option<TrustPath> {
        // Check cache first
        if let Some(cached) = self.path_cache.get(&(from, to)) {
            return cached.clone();
        }

        // If direct trust exists, return immediate path
        if let Some(level) = self.get_trust(from, to) {
            let path = TrustPath {
                nodes: vec![from, to],
                trust_levels: vec![level],
                strength: level,
                hops: 1,
            };
            self.path_cache.insert((from, to), Some(path.clone()));
            return Some(path);
        }

        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<PersonId, (PersonId, TrustLevel)> = HashMap::new();

        queue.push_back((from, 0));
        visited.insert(from);

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            if current == to {
                // Reconstruct path
                let path = self.reconstruct_path(from, to, &parent);
                self.path_cache.insert((from, to), Some(path.clone()));
                return Some(path);
            }

            // Explore neighbors
            if let Some(trusted) = self.edges.get(&current) {
                for (next, trust_level) in trusted {
                    if !visited.contains(next) {
                        visited.insert(*next);
                        parent.insert(*next, (current, *trust_level));
                        queue.push_back((*next, depth + 1));
                    }
                }
            }
        }

        // No path found
        self.path_cache.insert((from, to), None);
        None
    }

    /// Reconstruct path from parent map
    fn reconstruct_path(
        &self,
        from: PersonId,
        to: PersonId,
        parent: &HashMap<PersonId, (PersonId, TrustLevel)>,
    ) -> TrustPath {
        let mut nodes = vec![to];
        let mut trust_levels = Vec::new();
        let mut current = to;

        while current != from {
            if let Some((prev, level)) = parent.get(&current) {
                nodes.push(*prev);
                trust_levels.push(*level);
                current = *prev;
            } else {
                break;
            }
        }

        nodes.reverse();
        trust_levels.reverse();

        let strength = trust_levels.iter().min().copied().unwrap_or(TrustLevel::Unknown);
        let hops = nodes.len() - 1;

        TrustPath {
            nodes,
            trust_levels,
            strength,
            hops,
        }
    }

    /// Get all people within N hops
    pub fn get_network(&self, person: PersonId, max_hops: usize) -> Vec<(PersonId, usize)> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((person, 0));
        visited.insert(person);

        while let Some((current, depth)) = queue.pop_front() {
            if depth > 0 {
                result.push((current, depth));
            }

            if depth >= max_hops {
                continue;
            }

            if let Some(trusted) = self.edges.get(&current) {
                for next in trusted.keys() {
                    if !visited.contains(next) {
                        visited.insert(*next);
                        queue.push_back((*next, depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Count total number of people in graph
    pub fn total_people(&self) -> usize {
        let mut people = HashSet::new();

        for (person, trusted) in &self.edges {
            people.insert(person);
            for trustee in trusted.keys() {
                people.insert(trustee);
            }
        }

        people.len()
    }

    /// Count total number of trust relationships
    pub fn total_relationships(&self) -> usize {
        self.edges.values().map(|trusted| trusted.len()).sum()
    }

    /// Get statistics about the graph
    pub fn stats(&self) -> TrustGraphStats {
        let total_people = self.total_people();
        let total_relationships = self.total_relationships();

        // Calculate average connections per person
        let avg_connections = if total_people > 0 {
            total_relationships as f64 / total_people as f64
        } else {
            0.0
        };

        // Count by trust level
        let mut by_level = HashMap::new();
        for trusted in self.edges.values() {
            for level in trusted.values() {
                *by_level.entry(*level).or_insert(0) += 1;
            }
        }

        TrustGraphStats {
            total_people,
            total_relationships,
            avg_connections,
            by_trust_level: by_level,
        }
    }

    /// Clear the path cache (call after bulk updates)
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
    }
}

/// Statistics about the trust graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGraphStats {
    /// Total number of people
    pub total_people: usize,

    /// Total number of trust relationships
    pub total_relationships: usize,

    /// Average connections per person
    pub avg_connections: f64,

    /// Count of relationships by trust level
    pub by_trust_level: HashMap<TrustLevel, usize>,
}

impl Default for TrustGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustPath {
    /// Check if this path is strong enough (all nodes meet minimum level)
    pub fn is_strong_enough(&self, minimum: TrustLevel) -> bool {
        self.strength >= minimum
    }

    /// Get the weakest link in the path
    pub fn weakest_link(&self) -> (PersonId, PersonId, TrustLevel) {
        let min_idx = self.trust_levels
            .iter()
            .enumerate()
            .min_by_key(|(_, level)| *level)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        (
            self.nodes[min_idx],
            self.nodes[min_idx + 1],
            self.trust_levels[min_idx],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_graph_basic() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();

        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);

        assert_eq!(
            graph.get_trust(alice, bob),
            Some(TrustLevel::VerifiedInPerson)
        );
        assert_eq!(graph.get_trust(bob, alice), None);
    }

    #[test]
    fn test_trust_path_finding() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();
        let charlie = PersonId::new();

        // Alice trusts Bob, Bob trusts Charlie
        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);
        graph.add_trust(bob, charlie, TrustLevel::VerifiedRemote);

        // Find path from Alice to Charlie
        let path = graph.find_path(alice, charlie, 5);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.nodes[0], alice);
        assert_eq!(path.nodes[1], bob);
        assert_eq!(path.nodes[2], charlie);
        assert_eq!(path.hops, 2);
        assert_eq!(path.strength, TrustLevel::VerifiedRemote); // Weakest link
    }

    #[test]
    fn test_trust_network() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();
        let charlie = PersonId::new();
        let david = PersonId::new();

        // Create network: Alice -> Bob -> Charlie -> David
        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);
        graph.add_trust(bob, charlie, TrustLevel::VerifiedRemote);
        graph.add_trust(charlie, david, TrustLevel::Introduced);

        let network = graph.get_network(alice, 2);
        assert_eq!(network.len(), 2); // Bob (1 hop) and Charlie (2 hops)

        let network_3 = graph.get_network(alice, 3);
        assert_eq!(network_3.len(), 3); // Bob, Charlie, and David
    }

    #[test]
    fn test_trust_graph_stats() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();
        let charlie = PersonId::new();

        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);
        graph.add_trust(alice, charlie, TrustLevel::VerifiedRemote);
        graph.add_trust(bob, charlie, TrustLevel::Introduced);

        let stats = graph.stats();
        assert_eq!(stats.total_people, 3);
        assert_eq!(stats.total_relationships, 3);
    }

    #[test]
    fn test_trust_removal() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();

        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);
        assert!(graph.get_trust(alice, bob).is_some());

        graph.remove_trust(alice, bob);
        assert!(graph.get_trust(alice, bob).is_none());
    }

    #[test]
    fn test_path_strength() {
        let mut graph = TrustGraph::new();

        let alice = PersonId::new();
        let bob = PersonId::new();
        let charlie = PersonId::new();

        graph.add_trust(alice, bob, TrustLevel::VerifiedInPerson);
        graph.add_trust(bob, charlie, TrustLevel::Unknown);

        let path = graph.find_path(alice, charlie, 5).unwrap();

        // Path strength is the weakest link
        assert_eq!(path.strength, TrustLevel::Unknown);
        assert!(!path.is_strong_enough(TrustLevel::Introduced));
    }
}
