# Creating a New Rust Project for the AIBundle Refactor

Let's set up a fresh Rust project and Git repository for your AIBundle refactoring project. These steps will create the complete project structure as outlined in our refactoring plan.

## Step 1: Create a New Rust Project

```bash
# Create a new Rust binary project
cargo new aibundle-refactor
cd aibundle-refactor
```

## Step 2: Initialize Git Repository

```bash
# Initialize Git repository (Cargo already does this, but just to be explicit)
git init

# Create a .gitignore file for Rust projects
cat > .gitignore << EOF
/target
**/*.rs.bk
Cargo.lock
.DS_Store
.idea/
.vscode/
*.swp
*.swo
EOF
```

## Step 3: Configure Dependencies in Cargo.toml

Edit the Cargo.toml file to include all the dependencies from the original project:

```bash
# Open the Cargo.toml file in your preferred editor
# For example:
nano Cargo.toml
# or
code Cargo.toml
```

Replace the contents with:

```toml
[package]
name = "aibundle"
version = "0.6.14"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A CLI & TUI file aggregator and formatter"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
crossterm = "0.27"
ratatui = "0.24"
walkdir = "2.4"
glob = "0.3"
ignore = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
itertools = "0.11"
regex = "1.9"
papaparse = "0.1"  # If you need this, otherwise remove
# Add any other dependencies needed for your project
```

## Step 4: Create the Project Directory Structure

```bash
# Create the module directories
mkdir -p src/{cli,tui,config,fs,output,clipboard,models,utils}

# Create subdirectories for TUI module (for enhanced modularization)
mkdir -p src/tui/{components,state,handlers,views}
```

## Step 5: Create Basic Placeholder Files

```bash
# Create module entry points
for dir in cli tui config fs output clipboard models utils; do
  touch src/${dir}/mod.rs
done

# Create TUI submodule entry points
for dir in components state handlers views; do
  touch src/tui/${dir}/mod.rs
done

# Create main.rs with a basic skeleton
cat > src/main.rs << EOF
mod cli;
mod tui;
mod config;
mod fs;
mod output;
mod clipboard;
mod models;
mod utils;

fn main() {
    println!("AIBundle Refactoring Project - Initial Setup");
    // Actual code will be implemented according to the refactoring plan
}
EOF
```

## Step 6: Set Up Models Module First

Since other modules will depend on common types and constants:

```bash
# Create model files
touch src/models/{app_config.rs,enums.rs,constants.rs}

# Add basic exports to models/mod.rs
cat > src/models/mod.rs << EOF
mod app_config;
mod enums;
mod constants;

pub use app_config::*;
pub use enums::*;
pub use constants::*;
EOF

# Add version and constants placeholders
cat > src/models/constants.rs << EOF
pub const VERSION: &str = "0.6.14";
pub const DEFAULT_SELECTION_LIMIT: usize = 400;

// Placeholder for other constants to be implemented
// according to refactoring plan
EOF
```

## Step 7: Add Basic Implementation Placeholders to FS Module 

Since this is an existing module, let's set up a basic placeholder:

```bash
cat > src/fs/mod.rs << EOF
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn confirm_overwrite(file_path: &str) -> io::Result<bool> {
    if Path::new(file_path).exists() {
        println!("File '{}' already exists. Overwrite? (y/n): ", file_path);
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        return Ok(input.trim().eq_ignore_ascii_case("y"));
    }
    Ok(true)
}

pub fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_excluded(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_excluded(path: &std::path::Path) -> bool {
    let excluded = ["node_modules", ".git", "target"];
    path.components()
        .any(|c| excluded.contains(&c.as_os_str().to_str().unwrap_or("")))
}

// Additional functions will be implemented 
// according to the refactoring plan
EOF
```

## Step 8: Make Your First Commit

```bash
# Add all files to Git
git add .

# Make the initial commit
git commit -m "Initial project setup for AIBundle refactoring"
```

## Step 9: Implement Refactoring in Stages

Now you can follow the refactoring plan, stage by stage:

```bash
# For each stage, create a new branch
git checkout -b stage1-basic-module-structure

# Implement Stage 1 changes...

# Commit Stage 1 changes
git add .
git commit -m "Stage 1: Create basic module structure"

# Continue for subsequent stages
git checkout -b stage2-extract-constants-and-types
# ... and so on
```

## Step 10: Testing the Refactored Code

After each stage, verify the code compiles and (when applicable) runs correctly:

```bash
# Check that the code compiles
cargo check

# Build the project
cargo build

# Run the project (once it's functional)
cargo run
```

## Recommended Workflow for Implementing Each Stage

For each stage in the refactoring plan:

1. Create a new branch from the previous stage's branch:
   ```bash
   git checkout -b stage<N>-<description>
   ```

2. Implement the changes according to the refactoring plan for that stage

3. Compile and test:
   ```bash
   cargo check
   cargo test  # When you have tests
   ```

4. Commit your changes:
   ```bash
   git add .
   git commit -m "Stage <N>: <description>"
   ```

5. After completing all stages, you can merge everything back to main:
   ```bash
   git checkout main
   git merge stage<final>-<description>
   ```

## Additional Tips

1. **Cargo Add**: If you have `cargo-edit` installed, you can add dependencies more easily:
   ```bash
   cargo add clap --features derive
   cargo add crossterm ratatui walkdir glob ignore
   # etc.
   ```

2. **Documentation**: Add rustdoc comments as you implement:
   ```rust
   /// Brief description of function
   /// 
   /// # Examples
   /// 
   /// ```
   /// let result = module::function();
   /// assert_eq!(result, expected_value);
   /// ```
   pub fn function() -> ReturnType { /* ... */ }
   ```

3. **Unit Tests**: Add tests as you implement each module:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function() {
           // Test implementation
       }
   }
   ```

This setup provides a structured approach to implement the refactoring plan systematically while maintaining a clean Git history that tracks each stage of the refactoring process.

Code Output Complete.