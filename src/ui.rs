//! User interface utilities for ThoughtGraph
//! 
//! This module provides enhanced UI components for the command-line interface,
//! including interactive menus, progress indicators, and improved text rendering.

use anyhow::Result;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::{Tag, TagID, Thought, ThoughtGraph, ThoughtID};

/// Format a string with the given width for display
pub fn format_column(text: &str, width: usize) -> String {
    format!("{:<width$}", text, width = width)
}

/// UI Theme to use consistently throughout the application
pub fn get_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Display a multi-select menu to choose tags from existing tags
pub fn select_tags(graph: &ThoughtGraph, initial_selection: &[TagID]) -> Result<Vec<TagID>> {
    let tags: Vec<(&TagID, &Tag)> = graph.tags.iter().collect();
    
    if tags.is_empty() {
        return Ok(Vec::new());
    }
    
    let items: Vec<String> = tags
        .iter()
        .map(|(id, tag)| format!("{} - {}", id.id, tag.description))
        .collect();
    
    let initial_indices: Vec<usize> = initial_selection
        .iter()
        .filter_map(|sel_id| {
            tags.iter()
                .position(|(id, _)| *id == sel_id)
        })
        .collect();
    
    let selection = MultiSelect::with_theme(&get_theme())
        .with_prompt("Select tags (space to select, enter to confirm)")
        .items(&items)
        .defaults(&initial_indices.iter().map(|i| *i > 0).collect::<Vec<_>>())
        .interact()?;
    
    let selected_tags = selection
        .into_iter()
        .map(|i| tags[i].0.clone())
        .collect();
    
    Ok(selected_tags)
}

/// Interactive thought selection with fuzzy search
pub fn select_thought(graph: &ThoughtGraph, prompt: &str) -> Result<Option<ThoughtID>> {
    let thoughts: Vec<(&ThoughtID, &Thought)> = graph.thoughts.iter().collect();
    
    if thoughts.is_empty() {
        return Ok(None);
    }
    
    let items: Vec<String> = thoughts
        .iter()
        .map(|(id, thought)| {
            let title = thought.title.as_deref().unwrap_or("(Untitled)");
            format!("{} - {}", id.id, title)
        })
        .collect();
    
    let selection = FuzzySelect::with_theme(&get_theme())
        .with_prompt(prompt)
        .default(0)
        .items(&items)
        .interact_opt()?;
    
    Ok(selection.map(|i| thoughts[i].0.clone()))
}

/// Display a progress bar while loading a thought graph
pub fn with_loading_progress<F, T>(message: &str, operation: F) -> T
where
    F: FnOnce() -> T,
{
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    
    let result = operation();
    
    pb.finish_with_message(format!("{} Done!", message));
    result
}

/// Interactive thought browser that allows exploring references
pub fn browse_thoughts(graph: &ThoughtGraph) -> Result<()> {
    let term = Term::stdout();
    let mut current_id: Option<ThoughtID> = None;
    
    loop {
        term.clear_screen()?;
        
        if let Some(id) = &current_id {
            // Display current thought
            if let Some(thought) = graph.get_thought(id) {
                display_thought_details(graph, id, thought)?;
                
                println!("\n{}", style("Actions:").bold());
                let actions = &[
                    "View references",
                    "View backlinks",
                    "Select another thought",
                    "Back to main menu"
                ];
                
                let selection = Select::with_theme(&get_theme())
                    .items(actions)
                    .default(0)
                    .interact()?;
                
                match selection {
                    0 => {
                        // View references
                        if thought.references.is_empty() {
                            println!("No references to display.");
                            term.read_key()?;
                            continue;
                        }
                        
                        let references: Vec<String> = thought.references
                            .iter()
                            .map(|r| {
                                let title = graph.get_thought(&r.id)
                                    .and_then(|t| t.title.clone())
                                    .unwrap_or_else(|| "(Untitled)".to_string());
                                format!("{} - {}", r.id.id, title)
                            })
                            .collect();
                        
                        let ref_selection = Select::with_theme(&get_theme())
                            .with_prompt("Select a reference to view")
                            .items(&references)
                            .default(0)
                            .interact_opt()?;
                        
                        if let Some(index) = ref_selection {
                            current_id = Some(thought.references[index].id.clone());
                        }
                    },
                    1 => {
                        // View backlinks
                        let backlinks = graph.get_backlinks(id);
                        
                        if backlinks.is_empty() {
                            println!("No backlinks to display.");
                            term.read_key()?;
                            continue;
                        }
                        
                        let backlink_items: Vec<String> = backlinks
                            .iter()
                            .map(|link_id| {
                                let title = graph.get_thought(link_id)
                                    .and_then(|t| t.title.clone())
                                    .unwrap_or_else(|| "(Untitled)".to_string());
                                format!("{} - {}", link_id.id, title)
                            })
                            .collect();
                        
                        let backlink_selection = Select::with_theme(&get_theme())
                            .with_prompt("Select a backlink to view")
                            .items(&backlink_items)
                            .default(0)
                            .interact_opt()?;
                        
                        if let Some(index) = backlink_selection {
                            current_id = Some(backlinks[index].clone());
                        }
                    },
                    2 => {
                        // Select another thought
                        current_id = select_thought(graph, "Select a thought to view")?;
                        if current_id.is_none() {
                            return Ok(());
                        }
                    },
                    3 | _ => return Ok(()),
                }
            } else {
                println!("Thought not found.");
                term.read_key()?;
                current_id = None;
            }
        } else {
            // No thought selected yet
            current_id = select_thought(graph, "Select a thought to view")?;
            if current_id.is_none() {
                return Ok(());
            }
        }
    }
}

/// Display the details of a thought with enhanced formatting
pub fn display_thought_details(graph: &ThoughtGraph, id: &ThoughtID, thought: &Thought) -> Result<()> {
    // Display title
    if let Some(title) = &thought.title {
        println!("{}", style(title).bold().green());
    } else {
        println!("{}", style("(Untitled)").bold());
    }
    
    println!("ID: {}", style(&id.id).blue());
    
    // Display metadata
    println!("Created: {}", style(thought.created_at.format("%Y-%m-%d %H:%M:%S")).dim());
    println!("Updated: {}", style(thought.updated_at.format("%Y-%m-%d %H:%M:%S")).dim());
    
    // Display tags
    if !thought.tags.is_empty() {
        println!("\n{}", style("Tags:").bold());
        for tag_id in &thought.tags {
            let tag_desc = graph.get_tag(tag_id)
                .map(|tag| format!(" - {}", tag.description))
                .unwrap_or_default();
            println!("  {} {}", style(format!("#{}", tag_id.id)).yellow(), style(tag_desc).dim());
        }
    }
    
    // Display references
    if !thought.references.is_empty() {
        println!("\n{}", style("References:").bold());
        for reference in &thought.references {
            let ref_id = &reference.id;
            let title = graph.get_thought(ref_id)
                .and_then(|t| t.title.clone())
                .unwrap_or_else(|| "(Untitled)".to_string());
            
            println!("  → {} {}", style(&ref_id.id).blue(), title);
            if !reference.notes.is_empty() {
                println!("    {}", style(&reference.notes).dim());
            }
        }
    }
    
    // Display backlinks
    let backlinks = graph.get_backlinks(id);
    if !backlinks.is_empty() {
        println!("\n{}", style("Referenced by:").bold());
        for backlink in backlinks {
            let title = graph.get_thought(&backlink)
                .and_then(|t| t.title.clone())
                .unwrap_or_else(|| "(Untitled)".to_string());
            
            println!("  ← {} {}", style(&backlink.id).blue(), title);
        }
    }
    
    // Display content
    println!("\n{}", style("═".repeat(80)).dim());
    println!("{}", thought.contents);
    println!("{}", style("═".repeat(80)).dim());
    
    Ok(())
}

/// Display a list of thoughts with enhanced formatting
pub fn display_thought_list(_graph: &ThoughtGraph, thoughts: &[(&ThoughtID, &Thought)], max_display_length: usize) -> Result<()> {
    if thoughts.is_empty() {
        println!("{}", style("No thoughts found").italic());
        return Ok(());
    }

    // Create a table-like display
    println!("{} {} {}",
        style(format_column("ID", 20)).bold().underlined(),
        style(format_column("TITLE", 30)).bold().underlined(),
        style(format_column("UPDATED", 20)).bold().underlined()
    );
    
    for (id, thought) in thoughts {
        let title = thought.title.as_deref().unwrap_or("(Untitled)");
        let date = thought.updated_at.format("%Y-%m-%d %H:%M");
        
        println!("{} {} {}",
            style(format_column(&id.id, 20)).blue(),
            style(format_column(title, 30)),
            style(format_column(&date.to_string(), 20)).dim()
        );
        
        // Print truncated content
        let preview = if thought.contents.len() > max_display_length {
            format!("{}...", &thought.contents[..max_display_length])
        } else {
            thought.contents.clone()
        };
        println!("  {}", style(preview).dim());
        
        // Print tags
        if !thought.tags.is_empty() {
            let tag_list: Vec<String> = thought.tags.iter()
                .map(|t| format!("#{}", t.id))
                .collect();
            println!("  {}", style(tag_list.join(" ")).yellow());
        }
        
        println!();
    }
    
    Ok(())
}

/// Interactive tag selection or creation
pub fn tag_selector(graph: &ThoughtGraph) -> Result<(TagID, Option<String>)> {
    let existing_tags: Vec<String> = graph.tags.keys()
        .map(|tag| tag.id.clone())
        .collect();
    
    let options = vec!["Select existing tag", "Create new tag"];
    let selection = Select::with_theme(&get_theme())
        .with_prompt("What would you like to do?")
        .default(0)
        .items(&options)
        .interact()?;
    
    match selection {
        0 => {
            // Select existing tag
            if existing_tags.is_empty() {
                println!("No existing tags found. Creating a new tag instead.");
                let tag_id = Input::<String>::with_theme(&get_theme())
                    .with_prompt("Enter a new tag ID")
                    .interact()?;
                
                let description = Input::<String>::with_theme(&get_theme())
                    .with_prompt("Enter a description for the tag")
                    .interact()?;
                
                Ok((TagID::new(tag_id), Some(description)))
            } else {
                let tag_selection = Select::with_theme(&get_theme())
                    .with_prompt("Select a tag")
                    .default(0)
                    .items(&existing_tags)
                    .interact()?;
                
                Ok((TagID::new(existing_tags[tag_selection].clone()), None))
            }
        },
        1 | _ => {
            // Create new tag
            let tag_id = Input::<String>::with_theme(&get_theme())
                .with_prompt("Enter a new tag ID")
                .interact()?;
            
            let description = Input::<String>::with_theme(&get_theme())
                .with_prompt("Enter a description for the tag")
                .interact()?;
            
            Ok((TagID::new(tag_id), Some(description)))
        }
    }
}

/// Command selector for the main menu
pub fn command_selector() -> Result<usize> {
    let commands = vec![
        "Create a new thought",
        "List thoughts",
        "View thought details",
        "Edit a thought",
        "Delete a thought",
        "Tag a thought",
        "Untag a thought",
        "Add a reference between thoughts",
        "Search thoughts",
        "Browse thoughts interactively",
        "List all tags",
        "Visualize thought graph",
        "Exit"
    ];
    
    let selection = Select::with_theme(&get_theme())
        .with_prompt("What would you like to do?")
        .default(0)
        .items(&commands)
        .interact()?;
    
    Ok(selection)
}

/// Confirmation dialog with enhanced styling
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&get_theme())
        .with_prompt(message)
        .default(default)
        .interact()?)
}

/// Extract the first few words of a string for use as a title suggestion
pub fn suggest_title_from_content(content: &str) -> String {
    content
        .split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
}
