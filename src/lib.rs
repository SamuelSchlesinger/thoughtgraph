//! # ThoughtGraph
//! 
//! A library for managing interconnected thoughts with bidirectional references and tags.
//! 
//! ThoughtGraph provides a flexible data structure for representing knowledge in a graph
//! where nodes are "thoughts" that can be interconnected through references. The library
//! efficiently tracks both forward and backward references, allowing for rich, bidirectional
//! navigation between related concepts.
//!
//! ## Key Features
//!
//! - **Bidirectional References**: Create and track connections between thoughts
//! - **Tagging System**: Organize thoughts with customizable tags
//! - **Flexible Queries**: Search for thoughts using complex boolean expressions
//! - **Command-based Modifications**: Modify the graph via a command interface
//! - **Persistence**: Store and retrieve graph data using Serde and Bincode serialization
//!
//! ## Example
//!
//! ```
//! use thoughtgraph::{ThoughtGraph, ThoughtID, TagID, Thought, Tag, Reference, Command, Query};
//! use chrono::Utc;
//!
//! // Create a new graph
//! let mut graph = ThoughtGraph::new();
//!
//! // Create and add a tag
//! let programming_tag = TagID::new("programming".to_string());
//! graph.command(&Command::PutTag {
//!     id: programming_tag.clone(),
//!     tag: Tag::new("Programming concepts".to_string()),
//! });
//!
//! // Create a thought about Rust
//! let rust_id = ThoughtID::new("rust".to_string());
//! let rust_thought = Thought::new(
//!     Some("Rust Programming".to_string()),
//!     "Rust is a systems programming language focused on safety and performance.".to_string(),
//!     vec![programming_tag.clone()],
//!     vec![],
//! );
//!
//! // Add the thought to the graph
//! graph.command(&Command::PutThought {
//!     id: rust_id.clone(),
//!     thought: rust_thought,
//! });
//!
//! // Create a related thought that references the first one
//! let cargo_id = ThoughtID::new("cargo".to_string());
//! let cargo_thought = Thought::new(
//!     Some("Cargo".to_string()),
//!     "Cargo is Rust's package manager and build system.".to_string(),
//!     vec![programming_tag.clone()],
//!     vec![Reference::new(
//!         rust_id.clone(),
//!         "Built for Rust".to_string(),
//!         Utc::now(),
//!     )],
//! );
//!
//! // Add the second thought
//! graph.command(&Command::PutThought {
//!     id: cargo_id.clone(),
//!     thought: cargo_thought,
//! });
//!
//! // Query for thoughts with the programming tag
//! let programming_thoughts = graph.query(&Query::Tag(programming_tag.clone()));
//! assert_eq!(programming_thoughts.len(), 2);
//!
//! // Query for thoughts that reference the Rust thought
//! let rust_references = graph.query(&Query::References(rust_id.clone()));
//! assert_eq!(rust_references.len(), 1);
//! assert!(rust_references.contains(&cargo_id));
//! ```
//!
//! ## Command Line Tool
//!
//! ThoughtGraph includes a command-line tool called `thoughts` that lets you interact with your
//! thought graph from the terminal. See the binary documentation for more details.
//!

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

pub mod visualization;
pub mod ui;

/// Error types for ThoughtGraph operations
#[derive(Error, Debug)]
pub enum ThoughtGraphError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    
    #[error("Thought not found: {0}")]
    ThoughtNotFound(String),
    
    #[error("Tag not found: {0}")]
    TagNotFound(String),
    
    #[error("Invalid thought ID: {0}")]
    InvalidThoughtID(String),
    
    #[error("External editor error: {0}")]
    EditorError(String),
}

/// Result type for ThoughtGraph operations
pub type Result<T> = std::result::Result<T, ThoughtGraphError>;

/// Unique identifier for a thought in the graph.
///
/// Each thought has a unique string identifier that is used to reference it within the graph.
/// This ID is used for creating references between thoughts and for querying the graph.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThoughtID {
    /// The unique string identifier
    pub id: String,
}

impl ThoughtID {
    /// Creates a new ThoughtID with the given string identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the thought
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::ThoughtID;
    ///
    /// let thought_id = ThoughtID::new("unique-thought-123".to_string());
    /// ```
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

/// A reference from one thought to another.
///
/// References create connections between thoughts, establishing a graph-like structure.
/// Each reference includes the target thought's ID, optional notes about the relationship,
/// and a timestamp for when the reference was created or last accessed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reference {
    /// ID of the thought being referenced
    pub id: ThoughtID,
    /// Notes about this reference that describe the relationship
    pub notes: String,
    /// When this reference was created or last accessed
    pub access_date: DateTime<Utc>,
}

impl Reference {
    /// Creates a new reference to a thought.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the thought being referenced
    /// * `notes` - Optional notes describing the relationship
    /// * `access_date` - When this reference was created or accessed
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::{Reference, ThoughtID};
    /// use chrono::Utc;
    ///
    /// let thought_id = ThoughtID::new("target-thought".to_string());
    /// let reference = Reference::new(
    ///     thought_id,
    ///     "This thought expands on the target".to_string(),
    ///     Utc::now()
    /// );
    /// ```
    pub fn new(id: ThoughtID, notes: String, access_date: DateTime<Utc>) -> Self {
        Self { id, notes, access_date }
    }
}

/// A thought in the graph, containing content and metadata.
///
/// Thoughts are the primary nodes in the ThoughtGraph system. Each thought can have
/// a title, content text, associated tags for categorization, and references to other 
/// thoughts, creating a web of interconnected knowledge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thought {
    /// Optional title for the thought
    pub title: Option<String>,
    /// Main content of the thought
    pub contents: String,
    /// Tags associated with this thought for categorization
    pub tags: Vec<TagID>,
    /// References to other thoughts, creating connections in the graph
    pub references: Vec<Reference>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl Thought {
    /// Creates a new thought with the given attributes.
    ///
    /// # Arguments
    ///
    /// * `title` - Optional title for the thought
    /// * `contents` - The main content text of the thought
    /// * `tags` - Vector of tags associated with this thought
    /// * `references` - Vector of references to other thoughts
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::{Thought, ThoughtID, TagID, Reference};
    /// use chrono::Utc;
    ///
    /// // Create a thought with a title, content, one tag, and one reference
    /// let thought = Thought::new(
    ///     Some("My Thought Title".to_string()),
    ///     "This is the content of my thought.".to_string(),
    ///     vec![TagID::new("example-tag".to_string())],
    ///     vec![Reference::new(
    ///         ThoughtID::new("related-thought".to_string()),
    ///         "Related to this idea".to_string(),
    ///         Utc::now()
    ///     )]
    /// );
    /// ```
    pub fn new(
        title: Option<String>,
        contents: String,
        tags: Vec<TagID>,
        references: Vec<Reference>,
    ) -> Self {
        let now = Utc::now();
        Self { 
            title, 
            contents, 
            tags, 
            references,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Updates the content of the thought and its modified timestamp
    pub fn update_content(&mut self, new_content: String) {
        self.contents = new_content;
        self.updated_at = Utc::now();
    }
    
    /// Updates the title of the thought and its modified timestamp
    pub fn update_title(&mut self, new_title: Option<String>) {
        self.title = new_title;
        self.updated_at = Utc::now();
    }
    
    /// Adds a tag to the thought if it doesn't already exist
    pub fn add_tag(&mut self, tag: TagID) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
    
    /// Removes a tag from the thought
    pub fn remove_tag(&mut self, tag: &TagID) {
        let len_before = self.tags.len();
        self.tags.retain(|t| t != tag);
        
        if self.tags.len() != len_before {
            self.updated_at = Utc::now();
        }
    }
    
    /// Adds a reference to another thought
    pub fn add_reference(&mut self, reference: Reference) {
        // Check if a reference to this thought already exists
        if !self.references.iter().any(|r| r.id == reference.id) {
            self.references.push(reference);
            self.updated_at = Utc::now();
        }
    }
    
    /// Removes references to a specific thought
    pub fn remove_references_to(&mut self, thought_id: &ThoughtID) {
        let len_before = self.references.len();
        self.references.retain(|r| r.id != *thought_id);
        
        if self.references.len() != len_before {
            self.updated_at = Utc::now();
        }
    }
    
    /// Extract thought references from content in the format [thought_id]
    /// 
    /// This method scans the thought's content for any text patterns matching the format
    /// `[thought_id]` and extracts the IDs as potential references to other thoughts.
    /// This supports the auto-reference feature which allows creating connections between
    /// thoughts by simply mentioning their IDs in square brackets.
    ///
    /// The regex pattern matches alphanumeric characters, underscores, and hyphens between
    /// square brackets. For example, `[my-thought-123]` would be extracted as a reference
    /// to the thought with ID "my-thought-123".
    ///
    /// # Returns
    ///
    /// A vector of ThoughtIDs that were found in the content
    ///
    /// # Example
    ///
    /// ```
    /// use thoughtgraph::{Thought, ThoughtID};
    ///
    /// let thought = Thought::new(
    ///     Some("Example".to_string()),
    ///     "This references [thought1] and also [another-thought-2]".to_string(),
    ///     vec![],
    ///     vec![],
    /// );
    ///
    /// let refs = thought.extract_references_from_content();
    /// assert_eq!(refs.len(), 2);
    /// assert_eq!(refs[0].id, "thought1");
    /// assert_eq!(refs[1].id, "another-thought-2");
    /// ```
    pub fn extract_references_from_content(&self) -> Vec<ThoughtID> {
        let mut found_refs = Vec::new();
        let re = regex::Regex::new(r"\[([a-zA-Z0-9_-]+)\]").unwrap();
        
        for cap in re.captures_iter(&self.contents) {
            if let Some(thought_id) = cap.get(1) {
                found_refs.push(ThoughtID::new(thought_id.as_str().to_string()));
            }
        }
        
        found_refs
    }
}

/// Unique identifier for a tag in the graph.
///
/// Tags are used to categorize and group thoughts. Each tag has a unique string identifier
/// that is used to reference it within the graph.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagID {
    /// The unique string identifier for the tag
    pub id: String,
}

impl TagID {
    /// Creates a new TagID with the given string identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - A string that uniquely identifies the tag
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::TagID;
    ///
    /// let tag_id = TagID::new("concept".to_string());
    /// ```
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

/// A tag that can be attached to thoughts for categorization.
///
/// Tags provide a way to categorize and group related thoughts. Each tag has
/// a description that explains what the tag represents and what kinds of thoughts
/// it should be applied to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    /// Description of what this tag represents
    pub description: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    /// Creates a new tag with the given description.
    ///
    /// # Arguments
    ///
    /// * `description` - A string describing what this tag represents
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::Tag;
    ///
    /// let programming_tag = Tag::new("Programming-related concepts and tools".to_string());
    /// ```
    pub fn new(description: String) -> Self {
        let now = Utc::now();
        Self { 
            description,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Updates the description of the tag and its modified timestamp
    pub fn update_description(&mut self, new_description: String) {
        self.description = new_description;
        self.updated_at = Utc::now();
    }
}

/// A graph of interconnected thoughts with references and tags.
///
/// The `ThoughtGraph` is the main data structure of this library, representing a network
/// of thoughts that can reference each other and be organized with tags. It maintains
/// both forward references (from thoughts to other thoughts) and back-references
/// (tracking which thoughts reference a given thought), enabling efficient bidirectional
/// navigation of the knowledge graph.
///
/// The graph can be modified through the `command` method and queried through the `query` method.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ThoughtGraph {
    /// Map of thought IDs to thoughts
    pub thoughts: HashMap<ThoughtID, Thought>,
    /// Map of thought IDs to thoughts that reference them (inverse index of references)
    pub backreferences: HashMap<ThoughtID, Vec<ThoughtID>>,
    /// Map of tag IDs to tags
    pub tags: HashMap<TagID, Tag>,
}

/// Query operations for retrieving thoughts from the graph.
///
/// The `Query` enum provides a flexible way to search for thoughts in the graph.
/// Queries can be combined using logical AND and OR operations to create complex
/// search criteria.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Query {
    /// Find thoughts with the given tag.
    ///
    /// Returns all thoughts that have the specified tag.
    Tag(TagID),
    
    /// Find thoughts that reference the given thought.
    ///
    /// Returns all thoughts that contain a reference to the specified thought.
    /// This effectively returns the "backlinks" to a thought.
    References(ThoughtID),
    
    /// Find thoughts that are referenced by the given thought.
    ///
    /// Returns all thoughts that are referenced by the specified thought.
    /// This returns the "forward links" from a thought.
    ReferencedBy(ThoughtID),
    
    /// Logical AND of multiple queries.
    ///
    /// Returns thoughts that match ALL of the subqueries.
    And(Vec<Box<Query>>),
    
    /// Logical OR of multiple queries.
    ///
    /// Returns thoughts that match ANY of the subqueries.
    Or(Vec<Box<Query>>),
}

/// Commands for modifying the graph.
///
/// The `Command` enum represents operations that can modify the graph structure.
/// All modifications to the graph should be done through these commands to ensure
/// that the graph's internal state (including backreferences) remains consistent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Add or update a thought.
    ///
    /// If a thought with the given ID already exists, it will be replaced.
    /// All references and backreferences will be updated accordingly.
    PutThought { id: ThoughtID, thought: Thought },
    
    /// Remove a thought from the graph.
    ///
    /// This will also update all backreferences to maintain consistency.
    /// References to this thought in other thoughts will remain but will
    /// be treated as references to a non-existent thought.
    DeleteThought { id: ThoughtID },
    
    /// Add or update a tag.
    ///
    /// If a tag with the given ID already exists, it will be replaced.
    PutTag { id: TagID, tag: Tag },
    
    /// Remove a tag from the graph.
    ///
    /// This only removes the tag definition. Thoughts that reference this tag
    /// will continue to do so, but the tag will be treated as non-existent
    /// for query purposes.
    DeleteTag { id: TagID },
}

impl ThoughtGraph {
    /// Creates a new, empty ThoughtGraph.
    ///
    /// Initializes a fresh graph with no thoughts, tags, or references.
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::ThoughtGraph;
    ///
    /// let graph = ThoughtGraph::new();
    /// // The graph is now ready to accept commands and queries
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a command to modify the graph.
    ///
    /// This method applies the given command to modify the graph's structure.
    /// It handles all the necessary updates to maintain consistency, particularly
    /// with backreferences when thoughts are added, updated, or removed.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to apply to the graph
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::{ThoughtGraph, ThoughtID, Thought, Command};
    ///
    /// let mut graph = ThoughtGraph::new();
    /// let thought_id = ThoughtID::new("my-thought".to_string());
    /// let thought = Thought::new(
    ///     Some("Title".to_string()),
    ///     "Content".to_string(),
    ///     vec![],
    ///     vec![]
    /// );
    ///
    /// // Add a thought to the graph
    /// graph.command(&Command::PutThought {
    ///     id: thought_id.clone(),
    ///     thought,
    /// });
    ///
    /// // Delete the thought
    /// graph.command(&Command::DeleteThought { id: thought_id });
    /// ```
    pub fn command(&mut self, command: &Command) {
        match command {
            Command::PutThought { id, thought } => {
                // First, update backreferences
                // Remove old backreferences if this thought already exists
                if let Some(old_thought) = self.thoughts.get(id) {
                    for reference in &old_thought.references {
                        if let Some(backrefs) = self.backreferences.get_mut(&reference.id) {
                            backrefs.retain(|ref_id| ref_id != id);
                            // Clean up empty backreference entries
                            if backrefs.is_empty() {
                                self.backreferences.remove(&reference.id);
                            }
                        }
                    }
                }
                
                // Add new backreferences
                for reference in &thought.references {
                    self.backreferences
                        .entry(reference.id.clone())
                        .or_default()
                        .push(id.clone());
                }
                
                // Now insert or update the thought
                self.thoughts.insert(id.clone(), thought.clone());
            },
            
            Command::DeleteThought { id } => {
                // First, remove backreferences created by this thought
                if let Some(thought) = self.thoughts.get(id) {
                    for reference in &thought.references {
                        if let Some(backrefs) = self.backreferences.get_mut(&reference.id) {
                            backrefs.retain(|ref_id| ref_id != id);
                            // Clean up empty backreference entries
                            if backrefs.is_empty() {
                                self.backreferences.remove(&reference.id);
                            }
                        }
                    }
                }
                
                // Remove the thought itself
                self.thoughts.remove(id);
                
                // Remove any backreferences to this thought
                self.backreferences.remove(id);
            },
            
            Command::PutTag { id, tag } => {
                // Simply insert or update the tag
                self.tags.insert(id.clone(), tag.clone());
            },
            
            Command::DeleteTag { id } => {
                // Just remove the tag - no need to modify thoughts
                // as they will simply reference a non-existent tag
                self.tags.remove(id);
            },
        }
    }

    /// Execute a query against the graph and return matching thought IDs.
    ///
    /// This method evaluates the given query against the current state of the graph
    /// and returns a set of ThoughtIDs for all thoughts that match the query criteria.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to execute
    ///
    /// # Returns
    ///
    /// A HashSet of ThoughtIDs that match the query criteria
    ///
    /// # Examples
    ///
    /// ```
    /// use thoughtgraph::{ThoughtGraph, ThoughtID, TagID, Thought, Tag, Reference, Command, Query};
    /// use chrono::Utc;
    /// use std::collections::HashSet;
    ///
    /// let mut graph = ThoughtGraph::new();
    ///
    /// // Set up some test data
    /// let tag_id = TagID::new("test-tag".to_string());
    /// graph.command(&Command::PutTag {
    ///     id: tag_id.clone(),
    ///     tag: Tag::new("Test tag".to_string()),
    /// });
    ///
    /// let thought_id = ThoughtID::new("test-thought".to_string());
    /// graph.command(&Command::PutThought {
    ///     id: thought_id.clone(),
    ///     thought: Thought::new(
    ///         Some("Test".to_string()),
    ///         "Content".to_string(),
    ///         vec![tag_id.clone()],
    ///         vec![]
    ///     ),
    /// });
    ///
    /// // Simple tag query
    /// let results = graph.query(&Query::Tag(tag_id.clone()));
    /// assert!(results.contains(&thought_id));
    ///
    /// // Complex AND query
    /// let complex_query = Query::And(vec![
    ///     Box::new(Query::Tag(tag_id.clone())),
    ///     Box::new(Query::ReferencedBy(ThoughtID::new("nonexistent".to_string())))
    /// ]);
    /// let complex_results = graph.query(&complex_query);
    /// assert_eq!(complex_results.len(), 0); // Should be empty since one condition doesn't match
    /// ```
    pub fn query(&self, query: &Query) -> HashSet<ThoughtID> {
        match query {
            Query::Tag(tag_id) => {
                // Find all thoughts that have this tag
                // Only return thoughts if the tag still exists in the tags map
                if !self.tags.contains_key(tag_id) {
                    return HashSet::new();
                }
                
                self.thoughts
                    .iter()
                    .filter(|(_, thought)| thought.tags.contains(tag_id))
                    .map(|(id, _)| id.clone())
                    .collect()
            },
            
            Query::References(thought_id) => {
                // Find all thoughts that reference the given thought
                // We don't check if the referenced thought exists here,
                // since we want to find all thoughts that reference a specific ID
                // even if that ID doesn't exist yet
                self.backreferences
                    .get(thought_id)
                    .map_or_else(
                        HashSet::new,
                        |backrefs| backrefs.iter().cloned().collect()
                    )
            },
            
            Query::ReferencedBy(thought_id) => {
                // Find all thoughts that are referenced by the given thought
                let result = match self.thoughts.get(thought_id) {
                    Some(thought) => {
                        // Get all the thought IDs that this thought references
                        thought.references
                            .iter()
                            .map(|r| r.id.clone())
                            .filter(|id| self.thoughts.contains_key(id)) // Only include thoughts that exist
                            .collect()
                    },
                    None => HashSet::new()
                };
                result
            },
            
            Query::And(subqueries) => {
                // Start with all thoughts if there are no subqueries
                if subqueries.is_empty() {
                    return HashSet::new();
                }
                
                // Take the intersection of all subquery results
                subqueries
                    .iter()
                    .map(|subquery| self.query(subquery))
                    .reduce(|accum, item| {
                        accum.intersection(&item).cloned().collect()
                    })
                    .unwrap_or_else(HashSet::new)
            },
            
            Query::Or(subqueries) => {
                // Take the union of all subquery results
                let mut result = HashSet::new();
                for subquery in subqueries {
                    result.extend(self.query(subquery));
                }
                result
            },
        }
    }
    
    /// Get a thought by its ID
    pub fn get_thought(&self, id: &ThoughtID) -> Option<&Thought> {
        self.thoughts.get(id)
    }
    
    /// Get a tag by its ID
    pub fn get_tag(&self, id: &TagID) -> Option<&Tag> {
        self.tags.get(id)
    }
    
    /// Get all thoughts that reference the given thought ID
    pub fn get_backlinks(&self, id: &ThoughtID) -> Vec<ThoughtID> {
        self.backreferences
            .get(id)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
    
    /// Save the graph to a file in binary format
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let encoded = bincode::serialize(self)?;
        fs::write(path, encoded)?;
        Ok(())
    }
    
    /// Load a graph from a binary file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;
        let graph = bincode::deserialize(&data)?;
        Ok(graph)
    }
    
    /// Create a new thought with the given parameters
    pub fn create_thought(
        &mut self, 
        id: ThoughtID, 
        title: Option<String>, 
        contents: String,
        tags: Vec<TagID>,
        references: Vec<Reference>,
    ) -> Result<&Thought> {
        let thought = Thought::new(title, contents, tags, references);
        self.command(&Command::PutThought {
            id: id.clone(),
            thought,
        });
        
        self.thoughts.get(&id).ok_or_else(|| ThoughtGraphError::ThoughtNotFound(id.id.clone()))
    }
    
    /// Process automatic references from content (in [thought_id] format)
    /// and add them to the thought's references.
    ///
    /// This function scans the content of a thought for patterns like `[thought_id]`
    /// and automatically creates references to those thoughts if they exist in the graph.
    /// It allows users to easily create connections between thoughts by simply mentioning
    /// their IDs in square brackets within the content.
    ///
    /// # Arguments
    ///
    /// * `thought_id` - The ID of the thought whose content should be processed for references
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of ThoughtIDs that were added as references
    ///
    /// # Example
    ///
    /// ```
    /// use thoughtgraph::{ThoughtGraph, ThoughtID, Thought};
    ///
    /// // Create a graph with two thoughts
    /// let mut graph = ThoughtGraph::new();
    /// let thought1_id = ThoughtID::new("thought1".to_string());
    /// let thought2_id = ThoughtID::new("thought2".to_string());
    ///
    /// // Add the first thought
    /// graph.create_thought(
    ///     thought1_id.clone(),
    ///     Some("First Thought".to_string()),
    ///     "This is a standalone thought".to_string(),
    ///     vec![],
    ///     vec![],
    /// ).unwrap();
    ///
    /// // Add a second thought that mentions the first one in its content
    /// graph.create_thought(
    ///     thought2_id.clone(),
    ///     Some("Second Thought".to_string()),
    ///     "This thought references [thought1] using square brackets".to_string(),
    ///     vec![],
    ///     vec![],
    /// ).unwrap();
    ///
    /// // Process auto-references in the second thought
    /// let added_refs = graph.process_auto_references(&thought2_id).unwrap();
    ///
    /// // The first thought should now be referenced by the second
    /// assert_eq!(added_refs.len(), 1);
    /// assert_eq!(added_refs[0], thought1_id);
    /// ```
    pub fn process_auto_references(&mut self, thought_id: &ThoughtID) -> Result<Vec<ThoughtID>> {
        let mut added_refs = Vec::new();
        
        // Clone the thought to extract references
        if let Some(thought) = self.thoughts.get(thought_id).cloned() {
            let content_refs = thought.extract_references_from_content();
            
            // Create updated thought with new references
            let mut updated_thought = thought;
            
            for ref_id in &content_refs {
                // Skip self-references and already existing references
                if ref_id == thought_id || updated_thought.references.iter().any(|r| &r.id == ref_id) {
                    continue;
                }
                
                // Only add reference if the target thought exists
                if self.thoughts.contains_key(ref_id) {
                    updated_thought.add_reference(Reference::new(
                        ref_id.clone(),
                        format!("Auto-reference from [{}]", ref_id.id),
                        Utc::now(),
                    ));
                    added_refs.push(ref_id.clone());
                }
            }
            
            // Update the thought with new references
            if !added_refs.is_empty() {
                self.command(&Command::PutThought {
                    id: thought_id.clone(),
                    thought: updated_thought,
                });
            }
        }
        
        Ok(added_refs)
    }
    
    /// Create a new tag with the given parameters
    pub fn create_tag(&mut self, id: TagID, description: String) -> Result<&Tag> {
        let tag = Tag::new(description);
        self.command(&Command::PutTag {
            id: id.clone(),
            tag,
        });
        
        self.tags.get(&id).ok_or_else(|| ThoughtGraphError::TagNotFound(id.id.clone()))
    }
    
    /// Get a list of all thought IDs in the graph
    pub fn list_thoughts(&self) -> Vec<&ThoughtID> {
        self.thoughts.keys().collect()
    }
    
    /// Get a list of all tag IDs in the graph
    pub fn list_tags(&self) -> Vec<&TagID> {
        self.tags.keys().collect()
    }
    
    /// Find thoughts matching a query and return the actual thoughts (not just IDs).
    ///
    /// This is a convenience method that extends the `query` method by returning the
    /// actual thought objects along with their IDs, rather than just the IDs.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to execute against the graph
    ///
    /// # Returns
    ///
    /// A vector of tuples containing thought IDs and their corresponding thought objects
    ///
    /// # Example
    ///
    /// ```
    /// use thoughtgraph::{ThoughtGraph, ThoughtID, TagID, Thought, Tag, Query, Command};
    ///
    /// let mut graph = ThoughtGraph::new();
    ///
    /// // Add a tag and a thought
    /// let tag_id = TagID::new("example".to_string());
    /// graph.command(&Command::PutTag {
    ///     id: tag_id.clone(),
    ///     tag: Tag::new("Example tag".to_string()),
    /// });
    ///
    /// let thought_id = ThoughtID::new("thought1".to_string());
    /// graph.command(&Command::PutThought {
    ///     id: thought_id.clone(),
    ///     thought: Thought::new(
    ///         Some("Example".to_string()),
    ///         "Content".to_string(),
    ///         vec![tag_id.clone()],
    ///         vec![],
    ///     ),
    /// });
    ///
    /// // Find thoughts with the tag
    /// let results = graph.find_thoughts(&Query::Tag(tag_id));
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].0, &thought_id);
    /// assert_eq!(results[0].1.title, Some("Example".to_string()));
    /// ```
    pub fn find_thoughts<'a>(&'a self, query: &Query) -> Vec<(&'a ThoughtID, &'a Thought)> {
        self.query(query)
            .iter()
            .filter_map(|id| {
                self.thoughts.get_key_value(id)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Helper function to create a thought ID
    fn create_thought_id(id: &str) -> ThoughtID {
        ThoughtID::new(id.to_string())
    }

    // Helper function to create a tag ID
    fn create_tag_id(id: &str) -> TagID {
        TagID::new(id.to_string())
    }

    // Helper function to create a reference
    fn create_reference(id: &str, notes: &str) -> Reference {
        Reference::new(
            create_thought_id(id),
            notes.to_string(),
            Utc::now(),
        )
    }
    
    #[test]
    fn test_extract_references_from_content() {
        // Test extracting references from content
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "This references [thought1] and [thought2] and [invalid-] but not just plain text.".to_string(),
            vec![],
            vec![],
        );
        
        let refs = thought.extract_references_from_content();
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&create_thought_id("thought1")));
        assert!(refs.contains(&create_thought_id("thought2")));
        assert!(refs.contains(&create_thought_id("invalid-")));
    }
    
    #[test]
    fn test_auto_references() {
        // Test automatically adding references from content
        let mut graph = ThoughtGraph::new();
        
        // Create some thoughts first
        let thought1_id = create_thought_id("thought1");
        let thought2_id = create_thought_id("thought2");
        let thought3_id = create_thought_id("thought3");
        
        let thought1 = Thought::new(
            Some("First Thought".to_string()),
            "This is the first thought.".to_string(),
            vec![],
            vec![],
        );
        
        let thought2 = Thought::new(
            Some("Second Thought".to_string()),
            "This is the second thought.".to_string(),
            vec![],
            vec![],
        );
        
        // Third thought references the first two using [thought_id] format
        let thought3 = Thought::new(
            Some("Third Thought".to_string()),
            "This references [thought1] and [thought2] automatically.".to_string(),
            vec![],
            vec![],
        );
        
        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: thought1,
        });
        
        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: thought2,
        });
        
        graph.command(&Command::PutThought {
            id: thought3_id.clone(),
            thought: thought3,
        });
        
        // Process auto-references
        let added_refs = graph.process_auto_references(&thought3_id).unwrap();
        
        // Check that references were added
        assert_eq!(added_refs.len(), 2);
        assert!(added_refs.contains(&thought1_id));
        assert!(added_refs.contains(&thought2_id));
        
        // Check that the references are in the thought
        let updated_thought3 = graph.get_thought(&thought3_id).unwrap();
        assert_eq!(updated_thought3.references.len(), 2);
        assert!(updated_thought3.references.iter().any(|r| r.id == thought1_id));
        assert!(updated_thought3.references.iter().any(|r| r.id == thought2_id));
        
        // Check that backreferences are correctly set up
        let backlinks_to_thought1 = graph.get_backlinks(&thought1_id);
        let backlinks_to_thought2 = graph.get_backlinks(&thought2_id);
        
        assert_eq!(backlinks_to_thought1.len(), 1);
        assert_eq!(backlinks_to_thought2.len(), 1);
        assert!(backlinks_to_thought1.contains(&thought3_id));
        assert!(backlinks_to_thought2.contains(&thought3_id));
    }

    #[test]
    fn test_empty_graph() {
        // Test creating an empty graph and querying it
        let graph = ThoughtGraph::new();

        // An empty graph should return empty results for any query
        let result = graph.query(&Query::Tag(create_tag_id("nonexistent")));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_put_and_query_thought() {
        // Test adding a thought and then retrieving it
        let mut graph = ThoughtGraph::new();

        let thought_id = create_thought_id("thought1");
        let tag_id = create_tag_id("tag1");

        // Add a tag first
        let tag = Tag::new("Test tag".to_string());
        graph.command(&Command::PutTag {
            id: tag_id.clone(),
            tag,
        });

        // Create and add a thought with the tag
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "This is a test thought.".to_string(),
            vec![tag_id.clone()],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought,
        });

        // Query for thoughts with the tag
        let result = graph.query(&Query::Tag(tag_id.clone()));
        assert_eq!(result.len(), 1);
        assert!(result.contains(&thought_id));
    }

    #[test]
    fn test_references() {
        // Test adding thoughts with references and querying them
        let mut graph = ThoughtGraph::new();

        let thought1_id = create_thought_id("thought1");
        let thought2_id = create_thought_id("thought2");

        // Create and add the first thought
        let thought1 = Thought::new(
            Some("First Thought".to_string()),
            "This is the first thought.".to_string(),
            vec![],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: thought1,
        });

        // Create and add a second thought that references the first
        let thought2 = Thought::new(
            Some("Second Thought".to_string()),
            "This references the first thought.".to_string(),
            vec![],
            vec![create_reference("thought1", "Important reference")],
        );

        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: thought2,
        });

        // Query for thoughts that reference thought1
        let references_result = graph.query(&Query::References(thought1_id.clone()));
        assert_eq!(references_result.len(), 1);
        assert!(references_result.contains(&thought2_id));

        // Query for thoughts that are referenced by thought2
        let referenced_by_result = graph.query(&Query::ReferencedBy(thought2_id.clone()));
        assert_eq!(referenced_by_result.len(), 1);
        assert!(referenced_by_result.contains(&thought1_id));
    }

    #[test]
    fn test_delete_thought() {
        // Test deleting a thought
        let mut graph = ThoughtGraph::new();

        let thought_id = create_thought_id("thought1");
        let tag_id = create_tag_id("tag1");

        // Add a tag
        let tag = Tag::new("Test tag".to_string());
        graph.command(&Command::PutTag {
            id: tag_id.clone(),
            tag,
        });

        // Add a thought
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "This is a test thought.".to_string(),
            vec![tag_id.clone()],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought,
        });

        // Delete the thought
        graph.command(&Command::DeleteThought {
            id: thought_id.clone(),
        });

        // Query should return empty results
        let result = graph.query(&Query::Tag(tag_id.clone()));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_delete_tag() {
        // Test deleting a tag
        let mut graph = ThoughtGraph::new();

        let thought_id = create_thought_id("thought1");
        let tag_id = create_tag_id("tag1");

        // Add a tag
        let tag = Tag::new("Test tag".to_string());
        graph.command(&Command::PutTag {
            id: tag_id.clone(),
            tag,
        });

        // Add a thought with the tag
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "This is a test thought.".to_string(),
            vec![tag_id.clone()],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought,
        });

        // Delete the tag
        graph.command(&Command::DeleteTag { id: tag_id.clone() });

        // The thought should still exist, but query by the tag should return empty results
        let result = graph.query(&Query::Tag(tag_id.clone()));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_complex_queries() {
        // Test complex queries with And and Or
        let mut graph = ThoughtGraph::new();

        // Create two tags
        let tag1_id = create_tag_id("tag1");
        let tag2_id = create_tag_id("tag2");

        graph.command(&Command::PutTag {
            id: tag1_id.clone(),
            tag: Tag::new("Tag 1".to_string()),
        });

        graph.command(&Command::PutTag {
            id: tag2_id.clone(),
            tag: Tag::new("Tag 2".to_string()),
        });

        // Create three thoughts with different tag combinations
        let thought1_id = create_thought_id("thought1"); // has tag1
        let thought2_id = create_thought_id("thought2"); // has tag2
        let thought3_id = create_thought_id("thought3"); // has both tag1 and tag2

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: Thought::new(
                Some("Thought 1".to_string()),
                "Has tag1 only".to_string(),
                vec![tag1_id.clone()],
                vec![],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: Thought::new(
                Some("Thought 2".to_string()),
                "Has tag2 only".to_string(),
                vec![tag2_id.clone()],
                vec![],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought3_id.clone(),
            thought: Thought::new(
                Some("Thought 3".to_string()),
                "Has both tag1 and tag2".to_string(),
                vec![tag1_id.clone(), tag2_id.clone()],
                vec![],
            ),
        });

        // Test OR query: thoughts with either tag1 or tag2
        let or_query = Query::Or(vec![
            Box::new(Query::Tag(tag1_id.clone())),
            Box::new(Query::Tag(tag2_id.clone())),
        ]);

        let or_result = graph.query(&or_query);
        assert_eq!(or_result.len(), 3);
        assert!(or_result.contains(&thought1_id));
        assert!(or_result.contains(&thought2_id));
        assert!(or_result.contains(&thought3_id));

        // Test AND query: thoughts with both tag1 and tag2
        let and_query = Query::And(vec![
            Box::new(Query::Tag(tag1_id.clone())),
            Box::new(Query::Tag(tag2_id.clone())),
        ]);

        let and_result = graph.query(&and_query);
        assert_eq!(and_result.len(), 1);
        assert!(and_result.contains(&thought3_id));
    }

    #[test]
    fn test_circular_references() {
        // Test circular references between thoughts
        let mut graph = ThoughtGraph::new();

        let thought1_id = create_thought_id("thought1");
        let thought2_id = create_thought_id("thought2");

        // Create thought1 that initially doesn't reference anything
        let thought1 = Thought::new(
            Some("First Thought".to_string()),
            "This is the first thought.".to_string(),
            vec![],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: thought1,
        });

        // Create thought2 that references thought1
        let thought2 = Thought::new(
            Some("Second Thought".to_string()),
            "This references the first thought.".to_string(),
            vec![],
            vec![create_reference("thought1", "Reference to thought1")],
        );

        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: thought2,
        });

        // Now update thought1 to reference thought2, creating a circular reference
        let updated_thought1 = Thought::new(
            Some("Updated First Thought".to_string()),
            "This now references the second thought.".to_string(),
            vec![],
            vec![create_reference("thought2", "Reference to thought2")],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: updated_thought1,
        });

        // Check that references are correctly tracked in both directions
        let references_to_thought1 = graph.query(&Query::References(thought1_id.clone()));
        let references_to_thought2 = graph.query(&Query::References(thought2_id.clone()));

        assert_eq!(references_to_thought1.len(), 1);
        assert_eq!(references_to_thought2.len(), 1);
        assert!(references_to_thought1.contains(&thought2_id));
        assert!(references_to_thought2.contains(&thought1_id));

        // Check backreferences using the accessor method
        let backlinks_to_thought1 = graph.get_backlinks(&thought1_id);
        let backlinks_to_thought2 = graph.get_backlinks(&thought2_id);

        assert_eq!(backlinks_to_thought1.len(), 1);
        assert_eq!(backlinks_to_thought2.len(), 1);
        assert!(backlinks_to_thought1.contains(&thought2_id));
        assert!(backlinks_to_thought2.contains(&thought1_id));
    }

    #[test]
    fn test_updating_references() {
        // Test updating a thought's references
        let mut graph = ThoughtGraph::new();

        let thought1_id = create_thought_id("thought1");
        let thought2_id = create_thought_id("thought2");
        let thought3_id = create_thought_id("thought3");

        // Add three thoughts with no references initially
        let thought1 = Thought::new(
            Some("Thought 1".to_string()),
            "First thought.".to_string(),
            vec![],
            vec![],
        );

        let thought2 = Thought::new(
            Some("Thought 2".to_string()),
            "Second thought.".to_string(),
            vec![],
            vec![],
        );

        let thought3 = Thought::new(
            Some("Thought 3".to_string()),
            "Third thought.".to_string(),
            vec![],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: thought1,
        });

        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: thought2,
        });

        graph.command(&Command::PutThought {
            id: thought3_id.clone(),
            thought: thought3,
        });

        // Update thought1 to reference thought2
        let updated_thought1 = Thought::new(
            Some("Updated Thought 1".to_string()),
            "Now references thought2.".to_string(),
            vec![],
            vec![create_reference("thought2", "Reference to thought2")],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: updated_thought1,
        });

        // Check that reference and backreference are correctly tracked
        let references_from_thought1 = graph.query(&Query::ReferencedBy(thought1_id.clone()));
        assert_eq!(references_from_thought1.len(), 1);
        assert!(references_from_thought1.contains(&thought2_id));

        let backlinks_to_thought2 = graph.get_backlinks(&thought2_id);
        assert_eq!(backlinks_to_thought2.len(), 1);
        assert!(backlinks_to_thought2.contains(&thought1_id));

        // Update thought1 again to reference thought3 instead of thought2
        let updated_thought1_again = Thought::new(
            Some("Updated Thought 1 Again".to_string()),
            "Now references thought3 instead of thought2.".to_string(),
            vec![],
            vec![create_reference("thought3", "Reference to thought3")],
        );

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: updated_thought1_again,
        });

        // Check that old references are removed and new ones are added
        let backlinks_to_thought2_after = graph.get_backlinks(&thought2_id);
        let backlinks_to_thought3 = graph.get_backlinks(&thought3_id);

        assert_eq!(backlinks_to_thought2_after.len(), 0);
        assert_eq!(backlinks_to_thought3.len(), 1);
        assert!(backlinks_to_thought3.contains(&thought1_id));
    }

    #[test]
    fn test_multiple_backreferences() {
        // Test multiple thoughts referencing the same thought
        let mut graph = ThoughtGraph::new();

        let central_thought_id = create_thought_id("central");
        let ref1_id = create_thought_id("ref1");
        let ref2_id = create_thought_id("ref2");
        let ref3_id = create_thought_id("ref3");

        // Create a central thought
        let central_thought = Thought::new(
            Some("Central Thought".to_string()),
            "This thought will be referenced by multiple others.".to_string(),
            vec![],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: central_thought_id.clone(),
            thought: central_thought,
        });

        // Create three thoughts that all reference the central thought
        let ref1 = Thought::new(
            Some("Reference 1".to_string()),
            "First reference to central.".to_string(),
            vec![],
            vec![create_reference("central", "First reference")],
        );

        let ref2 = Thought::new(
            Some("Reference 2".to_string()),
            "Second reference to central.".to_string(),
            vec![],
            vec![create_reference("central", "Second reference")],
        );

        let ref3 = Thought::new(
            Some("Reference 3".to_string()),
            "Third reference to central.".to_string(),
            vec![],
            vec![create_reference("central", "Third reference")],
        );

        graph.command(&Command::PutThought {
            id: ref1_id.clone(),
            thought: ref1,
        });

        graph.command(&Command::PutThought {
            id: ref2_id.clone(),
            thought: ref2,
        });

        graph.command(&Command::PutThought {
            id: ref3_id.clone(),
            thought: ref3,
        });

        // Check that all backreferences are tracked
        let references_to_central = graph.query(&Query::References(central_thought_id.clone()));
        let backlinks_to_central = graph.get_backlinks(&central_thought_id);

        assert_eq!(references_to_central.len(), 3);
        assert_eq!(backlinks_to_central.len(), 3);
        assert!(references_to_central.contains(&ref1_id));
        assert!(references_to_central.contains(&ref2_id));
        assert!(references_to_central.contains(&ref3_id));

        // Delete one of the referencing thoughts and ensure backlinks are updated
        graph.command(&Command::DeleteThought { id: ref2_id.clone() });

        let backlinks_after_delete = graph.get_backlinks(&central_thought_id);
        assert_eq!(backlinks_after_delete.len(), 2);
        assert!(backlinks_after_delete.contains(&ref1_id));
        assert!(backlinks_after_delete.contains(&ref3_id));
        assert!(!backlinks_after_delete.contains(&ref2_id));
    }

    #[test]
    fn test_cascading_deletion() {
        // Test what happens when deleting a thought that is referenced by others
        let mut graph = ThoughtGraph::new();

        let central_thought_id = create_thought_id("central");
        let ref1_id = create_thought_id("ref1");
        let ref2_id = create_thought_id("ref2");

        // Create central thought
        let central_thought = Thought::new(
            Some("Central Thought".to_string()),
            "This will be deleted.".to_string(),
            vec![],
            vec![],
        );

        graph.command(&Command::PutThought {
            id: central_thought_id.clone(),
            thought: central_thought,
        });

        // Create thoughts that reference the central thought
        let ref1 = Thought::new(
            Some("Reference 1".to_string()),
            "References central.".to_string(),
            vec![],
            vec![create_reference("central", "Reference to central")],
        );

        let ref2 = Thought::new(
            Some("Reference 2".to_string()),
            "Also references central.".to_string(),
            vec![],
            vec![create_reference("central", "Another reference to central")],
        );

        graph.command(&Command::PutThought {
            id: ref1_id.clone(),
            thought: ref1,
        });

        graph.command(&Command::PutThought {
            id: ref2_id.clone(),
            thought: ref2,
        });

        // Verify references before deletion
        let refs_before = graph.query(&Query::References(central_thought_id.clone()));
        assert_eq!(refs_before.len(), 2);

        // Delete the central thought
        graph.command(&Command::DeleteThought { id: central_thought_id.clone() });

        // Verify the referencing thoughts still exist
        assert!(graph.get_thought(&ref1_id).is_some());
        assert!(graph.get_thought(&ref2_id).is_some());

        // Verify the central thought is gone
        assert!(graph.get_thought(&central_thought_id).is_none());

        // Verify that ReferencedBy queries for deleted thought return empty results
        let referenced_by_result = graph.query(&Query::ReferencedBy(central_thought_id.clone()));
        assert_eq!(referenced_by_result.len(), 0);

        // Verify that queries for references to the deleted thought return empty results
        // (even though the referencing thoughts still contain the references)
        let references_result = graph.query(&Query::References(central_thought_id.clone()));
        assert_eq!(references_result.len(), 0);
    }

    #[test]
    fn test_complex_query_combinations() {
        // Test more complex query combinations
        let mut graph = ThoughtGraph::new();

        // Create tags
        let tag1_id = create_tag_id("tag1");
        let tag2_id = create_tag_id("tag2");
        let tag3_id = create_tag_id("tag3");

        graph.command(&Command::PutTag {
            id: tag1_id.clone(),
            tag: Tag::new("Tag 1".to_string()),
        });

        graph.command(&Command::PutTag {
            id: tag2_id.clone(),
            tag: Tag::new("Tag 2".to_string()),
        });

        graph.command(&Command::PutTag {
            id: tag3_id.clone(),
            tag: Tag::new("Tag 3".to_string()),
        });

        // Create thoughts with various combinations of tags and references
        let thought1_id = create_thought_id("thought1"); // tag1, tag2
        let thought2_id = create_thought_id("thought2"); // tag2, tag3, references thought1
        let thought3_id = create_thought_id("thought3"); // tag1, tag3, references thought2
        let thought4_id = create_thought_id("thought4"); // tag3 only
        let thought5_id = create_thought_id("thought5"); // no tags, references thought1

        graph.command(&Command::PutThought {
            id: thought1_id.clone(),
            thought: Thought::new(
                Some("Thought 1".to_string()),
                "Has tag1 and tag2".to_string(),
                vec![tag1_id.clone(), tag2_id.clone()],
                vec![],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought2_id.clone(),
            thought: Thought::new(
                Some("Thought 2".to_string()),
                "Has tag2, tag3, references thought1".to_string(),
                vec![tag2_id.clone(), tag3_id.clone()],
                vec![create_reference("thought1", "Reference to thought1")],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought3_id.clone(),
            thought: Thought::new(
                Some("Thought 3".to_string()),
                "Has tag1, tag3, references thought2".to_string(),
                vec![tag1_id.clone(), tag3_id.clone()],
                vec![create_reference("thought2", "Reference to thought2")],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought4_id.clone(),
            thought: Thought::new(
                Some("Thought 4".to_string()),
                "Has tag3 only".to_string(),
                vec![tag3_id.clone()],
                vec![],
            ),
        });

        graph.command(&Command::PutThought {
            id: thought5_id.clone(),
            thought: Thought::new(
                Some("Thought 5".to_string()),
                "No tags, references thought1".to_string(),
                vec![],
                vec![create_reference("thought1", "Another reference to thought1")],
            ),
        });

        // Test: thoughts with tag1 AND that reference thought2
        let query1 = Query::And(vec![
            Box::new(Query::Tag(tag1_id.clone())),
            Box::new(Query::References(thought2_id.clone())),
        ]);
        let result1 = graph.query(&query1);
        assert_eq!(result1.len(), 1);
        assert!(result1.contains(&thought3_id));

        // Test: thoughts with tag3 OR that reference thought1
        let query2 = Query::Or(vec![
            Box::new(Query::Tag(tag3_id.clone())),
            Box::new(Query::References(thought1_id.clone())),
        ]);
        let result2 = graph.query(&query2);
        assert_eq!(result2.len(), 4);
        assert!(result2.contains(&thought2_id));
        assert!(result2.contains(&thought3_id));
        assert!(result2.contains(&thought4_id));
        assert!(result2.contains(&thought5_id));

        // Test: (thoughts with tag1 AND tag3) OR (thoughts referenced by thought3)
        let query3 = Query::Or(vec![
            Box::new(Query::And(vec![
                Box::new(Query::Tag(tag1_id.clone())),
                Box::new(Query::Tag(tag3_id.clone())),
            ])),
            Box::new(Query::ReferencedBy(thought3_id.clone())),
        ]);
        let result3 = graph.query(&query3);
        assert_eq!(result3.len(), 2);
        assert!(result3.contains(&thought2_id));
        assert!(result3.contains(&thought3_id));
    }

    #[test]
    fn test_empty_queries() {
        // Test edge cases with empty AND/OR queries
        let mut graph = ThoughtGraph::new();
        
        let thought_id = create_thought_id("thought1");
        let tag_id = create_tag_id("tag1");
        
        graph.command(&Command::PutTag {
            id: tag_id.clone(),
            tag: Tag::new("Tag 1".to_string()),
        });
        
        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought: Thought::new(
                Some("Test Thought".to_string()),
                "Test content".to_string(),
                vec![tag_id.clone()],
                vec![],
            ),
        });
        
        // Empty AND query should return empty set
        let empty_and = Query::And(vec![]);
        let and_result = graph.query(&empty_and);
        assert_eq!(and_result.len(), 0);
        
        // Empty OR query should return empty set
        let empty_or = Query::Or(vec![]);
        let or_result = graph.query(&empty_or);
        assert_eq!(or_result.len(), 0);
        
        // AND with one subquery should behave like the subquery
        let and_single = Query::And(vec![Box::new(Query::Tag(tag_id.clone()))]);
        let and_single_result = graph.query(&and_single);
        assert_eq!(and_single_result.len(), 1);
        assert!(and_single_result.contains(&thought_id));
    }

    #[test]
    fn test_nonexistent_references() {
        // Test handling of references to thoughts that don't exist
        let mut graph = ThoughtGraph::new();
        
        let thought_id = create_thought_id("thought1");
        let nonexistent_id = create_thought_id("nonexistent");
        
        // Create a thought with reference to a nonexistent thought
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "References a nonexistent thought".to_string(),
            vec![],
            vec![create_reference("nonexistent", "Reference to nowhere")],
        );
        
        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought,
        });
        
        // Test References query - should work normally
        let refs_to_nonexistent = graph.query(&Query::References(nonexistent_id.clone()));
        assert_eq!(refs_to_nonexistent.len(), 1);
        assert!(refs_to_nonexistent.contains(&thought_id));
        
        // Test ReferencedBy query - should return empty set for nonexistent thought
        let refs_by_nonexistent = graph.query(&Query::ReferencedBy(nonexistent_id.clone()));
        assert_eq!(refs_by_nonexistent.len(), 0);
        
        // Test get_backlinks - should return empty vector for nonexistent thought
        let backlinks = graph.get_backlinks(&nonexistent_id);
        assert_eq!(backlinks.len(), 1);
        assert!(backlinks.contains(&thought_id));
    }

    #[test]
    fn test_accessor_methods() {
        // Test the get_thought, get_tag, and get_backlinks methods
        let mut graph = ThoughtGraph::new();
        
        let thought_id = create_thought_id("thought1");
        let tag_id = create_tag_id("tag1");
        let ref_id = create_thought_id("ref1");
        
        let tag = Tag::new("Test Tag".to_string());
        graph.command(&Command::PutTag {
            id: tag_id.clone(),
            tag: tag.clone(),
        });
        
        let thought = Thought::new(
            Some("Test Thought".to_string()),
            "Test content".to_string(),
            vec![tag_id.clone()],
            vec![],
        );
        
        let ref_thought = Thought::new(
            Some("Reference Thought".to_string()),
            "References the test thought".to_string(),
            vec![],
            vec![create_reference("thought1", "Test reference")],
        );
        
        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought: thought.clone(),
        });
        
        graph.command(&Command::PutThought {
            id: ref_id.clone(),
            thought: ref_thought.clone(),
        });
        
        // Test get_thought
        let retrieved_thought = graph.get_thought(&thought_id);
        assert!(retrieved_thought.is_some());
        assert_eq!(retrieved_thought.unwrap().title, thought.title);
        
        // Test get_tag
        let retrieved_tag = graph.get_tag(&tag_id);
        assert!(retrieved_tag.is_some());
        assert_eq!(retrieved_tag.unwrap().description, tag.description);
        
        // Test get_backlinks
        let backlinks = graph.get_backlinks(&thought_id);
        assert_eq!(backlinks.len(), 1);
        assert!(backlinks.contains(&ref_id));
        
        // Test nonexistent IDs
        let nonexistent_id = create_thought_id("nonexistent");
        assert!(graph.get_thought(&nonexistent_id).is_none());
        assert!(graph.get_tag(&create_tag_id("nonexistent")).is_none());
    }

    #[test]
    fn test_self_reference() {
        // Test a thought that references itself
        let mut graph = ThoughtGraph::new();
        
        let thought_id = create_thought_id("self_ref");
        
        // Create a thought that references itself
        let thought = Thought::new(
            Some("Self-referential".to_string()),
            "This thought references itself.".to_string(),
            vec![],
            vec![create_reference("self_ref", "Self reference")],
        );
        
        // Try to add the self-referential thought
        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought: thought.clone(),
        });
        
        // Verify the thought was added successfully
        let retrieved = graph.get_thought(&thought_id);
        assert!(retrieved.is_some());
        
        // Check that self-reference is properly tracked
        let refs_to_self = graph.query(&Query::References(thought_id.clone()));
        assert_eq!(refs_to_self.len(), 1);
        assert!(refs_to_self.contains(&thought_id));
        
        // Check that self-reference appears in backreferences
        let backrefs = graph.get_backlinks(&thought_id);
        assert_eq!(backrefs.len(), 1);
        assert!(backrefs.contains(&thought_id));
        
        // Check that ReferencedBy also works correctly
        let referenced_by = graph.query(&Query::ReferencedBy(thought_id.clone()));
        assert_eq!(referenced_by.len(), 1);
        assert!(referenced_by.contains(&thought_id));
        
        // Test updating the self-referential thought
        let updated_thought = Thought::new(
            Some("Updated Self-referential".to_string()),
            "No longer references itself.".to_string(),
            vec![],
            vec![],
        );
        
        graph.command(&Command::PutThought {
            id: thought_id.clone(),
            thought: updated_thought,
        });
        
        // Verify backlinks were properly updated
        let backrefs_after = graph.get_backlinks(&thought_id);
        assert_eq!(backrefs_after.len(), 0);
    }
}
