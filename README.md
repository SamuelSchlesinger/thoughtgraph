# ThoughtGraph: A Tool for Personal Knowledge Management

ThoughtGraph is a powerful command-line tool for personal journaling, note-taking, and knowledge management. It helps you capture, organize, and connect your thoughts in a flexible graph structure with a focus on simplicity, privacy, and developer-friendly workflows.

## Features

- **Thought Capture**: Quickly record ideas, notes, and journal entries
- **Bidirectional Links**: Create connections between related thoughts
- **Tagging System**: Organize thoughts with customizable tags
- **Flexible Search**: Find thoughts using powerful search capabilities
- **Command-line Interface**: Fast and efficient text-based workflow
- **Local Storage**: All your data stays on your machine
- **Plain Text Workflow**: Use your favorite text editor to write entries
- **Lightweight**: Minimal resource usage with a small binary

## Installation

```bash
# Clone the repository
git clone https://github.com/your-username/thoughtgraph.git

# Build the project
cd thoughtgraph
cargo build --release

# Optional: add to your PATH
cp target/release/thoughts /usr/local/bin/
```

## Getting Started

### Initialize a New Thought Graph

```bash
# Create a new thought graph in the default location
thoughts init

# Or specify a custom location
thoughts -f my_thoughts.bin init
```

### Creating Thoughts

```bash
# Create a new thought with an interactive prompt
thoughts create

# Create with command-line arguments
thoughts create --id daily-journal-2025-02-26 --title "Daily Journal" --content "Today I learned about ThoughtGraph..." --tag journal --tag daily
```

When creating a thought without the `--content` parameter, ThoughtGraph will open your default text editor (set by the `EDITOR` environment variable).

### Viewing and Managing Thoughts

```bash
# List all thoughts
thoughts list

# List thoughts with a specific tag
thoughts list --tag journal

# View a specific thought
thoughts view daily-journal-2025-02-26

# Edit a thought
thoughts edit daily-journal-2025-02-26

# Delete a thought (with confirmation prompt)
thoughts delete daily-journal-2025-02-26

# Force delete without confirmation
thoughts delete daily-journal-2025-02-26 --force
```

### Using Tags

```bash
# List all tags
thoughts tags

# Add a tag to a thought
thoughts tag daily-journal-2025-02-26 important --description "High priority items"

# Remove a tag
thoughts untag daily-journal-2025-02-26 important
```

### Creating Connections

```bash
# Create a reference from one thought to another
thoughts reference daily-journal-2025-02-26 project-idea-xyz --notes "Daily journal entry mentions this project"
```

When you view a thought with `thoughts view`, ThoughtGraph will display both outgoing references (thoughts you link to) and incoming references (thoughts that link to this one).

### Searching

```bash
# Search for thoughts containing specific terms
thoughts search journal project meeting
```

## Journaling Tips

ThoughtGraph is perfect for personal journaling. Here are some tips to make the most of it:

### 1. Create a Daily Journal Structure

Establish a consistent ID format for daily entries:
```bash
thoughts create --id journal-YYYY-MM-DD --title "Journal for YYYY-MM-DD" --tag journal --tag daily
```

### 2. Use Tags for Organization

- Create tags for different areas of life: `work`, `personal`, `health`, `learning`
- Tag emotional states: `happy`, `stressed`, `inspired`
- Tag entry types: `reflection`, `gratitude`, `goal-setting`
- Create project-specific tags for all related entries

### 3. Link Related Thoughts

Connect your daily journal entries to ongoing projects, goals, or ideas:
```bash
# First create the project thought
thoughts create --id project-learn-rust --title "Learning Rust Programming" --tag project --tag learning

# Then link your daily journal to it
thoughts reference journal-2025-02-26 project-learn-rust --notes "Worked on Rust exercises today"
```

### 4. Create Topic-Specific Thoughts

For recurring themes, create dedicated thoughts:
```bash
thoughts create --id book-atomic-habits --title "Book Notes: Atomic Habits" --tag book --tag productivity
```

Then reference these in your daily journals when relevant.

### 5. Establish Regular Review Patterns

- **Daily**: Tag important items with `review`
- **Weekly**: Create a weekly summary thought that references key daily entries
- **Monthly/Quarterly**: Search for patterns across multiple entries

Example weekly review process:
```bash
# Create weekly review entry
thoughts create --id review-2025-W08 --title "Weekly Review: Feb 24-Mar 2, 2025" --tag review --tag weekly

# Link it to relevant daily entries
thoughts reference review-2025-W08 journal-2025-02-24
thoughts reference review-2025-W08 journal-2025-02-25
# ...and so on
```

### 6. Use Search to Find Patterns

```bash
# Find all entries mentioning a particular topic
thoughts search meditation mindfulness

# Find entries with specific tag combinations
thoughts list --tag journal --tag important
```

### 7. Create a Book Notes System

For books you're reading:
```bash
# Create a main book entry
thoughts create --id book-pragmatic-programmer --title "Book: The Pragmatic Programmer" --tag book

# Create chapter notes with references to the main book entry
thoughts create --id book-pragmatic-programmer-ch1 --title "Ch 1: A Pragmatic Philosophy" --tag book --tag chapter
thoughts reference book-pragmatic-programmer-ch1 book-pragmatic-programmer
```

## Advanced Usage

### Creating a Template System

You can create shell aliases or scripts to standardize journal entries:

```bash
# Add to your .bashrc or .zshrc
alias journal-today='thoughts create --id journal-$(date +%Y-%m-%d) --title "Journal for $(date +%Y-%m-%d)" --tag journal --tag daily'
alias weekly-review='thoughts create --id review-$(date +%Y)-W$(date +%V) --title "Weekly Review: $(date +%b\ %d-%d,\ %Y)" --tag review --tag weekly'
```

### Backup Your Thoughts

Regularly back up your thought graph:

```bash
# If using default location
cp "$(find ~/.local/share/thoughtgraph -name thoughts.bin)" ~/backups/thoughts-$(date +%Y-%m-%d).bin

# If using custom location
cp my_thoughts.bin ~/backups/thoughts-$(date +%Y-%m-%d).bin
```

### Setting Up Automated Backup

Create a cron job to back up your thoughts regularly:

```bash
# Add to crontab (crontab -e)
# Backup every day at 11pm
0 23 * * * cp ~/.local/share/thoughtgraph/thoughts.bin ~/backups/thoughts-$(date +\%Y-\%m-\%d).bin
```

### Integration with Git

Maintain a version history of your thought graph:

```bash
# Initialize a git repository for your backups
mkdir -p ~/thought-backups && cd ~/thought-backups
git init

# Create a backup script (~/bin/backup-thoughts.sh)
#!/bin/bash
BACKUP_DIR=~/thought-backups
BACKUP_FILE="thoughts-$(date +%Y-%m-%d).bin"
cp ~/.local/share/thoughtgraph/thoughts.bin "$BACKUP_DIR/$BACKUP_FILE"
cd "$BACKUP_DIR"
git add "$BACKUP_FILE"
git commit -m "Backup thoughts for $(date +%Y-%m-%d)"
```

## Comparison with Other Tools

ThoughtGraph excels in specific use cases but may not be the best fit for every scenario. Here's how it compares to other popular knowledge management tools:

### ThoughtGraph vs. Obsidian

**Choose ThoughtGraph when:**
- You prefer command-line interfaces and terminal workflows
- You want minimal resource usage
- You need a simple, lightweight solution
- You work primarily in plain text with your preferred editor
- You want a tool that integrates well with scripts and automation

**Choose Obsidian when:**
- You prefer a graphical interface with visual graph display
- You need rich text formatting (headers, lists, tables, etc.)
- You want plugins for extended functionality
- You need support for embedding images, PDFs, or other media
- You prefer working with Markdown files directly

### ThoughtGraph vs. Logseq

**Choose ThoughtGraph when:**
- You prefer a traditional document-based approach over outlining
- You want a simpler learning curve
- You need a faster, more lightweight tool
- You prefer direct control through a CLI

**Choose Logseq when:**
- You prefer an outliner-style workflow
- You want a built-in daily journaling system
- You need block-level references and queries
- You want a visual graph view
- You prefer working with Markdown/Org-mode files directly

### ThoughtGraph vs. Org-mode

**Choose ThoughtGraph when:**
- You don't use Emacs
- You want a more focused tool just for note connections
- You need a simpler tagging system

**Choose Org-mode when:**
- You're already an Emacs user
- You want the ultimate customization and flexibility
- You need combined task/project management and notes
- You prefer a fully keyboard-driven workflow
- You want a mature ecosystem with many extensions

### ThoughtGraph vs. Notion

**Choose ThoughtGraph when:**
- You want complete data privacy and local storage
- You prefer working in the terminal
- You need a lightweight, fast tool
- You want to avoid subscription fees

**Choose Notion when:**
- You need collaboration features
- You want databases, tables, and rich layouts
- You need web access from multiple devices
- You prefer a visual drag-and-drop interface
- You want integrated task/project management

## Use Cases

ThoughtGraph is particularly well-suited for:

1. **Developer journals**: Track coding progress, bugs, and solutions
2. **Research notes**: Capture findings with interconnected references
3. **Learning journals**: Document your learning path with linked concepts
4. **Personal knowledge base**: Build a network of interconnected ideas
5. **Project documentation**: Create references between related project notes
6. **Reading notes**: Track insights from books and articles with connections

## Customization

### Using Different Editors

ThoughtGraph uses your system's default editor, but you can override it:

```bash
# Set for one session
export EDITOR=vim
thoughts create

# Set permanently in your shell profile (.bashrc, .zshrc, etc.)
export EDITOR=nano
```

### Custom Storage Location

```bash
# Set a custom storage location
thoughts -f ~/Dropbox/thoughts.bin create --id new-thought
```

## Troubleshooting

### Common Issues

1. **Editor doesn't open**: Ensure your `EDITOR` environment variable is set correctly
2. **Command not found**: Make sure the `thoughts` binary is in your PATH 
3. **Permission denied**: Check file permissions on your thoughts.bin file

## License

[License information]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

ThoughtGraph was inspired by various knowledge management systems including Zettelkasten, wiki-linking tools, and graph-based note applications.