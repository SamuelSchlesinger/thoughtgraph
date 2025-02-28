//! Visualization module for ThoughtGraph
//! 
//! This module provides functionality to visualize the connections between thoughts
//! in a ThoughtGraph by generating formats suitable for rendering as a network graph.

use std::collections::HashSet;
use crate::{ThoughtGraph, ThoughtID};

/// GraphData structure representing the graph for visualization
#[derive(Debug, Clone)]
pub struct GraphData {
    /// Nodes in the graph, representing thoughts
    pub nodes: Vec<Node>,
    /// Edges in the graph, representing references between thoughts
    pub edges: Vec<Edge>,
}

/// A node in the graph visualization, representing a single thought
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for the node
    pub id: String,
    /// Display label for the node (typically the thought's title)
    pub label: String,
    /// Tags associated with this node
    pub tags: Vec<String>,
}

/// An edge in the graph visualization, representing a reference between thoughts
#[derive(Debug, Clone)]
pub struct Edge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID (the thought containing the reference)
    pub source: String,
    /// Target node ID (the thought being referenced)
    pub target: String,
    /// Label/description of the reference
    pub label: String,
}

impl GraphData {
    /// Generate DOT format representation of the graph suitable for Graphviz
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph ThoughtGraph {\n");
        dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");
        
        // Add nodes
        for node in &self.nodes {
            let label = node.label.replace("\"", "\\\"");
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", node.id, label));
        }
        
        dot.push_str("\n");
        
        // Add edges
        for edge in &self.edges {
            let label = edge.label.replace("\"", "\\\"");
            dot.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", 
                edge.source, edge.target, label));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Generate JSON representation of the graph suitable for D3.js or other web visualizations
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str("  \"nodes\": [\n");
        
        // Add nodes
        for (i, node) in self.nodes.iter().enumerate() {
            let comma = if i < self.nodes.len() - 1 { "," } else { "" };
            json.push_str(&format!("    {{\"id\": \"{}\", \"label\": \"{}\", \"tags\": {:?}}}{}\n", 
                node.id, node.label, node.tags, comma));
        }
        
        json.push_str("  ],\n");
        json.push_str("  \"edges\": [\n");
        
        // Add edges
        for (i, edge) in self.edges.iter().enumerate() {
            let comma = if i < self.edges.len() - 1 { "," } else { "" };
            json.push_str(&format!("    {{\"id\": \"{}\", \"source\": \"{}\", \"target\": \"{}\", \"label\": \"{}\"}}{}\n", 
                edge.id, edge.source, edge.target, edge.label, comma));
        }
        
        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    }
}

/// Function to generate visualization data from a ThoughtGraph
pub fn generate_graph_data(graph: &ThoughtGraph) -> GraphData {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut edge_id = 0;
    
    // Process all thoughts in the graph
    for (thought_id, thought) in &graph.thoughts {
        // Add the thought as a node
        nodes.push(Node {
            id: thought_id.id.clone(),
            label: thought.title.clone().unwrap_or_else(|| thought_id.id.clone()),
            tags: thought.tags.iter().map(|tag_id| tag_id.id.clone()).collect(),
        });
        
        // Process all references as edges
        for reference in &thought.references {
            edge_id += 1;
            edges.push(Edge {
                id: format!("edge_{}", edge_id),
                source: thought_id.id.clone(),
                target: reference.id.id.clone(),
                label: reference.notes.clone(),
            });
        }
    }
    
    GraphData { nodes, edges }
}

/// Function to generate a subgraph centered around a specific thought
pub fn generate_focused_graph(
    graph: &ThoughtGraph, 
    center_id: &ThoughtID, 
    depth: usize
) -> GraphData {
    let mut visited = HashSet::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut edge_id = 0;
    
    // Start with the central thought
    if let Some(_center_thought) = graph.get_thought(center_id) {
        // BFS traversal of the graph up to specified depth
        let mut queue = vec![(center_id.clone(), 0)];
        visited.insert(center_id.clone());
        
        while let Some((current_id, current_depth)) = queue.pop() {
            if current_depth > depth {
                continue;
            }
            
            if let Some(thought) = graph.get_thought(&current_id) {
                // Add current thought as a node
                nodes.push(Node {
                    id: current_id.id.clone(),
                    label: thought.title.clone().unwrap_or_else(|| current_id.id.clone()),
                    tags: thought.tags.iter().map(|tag_id| tag_id.id.clone()).collect(),
                });
                
                // Process outgoing references
                for reference in &thought.references {
                    if !visited.contains(&reference.id) && current_depth < depth {
                        queue.push((reference.id.clone(), current_depth + 1));
                        visited.insert(reference.id.clone());
                    }
                    
                    // Add the edge regardless of whether we traverse to that node
                    edge_id += 1;
                    edges.push(Edge {
                        id: format!("edge_{}", edge_id),
                        source: current_id.id.clone(),
                        target: reference.id.id.clone(),
                        label: reference.notes.clone(),
                    });
                }
                
                // Process incoming references (backlinks)
                for backlink_id in graph.get_backlinks(&current_id) {
                    if !visited.contains(&backlink_id) && current_depth < depth {
                        queue.push((backlink_id.clone(), current_depth + 1));
                        visited.insert(backlink_id.clone());
                    }
                    
                    // Add the edge if the source is already in our visited set
                    if visited.contains(&backlink_id) {
                        // Find the reference from the backlink to the current thought
                        if let Some(backlink_thought) = graph.get_thought(&backlink_id) {
                            for reference in &backlink_thought.references {
                                if reference.id == current_id {
                                    edge_id += 1;
                                    edges.push(Edge {
                                        id: format!("edge_{}", edge_id),
                                        source: backlink_id.id.clone(),
                                        target: current_id.id.clone(),
                                        label: reference.notes.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    GraphData { nodes, edges }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Thought, Tag, TagID, Reference};
    use chrono::Utc;
    
    fn create_test_graph() -> ThoughtGraph {
        let mut graph = ThoughtGraph::new();
        
        // Create tags
        let tag1 = TagID::new("programming".to_string());
        let tag2 = TagID::new("concept".to_string());
        
        graph.command(&Command::PutTag {
            id: tag1.clone(),
            tag: Tag::new("Programming".to_string()),
        });
        
        graph.command(&Command::PutTag {
            id: tag2.clone(),
            tag: Tag::new("Concept".to_string()),
        });
        
        // Create thoughts with references
        let thought1_id = ThoughtID::new("rust".to_string());
        let thought2_id = ThoughtID::new("programming".to_string());
        let thought3_id = ThoughtID::new("memory-safety".to_string());
        
        // Thought 1: Rust
        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: Thought::new(
                Some("Rust Programming Language".to_string()),
                "Rust is a systems programming language focused on safety and performance.".to_string(),
                vec![tag1.clone()],
                vec![Reference::new(
                    thought2_id.clone(),
                    "Type of programming".to_string(),
                    Utc::now(),
                )],
            ),
        });
        
        // Thought 2: Programming
        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: Thought::new(
                Some("Programming".to_string()),
                "The process of creating software.".to_string(),
                vec![tag2.clone()],
                vec![],
            ),
        });
        
        // Thought 3: Memory Safety
        graph.command(&Command::PutThought {
            id: thought3_id.clone(),
            thought: Thought::new(
                Some("Memory Safety".to_string()),
                "A property that ensures memory accesses are always valid.".to_string(),
                vec![tag2.clone()],
                vec![Reference::new(
                    thought1_id.clone(),
                    "Rust enforces memory safety".to_string(),
                    Utc::now(),
                )],
            ),
        });
        
        graph
    }
    
    #[test]
    fn test_generate_graph_data() {
        let graph = create_test_graph();
        let graph_data = generate_graph_data(&graph);
        
        // Check nodes
        assert_eq!(graph_data.nodes.len(), 3);
        
        // Check edges
        assert_eq!(graph_data.edges.len(), 2);
        
        // Verify node content
        let rust_node = graph_data.nodes.iter().find(|n| n.id == "rust").unwrap();
        assert_eq!(rust_node.label, "Rust Programming Language");
        assert_eq!(rust_node.tags, vec!["programming"]);
        
        // Verify edge content
        let rust_to_programming_edge = graph_data.edges.iter()
            .find(|e| e.source == "rust" && e.target == "programming")
            .unwrap();
        assert_eq!(rust_to_programming_edge.label, "Type of programming");
    }
    
    #[test]
    fn test_focused_graph() {
        let graph = create_test_graph();
        let center_id = ThoughtID::new("rust".to_string());
        
        // Depth 1 should include rust and its immediate connections
        let focused_graph_d1 = generate_focused_graph(&graph, &center_id, 1);
        assert_eq!(focused_graph_d1.nodes.len(), 3); // rust, programming, memory-safety
        
        // Check that memory-safety -> rust edge exists
        let memory_to_rust_edge = focused_graph_d1.edges.iter()
            .find(|e| e.source == "memory-safety" && e.target == "rust")
            .unwrap();
        assert_eq!(memory_to_rust_edge.label, "Rust enforces memory safety");
    }
    
    #[test]
    fn test_dot_format() {
        let graph = create_test_graph();
        let graph_data = generate_graph_data(&graph);
        let dot = graph_data.to_dot();
        
        // Check that DOT format contains all nodes and edges
        assert!(dot.contains("\"rust\" [label=\"Rust Programming Language\"]"));
        assert!(dot.contains("\"programming\" [label=\"Programming\"]"));
        assert!(dot.contains("\"memory-safety\" [label=\"Memory Safety\"]"));
        assert!(dot.contains("\"rust\" -> \"programming\""));
        assert!(dot.contains("\"memory-safety\" -> \"rust\""));
    }
    
    #[test]
    fn test_json_format() {
        let graph = create_test_graph();
        let graph_data = generate_graph_data(&graph);
        let json = graph_data.to_json();
        
        // Check that JSON format contains all nodes and edges
        assert!(json.contains("\"id\": \"rust\""));
        assert!(json.contains("\"label\": \"Rust Programming Language\""));
        assert!(json.contains("\"source\": \"rust\", \"target\": \"programming\""));
        assert!(json.contains("\"source\": \"memory-safety\", \"target\": \"rust\""));
    }
}