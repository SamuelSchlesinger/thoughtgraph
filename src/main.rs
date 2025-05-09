use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use colored::*;
use console::{style, Term};
use dialoguer::Input;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use thoughtgraph::{Reference, Tag, TagID, Thought, ThoughtGraph, ThoughtID};
use thoughtgraph::ui;
use thoughtgraph::visualization::{generate_graph_data, generate_focused_graph};

/// Default filename for the thought graph
const DEFAULT_FILENAME: &str = "thoughts.bin";

/// Maximum length of thought content to display in list view
const MAX_DISPLAY_LENGTH: usize = 70;

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about = "Command-line tool for managing thoughts in a graph", long_about = None)]
struct Cli {
    /// Path to the thoughts binary file
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new thought
    Create {
        /// ID of the thought (optional, will be prompted if not provided)
        #[arg(long)]
        id: Option<String>,

        /// Title of the thought (optional, will be prompted if not provided)
        #[arg(long)]
        title: Option<String>,

        /// Content of the thought (if not provided, will open an editor)
        #[arg(long)]
        content: Option<String>,

        /// Tags to add to the thought (can be repeated)
        #[arg(long = "tag")]
        tags: Vec<String>,

        /// IDs of thoughts to reference (can be repeated)
        #[arg(long = "ref")]
        references: Vec<String>,
    },

    /// List thoughts in the graph
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// View details of a specific thought
    View {
        /// ID of the thought to view
        id: String,
    },

    /// Edit an existing thought
    Edit {
        /// ID of the thought to edit
        id: String,
    },

    /// Delete a thought
    Delete {
        /// ID of the thought to delete
        id: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Add a tag to a thought
    Tag {
        /// ID of the thought to tag
        id: String,
        
        /// ID of the tag to add
        tag: String,
        
        /// Description of the tag if it doesn't exist yet
        #[arg(long)]
        description: Option<String>,
    },

    /// Remove a tag from a thought
    Untag {
        /// ID of the thought to remove the tag from
        id: String,
        
        /// ID of the tag to remove
        tag: String,
    },

    /// Add a reference from one thought to another
    Reference {
        /// ID of the thought that will contain the reference
        #[arg(name = "from")]
        from_id: String,
        
        /// ID of the thought that will be referenced
        #[arg(name = "to")]
        to_id: String,
        
        /// Notes about the reference
        #[arg(long)]
        notes: Option<String>,
    },

    /// Search for thoughts matching a query
    Search {
        /// Search query terms (searches in titles and content)
        query: Vec<String>,
    },

    /// List all available tags
    Tags,

    /// Initialize a new empty thought graph
    Init,
    
    /// Visualize the thought graph
    Visualize {
        /// Format for visualization (dot or json)
        #[arg(short, long, default_value = "dot")]
        format: String,
        
        /// Focus visualization on a specific thought
        #[arg(short, long)]
        focus: Option<String>,
        
        /// Depth limit for focused visualization (default: 1)
        #[arg(short, long, default_value = "1")]
        depth: usize,
        
        /// Output file (if not specified, outputs to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Start an interactive CLI session
    Interactive,
    
    /// Browse thoughts interactively
    Browse,
}

/// Interactive CLI interface for ThoughtGraph
fn interactive_mode(file_path: &Path) -> Result<()> {
    let term = Term::stdout();
    
    // Display welcome message
    term.clear_screen()?;
    println!("{}", style("Welcome to ThoughtGraph Interactive Mode").bold().cyan());
    println!("Managing thoughts at: {}", style(file_path.display()).green());
    println!("");
    
    // Load the graph
    let mut graph = load_or_create_graph(file_path)?;
    
    loop {
        // Display stats
        let thought_count = graph.thoughts.len();
        let tag_count = graph.tags.len();
        println!("\n{} | {} | {}", 
            style(format!("Thoughts: {}", thought_count)).dim(),
            style(format!("Tags: {}", tag_count)).dim(),
            style(format!("File: {}", file_path.display())).dim()
        );
        
        // Show command selector
        let command_index = ui::command_selector()?;
        term.clear_screen()?;
        
        let result = match command_index {
            0 => {
                // Create a new thought
                let id = Input::with_theme(&ui::get_theme())
                    .with_prompt("Enter a unique ID for the thought")
                    .interact()?;
                
                let title_str: String = Input::with_theme(&ui::get_theme())
                    .with_prompt("Enter a title (optional, press Enter to skip)")
                    .allow_empty(true)
                    .interact()?;
                
                let title = if title_str.is_empty() { None } else { Some(title_str) };
                
                // Get content by opening an editor
                let content = edit_in_external_editor("", "# Enter your thought content here")?;
                
                // Suggest tags based on existing ones
                let tags = if tag_count > 0 {
                    ui::select_tags(&graph, &[])?
                } else {
                    vec![]
                };
                
                // Suggest references
                let references = if thought_count > 0 {
                    if ui::confirm("Would you like to add references to other thoughts?", false)? {
                        let mut refs = Vec::new();
                        while let Some(ref_id) = ui::select_thought(&graph, "Select a thought to reference (ESC to finish)")? {
                            let notes = Input::with_theme(&ui::get_theme())
                                .with_prompt("Add optional notes about this reference")
                                .allow_empty(true)
                                .interact()?;
                            
                            refs.push(Reference::new(
                                ref_id,
                                notes,
                                Utc::now(),
                            ));
                        }
                        refs
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                // Create the thought
                create_thought(&mut graph, Some(id), title, Some(content), 
                    tags.iter().map(|t| t.id.clone()).collect(), 
                    references.iter().map(|r| r.id.id.clone()).collect())
            },
            1 => {
                // List thoughts
                if tag_count > 0 && ui::confirm("Would you like to filter by tag?", false)? {
                    let (tag_id, _) = ui::tag_selector(&graph)?;
                    list_thoughts(&graph, Some(tag_id.id))
                } else {
                    list_thoughts(&graph, None)
                }
            },
            2 => {
                // View thought
                if let Some(id) = ui::select_thought(&graph, "Select a thought to view")? {
                    view_thought(&graph, &id.id)
                } else {
                    println!("No thought selected.");
                    Ok(())
                }
            },
            3 => {
                // Edit thought
                if let Some(id) = ui::select_thought(&graph, "Select a thought to edit")? {
                    edit_thought(&mut graph, &id.id)
                } else {
                    println!("No thought selected.");
                    Ok(())
                }
            },
            4 => {
                // Delete thought
                if let Some(id) = ui::select_thought(&graph, "Select a thought to delete")? {
                    delete_thought(&mut graph, &id.id, false)
                } else {
                    println!("No thought selected.");
                    Ok(())
                }
            },
            5 => {
                // Tag a thought
                if let Some(id) = ui::select_thought(&graph, "Select a thought to tag")? {
                    let (tag_id, description) = ui::tag_selector(&graph)?;
                    tag_thought(&mut graph, &id.id, &tag_id.id, description)
                } else {
                    println!("No thought selected.");
                    Ok(())
                }
            },
            6 => {
                // Untag a thought
                if let Some(id) = ui::select_thought(&graph, "Select a thought to untag")? {
                    // Clone the necessary data to avoid borrow conflicts
                    let thought_tags = match graph.get_thought(&id) {
                        Some(thought) => thought.tags.clone(),
                        None => {
                            println!("Thought not found.");
                            return Ok(());
                        }
                    };

                    if thought_tags.is_empty() {
                        println!("This thought has no tags.");
                        Ok(())
                    } else {
                        let tag_items: Vec<String> = thought_tags.iter()
                            .map(|tag_id| {
                                let desc = graph.get_tag(tag_id)
                                    .map(|tag| format!(" - {}", tag.description))
                                    .unwrap_or_default();
                                format!("#{}{}", tag_id.id, desc)
                            })
                            .collect();

                        let selection = dialoguer::Select::with_theme(&ui::get_theme())
                            .with_prompt("Select a tag to remove")
                            .default(0)
                            .items(&tag_items)
                            .interact()?;

                        untag_thought(&mut graph, &id.id, &thought_tags[selection].id)
                    }
                } else {
                    println!("No thought selected.");
                    Ok(())
                }
            },
            7 => {
                // Add reference
                if thought_count < 2 {
                    println!("You need at least two thoughts to create a reference.");
                    Ok(())
                } else {
                    let from_id = ui::select_thought(&graph, "Select the source thought")?
                        .ok_or_else(|| anyhow::anyhow!("No thought selected"))?;
                    
                    let to_id = ui::select_thought(&graph, "Select the target thought")?
                        .ok_or_else(|| anyhow::anyhow!("No thought selected"))?;
                    
                    let notes: String = Input::with_theme(&ui::get_theme())
                        .with_prompt("Add optional notes about this reference")
                        .allow_empty(true)
                        .interact()?;
                    
                    let notes = if notes.is_empty() { None } else { Some(notes) };
                    
                    add_reference(&mut graph, &from_id.id, &to_id.id, notes)
                }
            },
            8 => {
                // Search
                let query: String = Input::with_theme(&ui::get_theme())
                    .with_prompt("Enter search terms")
                    .interact()?;
                
                search_thoughts(&graph, &query.split_whitespace().map(String::from).collect::<Vec<_>>())
            },
            9 => {
                // Browse thoughts interactively
                ui::browse_thoughts(&graph)
            },
            10 => {
                // List tags
                list_tags(&graph)
            },
            11 => {
                // Visualize
                let format_options = vec!["dot", "json"];
                let format_selection = dialoguer::Select::with_theme(&ui::get_theme())
                    .with_prompt("Select output format")
                    .default(0)
                    .items(&format_options)
                    .interact()?;
                
                let format = format_options[format_selection];
                
                let depth = if ui::confirm("Would you like to customize graph depth?", false)? {
                    let depth = dialoguer::Input::<usize>::with_theme(&ui::get_theme())
                        .with_prompt("Enter depth (1-5)")
                        .validate_with(|input: &usize| {
                            if *input >= 1 && *input <= 5 {
                                Ok(())
                            } else {
                                Err("Depth must be between 1 and 5")
                            }
                        })
                        .default(1)
                        .interact()?;
                    depth
                } else {
                    1
                };
                
                let use_file = ui::confirm("Would you like to save to a file?", true)?;
                
                let output = if use_file {
                    let default_filename = format!("thoughtgraph.{}", format);
                    let filename = dialoguer::Input::<String>::with_theme(&ui::get_theme())
                        .with_prompt("Enter output filename")
                        .default(default_filename)
                        .interact()?;
                    
                    Some(PathBuf::from(filename))
                } else {
                    None
                };
                
                visualize_graph(&graph, format, None, depth, output)
            },
            12 | _ => {
                // Exit
                if ui::confirm("Are you sure you want to exit?", false)? {
                    return Ok(());
                } else {
                    Ok(())
                }
            }
        };
        
        // Save graph changes if the command succeeded
        if result.is_ok() {
            ui::with_loading_progress("Saving changes...", || {
                graph.save_to_file(file_path)
            })?;

            // Add a pause after successful commands so users can see the output
            println!("\n{}", style("Press any key to continue...").dim());
            term.read_key()?;
        } else if let Err(e) = result {
            println!("\n{}", style(format!("Error: {}", e)).red());
            println!("Press any key to continue...");
            term.read_key()?;
        }

        term.clear_screen()?;
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Determine file path: either from argument or default
    let file_path = match cli.file {
        Some(path) => path,
        None => {
            let data_dir = dirs::data_dir()
                .context("Could not determine data directory for your platform")?;
            let app_dir = data_dir.join("thoughtgraph");
            fs::create_dir_all(&app_dir)
                .context("Failed to create application data directory")?;
            app_dir.join(DEFAULT_FILENAME)
        }
    };
    
    match cli.command {
        Commands::Init => init_graph(&file_path),
        Commands::Interactive => interactive_mode(&file_path),
        Commands::Browse => {
            let graph = load_or_create_graph(&file_path)?;
            ui::browse_thoughts(&graph)
        },
        _ => {
            // For all other commands, load the existing graph or create a new one
            let mut graph = load_or_create_graph(&file_path)?;
            
            let result = match cli.command {
                Commands::Create { id, title, content, tags, references } => {
                    create_thought(&mut graph, id, title, content, tags, references)
                }
                Commands::List { tag } => list_thoughts(&graph, tag),
                Commands::View { id } => view_thought(&graph, &id),
                Commands::Edit { id } => edit_thought(&mut graph, &id),
                Commands::Delete { id, force } => delete_thought(&mut graph, &id, force),
                Commands::Tag { id, tag, description } => tag_thought(&mut graph, &id, &tag, description),
                Commands::Untag { id, tag } => untag_thought(&mut graph, &id, &tag),
                Commands::Reference { from_id, to_id, notes } => add_reference(&mut graph, &from_id, &to_id, notes),
                Commands::Search { query } => search_thoughts(&graph, &query),
                Commands::Tags => list_tags(&graph),
                Commands::Visualize { format, focus, depth, output } => 
                    visualize_graph(&graph, &format, focus, depth, output),
                Commands::Init | Commands::Interactive | Commands::Browse => unreachable!(), // Handled above
            };
            
            // Save graph changes if the command succeeded
            if result.is_ok() {
                ui::with_loading_progress("Saving changes...", || {
                    graph.save_to_file(&file_path)
                })?;
            }
            
            result
        }
    }
}

/// Initialize a new empty thought graph
fn init_graph(file_path: &Path) -> Result<()> {
    if file_path.exists() {
        println!("A thought graph already exists at {}", file_path.display());
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Do you want to overwrite it with a new empty graph?")
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Operation cancelled.");
            return Ok(());
        }
    }
    
    let graph = ThoughtGraph::default();
    ui::with_loading_progress("Initializing new graph...", || {
        graph.save_to_file(file_path)
    })?;
    
    println!("Initialized a new thought graph at {}", file_path.display());
    Ok(())
}

/// Load an existing graph or create a new one
fn load_or_create_graph(file_path: &Path) -> Result<ThoughtGraph> {
    if file_path.exists() {
        ui::with_loading_progress("Loading thought graph...", || {
            ThoughtGraph::load_from_file(file_path)
                .context(format!("Failed to load thought graph from {}", file_path.display()))
        })
    } else {
        println!("No thought graph found at {}. Creating a new one.", file_path.display());
        let graph = ThoughtGraph::default();
        ui::with_loading_progress("Saving new thought graph...", || {
            graph.save_to_file(file_path)
        })?;
        Ok(graph)
    }
}

/// Create a new thought, prompting for any missing information
fn create_thought(
    graph: &mut ThoughtGraph,
    id: Option<String>,
    title: Option<String>,
    content: Option<String>,
    tags: Vec<String>,
    references: Vec<String>,
) -> Result<()> {
    // Ask for ID if not provided
    let id = match id {
        Some(id) => id,
        None => {
            if !io::stdin().is_terminal() {
                return Err(anyhow::anyhow!("ID is required in non-interactive mode"));
            }
            Input::<String>::new()
                .with_prompt("Enter a unique ID for the thought")
                .interact()?
        }
    };
    
    // Ask for title if not provided
    let title = match title {
        Some(t) => Some(t),
        None => {
            if !io::stdin().is_terminal() {
                None
            } else {
                let t: String = Input::new()
                    .with_prompt("Enter a title (optional, press Enter to skip)")
                    .allow_empty(true)
                    .interact()?;
                
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            }
        }
    };
    
    // Get content either from argument or by opening an editor
    let content = match content {
        Some(c) => c,
        None => {
            if !io::stdin().is_terminal() {
                return Err(anyhow::anyhow!("Content is required in non-interactive mode"));
            }
            edit_in_external_editor("", "# Enter your thought content here")?
        },
    };
    
    // Convert tags to TagIDs
    let tag_ids: Vec<TagID> = tags.into_iter()
        .map(|t| TagID::new(t))
        .collect();
    
    // Create any tags that don't exist yet
    for tag_id in &tag_ids {
        if !graph.tags.contains_key(tag_id) {
            let description = match std::io::stdin().is_terminal() {
                true => Input::<String>::new()
                    .with_prompt(format!("Enter description for new tag '{}'", tag_id.id))
                    .interact()?,
                false => format!("Description for tag '{}'", tag_id.id),
            };
            
            graph.create_tag(tag_id.clone(), description)?;
        }
    }
    
    // Convert references to References
    let refs: Vec<Reference> = references.into_iter()
        .filter_map(|r| {
            let r_clone = r.clone();
            let thought_id = ThoughtID::new(r);
            if graph.thoughts.contains_key(&thought_id) {
                Some(Reference::new(
                    thought_id,
                    "".to_string(),
                    Utc::now(),
                ))
            } else {
                eprintln!("Warning: Skipping reference to non-existent thought '{}'", r_clone);
                None
            }
        })
        .collect();
    
    // Create the thought
    let thought_id = ThoughtID::new(id.clone());
    
    ui::with_loading_progress("Creating thought...", || {
        graph.create_thought(
            thought_id.clone(),
            title,
            content,
            tag_ids,
            refs,
        )
    })?;
    
    // Process any auto-references in the format [thought_id]
    let auto_refs = ui::with_loading_progress("Processing auto-references...", || {
        graph.process_auto_references(&thought_id)
    })?;
    
    println!("Created thought '{}' successfully", id.green());
    
    // Report any auto-references that were added
    if !auto_refs.is_empty() {
        println!("Auto-added references to:");
        for ref_id in auto_refs {
            println!("  → {}", ref_id.id.blue());
        }
    }
    
    Ok(())
}

/// List thoughts in the graph, optionally filtering by tag
fn list_thoughts(graph: &ThoughtGraph, tag_filter: Option<String>) -> Result<()> {
    let thoughts = match tag_filter {
        Some(tag) => {
            let tag_id = TagID::new(tag.clone());
            if !graph.tags.contains_key(&tag_id) {
                return Err(anyhow::anyhow!("Tag '{}' not found", tag));
            }

            // Use the query functionality to find thoughts with this tag
            graph.find_thoughts(&thoughtgraph::Query::Tag(tag_id))
        },
        None => graph.thoughts.iter().map(|(id, thought)| (id, thought)).collect(),
    };

    // Use the enhanced display function
    ui::display_thought_list(graph, &thoughts, MAX_DISPLAY_LENGTH)?;

    // If in interactive mode, offer to select a thought to view
    if io::stdin().is_terminal() && !thoughts.is_empty() {
        if ui::confirm("Would you like to view one of these thoughts?", false)? {
            if let Some(id) = ui::select_thought(graph, "Select a thought to view")? {
                return view_thought(graph, &id.id);
            }
        }
    }

    Ok(())
}

/// View details of a specific thought
fn view_thought(graph: &ThoughtGraph, id: &str) -> Result<()> {
    let thought_id = ThoughtID::new(id.to_string());
    let thought = graph.get_thought(&thought_id)
        .ok_or_else(|| anyhow::anyhow!("Thought '{}' not found", id))?;
    
    // Use the enhanced display function
    ui::display_thought_details(graph, &thought_id, thought)?;
    
    // Ask if the user wants to explore related thoughts
    if io::stdin().is_terminal() && !thought.references.is_empty() && graph.get_backlinks(&thought_id).len() > 0 {
        if ui::confirm("Would you like to explore related thoughts?", false)? {
            ui::browse_thoughts(graph)?;
        }
    }
    
    Ok(())
}

/// Edit a thought using an external editor
fn edit_thought(graph: &mut ThoughtGraph, id: &str) -> Result<()> {
    let thought_id = ThoughtID::new(id.to_string());
    let thought = graph.get_thought(&thought_id)
        .ok_or_else(|| anyhow::anyhow!("Thought '{}' not found", id))?;
    
    // Check if we're in non-interactive mode
    if !io::stdin().is_terminal() {
        // For non-interactive testing, just update the timestamp
        let mut updated_thought = thought.clone();
        updated_thought.update_content(format!("{}\n(Updated non-interactively)", thought.contents));
        
        graph.command(&thoughtgraph::Command::PutThought {
            id: thought_id.clone(),
            thought: updated_thought,
        });
        
        println!("Thought '{}' updated non-interactively", id.green());
        return Ok(());
    }
    
    // Prepare content for editing
    let initial_content = format!(
        "# Title: {}\n\n{}", 
        thought.title.clone().unwrap_or_default(), 
        thought.contents
    );
    
    // Open in external editor
    let edited_content = edit_in_external_editor(&initial_content, "")?;
    
    // Parse the edited content
    let mut lines = edited_content.lines();
    let title_line = lines.next().unwrap_or_default();
    let title = if title_line.starts_with("# Title:") {
        let title_str = title_line["# Title:".len()..].trim();
        if title_str.is_empty() {
            None
        } else {
            Some(title_str.to_string())
        }
    } else {
        thought.title.clone()
    };
    
    // Skip empty lines after title
    let mut content_lines = Vec::new();
    let mut started = false;
    for line in lines {
        if !started && line.trim().is_empty() {
            continue;
        }
        started = true;
        content_lines.push(line);
    }
    let content = content_lines.join("\n");
    
    // Update the thought
    let mut updated_thought = thought.clone();
    updated_thought.update_title(title);
    updated_thought.update_content(content);
    
    ui::with_loading_progress("Updating thought...", || {
        graph.command(&thoughtgraph::Command::PutThought {
            id: thought_id.clone(),
            thought: updated_thought,
        });
    });
    
    // Process any auto-references in the format [thought_id]
    let auto_refs = ui::with_loading_progress("Processing auto-references...", || {
        graph.process_auto_references(&thought_id)
    })?;
    
    println!("Thought '{}' updated successfully", id.green());
    
    // Report any auto-references that were added
    if !auto_refs.is_empty() {
        println!("Auto-added references to:");
        for ref_id in auto_refs {
            println!("  → {}", ref_id.id.blue());
        }
    }
    
    Ok(())
}

/// Delete a thought
fn delete_thought(graph: &mut ThoughtGraph, id: &str, force: bool) -> Result<()> {
    let thought_id = ThoughtID::new(id.to_string());
    
    // Check if thought exists
    if !graph.thoughts.contains_key(&thought_id) {
        return Err(anyhow::anyhow!("Thought '{}' not found", id));
    }
    
    // Confirm deletion if not forced
    if !force {
        if io::stdin().is_terminal() {
            if !ui::confirm(&format!("Are you sure you want to delete thought '{}'?", id), false)? {
                println!("Deletion cancelled");
                return Ok(());
            }
        } else {
            // In non-interactive mode, we should require the --force flag
            return Err(anyhow::anyhow!("Deletion requires --force flag in non-interactive mode"));
        }
    }
    
    // Delete the thought with progress indicator
    ui::with_loading_progress(&format!("Deleting thought '{}'...", id), || {
        graph.command(&thoughtgraph::Command::DeleteThought {
            id: thought_id.clone(),
        });
    });
    
    println!("Thought '{}' deleted successfully", id.green());
    Ok(())
}

/// Add a tag to a thought
fn tag_thought(graph: &mut ThoughtGraph, id: &str, tag: &str, description: Option<String>) -> Result<()> {
    let thought_id = ThoughtID::new(id.to_string());
    let tag_id = TagID::new(tag.to_string());
    
    // Check if thought exists
    let thought = match graph.get_thought(&thought_id) {
        Some(t) => t.clone(),
        None => return Err(anyhow::anyhow!("Thought '{}' not found", id)),
    };
    
    // Create the tag if it doesn't exist
    if !graph.tags.contains_key(&tag_id) {
        let desc = match description {
            Some(d) => d,
            None => Input::with_theme(&ui::get_theme())
                .with_prompt(format!("Enter description for new tag '{}'", tag))
                .interact()?,
        };
        
        ui::with_loading_progress("Creating tag...", || {
            graph.create_tag(tag_id.clone(), desc)
        })?;
    }
    
    // Add the tag to the thought
    let mut updated_thought = thought.clone();
    updated_thought.add_tag(tag_id.clone());
    
    ui::with_loading_progress("Updating thought...", || {
        graph.command(&thoughtgraph::Command::PutThought {
            id: thought_id.clone(),
            thought: updated_thought,
        });
    });
    
    println!("Added tag '{}' to thought '{}'", tag.yellow(), id.green());
    Ok(())
}

/// Remove a tag from a thought
fn untag_thought(graph: &mut ThoughtGraph, id: &str, tag: &str) -> Result<()> {
    let thought_id = ThoughtID::new(id.to_string());
    let tag_id = TagID::new(tag.to_string());
    
    // Check if thought exists
    let thought = match graph.get_thought(&thought_id) {
        Some(t) => t.clone(),
        None => return Err(anyhow::anyhow!("Thought '{}' not found", id)),
    };
    
    // Check if the thought actually has this tag
    if !thought.tags.contains(&tag_id) {
        return Err(anyhow::anyhow!("Thought '{}' doesn't have tag '{}'", id, tag));
    }
    
    // Remove the tag from the thought
    let mut updated_thought = thought.clone();
    updated_thought.remove_tag(&tag_id);
    
    ui::with_loading_progress("Updating thought...", || {
        graph.command(&thoughtgraph::Command::PutThought {
            id: thought_id.clone(),
            thought: updated_thought,
        });
    });
    
    println!("Removed tag '{}' from thought '{}'", tag.yellow(), id.green());
    Ok(())
}

/// Add a reference from one thought to another
fn add_reference(graph: &mut ThoughtGraph, from: &str, to: &str, notes: Option<String>) -> Result<()> {
    let from_id = ThoughtID::new(from.to_string());
    let to_id = ThoughtID::new(to.to_string());
    
    // Check if both thoughts exist
    let from_thought = match graph.get_thought(&from_id) {
        Some(t) => t.clone(),
        None => return Err(anyhow::anyhow!("Thought '{}' not found", from)),
    };
    
    if !graph.thoughts.contains_key(&to_id) {
        return Err(anyhow::anyhow!("Thought '{}' not found", to));
    }
    
    // Create a new reference
    let reference = Reference::new(
        to_id,
        notes.unwrap_or_default(),
        Utc::now(),
    );
    
    // Add the reference to the thought
    let mut updated_thought = from_thought.clone();
    updated_thought.add_reference(reference);
    
    ui::with_loading_progress("Adding reference...", || {
        graph.command(&thoughtgraph::Command::PutThought {
            id: from_id.clone(),
            thought: updated_thought,
        });
    });
    
    println!("Added reference from '{}' to '{}'", from.green(), to.green());
    Ok(())
}

/// Search for thoughts matching a query
fn search_thoughts(graph: &ThoughtGraph, query_terms: &[String]) -> Result<()> {
    if query_terms.is_empty() {
        return Err(anyhow::anyhow!("Please provide search terms"));
    }
    
    let search_terms: Vec<String> = query_terms.iter()
        .map(|s| s.to_lowercase())
        .collect();
    
    println!("Searching for: {}", search_terms.join(" ").cyan());
    
    // Create a progress bar for the search operation
    let matching_thoughts = ui::with_loading_progress("Searching thoughts...", || {
        // Simple search in titles and contents
        graph.thoughts.iter()
            .filter(|(_, thought)| {
                let title_text = thought.title.clone().unwrap_or_default().to_lowercase();
                let content_text = thought.contents.to_lowercase();
                let combined_text = format!("{} {}", title_text, content_text);
                
                search_terms.iter().all(|term| combined_text.contains(term))
            })
            .collect::<Vec<(&ThoughtID, &Thought)>>()
    });
    
    if matching_thoughts.is_empty() {
        println!("No thoughts found matching query: {}", search_terms.join(" "));
        return Ok(());
    }
    
    println!("Found {} matching thoughts", matching_thoughts.len());
    
    // Display results with enhanced formatting
    ui::display_thought_list(graph, &matching_thoughts, MAX_DISPLAY_LENGTH)?;
    
    // If in interactive mode, allow selecting a thought to view
    if io::stdin().is_terminal() && !matching_thoughts.is_empty() {
        if ui::confirm("Would you like to view one of these thoughts?", true)? {
            let selected_id = ui::select_thought(graph, "Select a thought to view")?;
            
            if let Some(thought_id) = selected_id {
                return view_thought(graph, &thought_id.id);
            }
        }
    }
    
    Ok(())
}

/// List all available tags
fn list_tags(graph: &ThoughtGraph) -> Result<()> {
    let tags: Vec<(&TagID, &Tag)> = graph.tags.iter().collect();
    
    if tags.is_empty() {
        println!("{}", style("No tags found").italic());
        return Ok(());
    }
    
    // Display tags with enhanced formatting
    println!("{} {} {}",
        style(ui::format_column("TAG", 20)).bold().underlined(),
        style(ui::format_column("DESCRIPTION", 40)).bold().underlined(),
        style(ui::format_column("COUNT", 10)).bold().underlined()
    );
    
    // Pre-compute counts to avoid repeated iterations
    let counts = ui::with_loading_progress("Counting tag usage...", || {
        tags.iter().map(|(id, _)| {
            let count = graph.thoughts.values()
                .filter(|thought| thought.tags.contains(*id))
                .count();
            ((*id).clone(), count)
        }).collect::<HashMap<_, _>>()
    });

    for (id, tag) in &tags {
        let count = counts.get(id).unwrap_or(&0);
        
        println!("{} {} {}",
            style(ui::format_column(&format!("#{}", id.id), 20)).yellow(),
            style(ui::format_column(&tag.description, 40)),
            style(ui::format_column(&count.to_string(), 10))
        );
    }
    
    // If in interactive mode, offer to view thoughts with a specific tag
    if io::stdin().is_terminal() && !tags.is_empty() {
        if ui::confirm("Would you like to view thoughts with a specific tag?", false)? {
            let tag_items: Vec<String> = tags.iter()
                .map(|(id, tag)| format!("#{} - {}", id.id, tag.description))
                .collect();
            
            let selection = dialoguer::Select::with_theme(&ui::get_theme())
                .with_prompt("Select a tag")
                .default(0)
                .items(&tag_items)
                .interact_opt()?;
            
            if let Some(idx) = selection {
                let selected_tag = &tags[idx].0.id;
                return list_thoughts(graph, Some(selected_tag.clone()));
            }
        }
    }
    
    Ok(())
}

/// Visualize the thought graph
fn visualize_graph(
    graph: &ThoughtGraph,
    format: &str,
    focus: Option<String>,
    depth: usize,
    output: Option<PathBuf>,
) -> Result<()> {
    // If focus is not provided but we're in interactive mode, offer to select a focus
    let focus_id_str = if focus.is_none() && io::stdin().is_terminal() && !graph.thoughts.is_empty() {
        if ui::confirm("Would you like to focus on a specific thought?", true)? {
            match ui::select_thought(graph, "Select a thought to focus on")? {
                Some(id) => Some(id.id),
                None => None
            }
        } else {
            None
        }
    } else {
        focus
    };
    
    // Generate graph data with progress indicator
    let graph_data = ui::with_loading_progress("Generating graph visualization...", || {
        if let Some(focus_str) = &focus_id_str {
            let focus_id = ThoughtID::new(focus_str.clone());
            
            // Check if the focused thought exists
            if !graph.thoughts.contains_key(&focus_id) {
                return Err(anyhow::anyhow!("Thought '{}' not found", focus_str));
            }
            
            Ok(generate_focused_graph(graph, &focus_id, depth))
        } else {
            Ok(generate_graph_data(graph))
        }
    })?;
    
    // Generate output in the requested format
    let format = format.to_lowercase();
    let output_text = match format.as_str() {
        "dot" => graph_data.to_dot(),
        "json" => graph_data.to_json(),
        _ => return Err(anyhow::anyhow!("Unsupported visualization format: {}. Use 'dot' or 'json'.", format)),
    };
    
    // Output to file or stdout with progress indicator
    if let Some(output_path) = output {
        ui::with_loading_progress(&format!("Saving {} visualization to file...", format), || {
            fs::write(&output_path, &output_text)
        })?;
        
        println!("{}", style(format!("Visualization saved to {}", output_path.display())).green());
        
        // If it's a dot file, suggest using Graphviz
        if format == "dot" {
            println!("\nTip: To render this file with Graphviz, run:");
            println!("  dot -Tpng {} -o graph.png", output_path.display());
        }
    } else {
        println!("{}", output_text);
    }
    
    Ok(())
}

/// Edit text in an external editor
fn edit_in_external_editor(initial_content: &str, header_comment: &str) -> Result<String> {
    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    
    // Write initial content to the file
    if !header_comment.is_empty() {
        writeln!(temp_file, "{}", header_comment)?;
        writeln!(temp_file)?;
    }
    
    write!(temp_file, "{}", initial_content)?;
    temp_file.flush()?;
    
    // Get the editor command from environment or use a default
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    // Open the file in the editor
    let status = Command::new(&editor)
        .arg(temp_file.path())
        .status()
        .context(format!("Failed to open editor: {}", editor))?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with error status"));
    }
    
    // Read the updated content
    let content = fs::read_to_string(temp_file.path())?;
    
    // Filter out header comment
    let content = if !header_comment.is_empty() && content.starts_with(header_comment) {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 1 && lines[1].trim().is_empty() {
            lines[2..].join("\n")
        } else {
            lines[1..].join("\n")
        }
    } else {
        content
    };
    
    Ok(content)
}
