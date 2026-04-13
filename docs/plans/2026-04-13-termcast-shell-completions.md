# TermCast Shell Completions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate shell completion scripts for bash, zsh, and fish that make TermCast feel like a first-class citizen in the terminal ecosystem.

**Architecture:** Use `clap`'s built-in completion generation (from `Cargo.toml` dependency) to auto-generate completions for all three shells. Distribute via:
1. Generated files committed to repo (`completions/bash/termcast.bash`, etc.)
2. Cargo release hook to regenerate on version bump

**Tech Stack:** clap, clap_complete (Rust crates)

---

## Background

TermCast already uses `clap` for CLI argument parsing (see `src/main.rs`):

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    location: Option<String>,
    #[arg(long)]
    ambient: bool,
    #[arg(long, default_value_t = 15)]
    cache_ttl: u64,
    #[arg(long)]
    install: Option<String>,
}
```

Clap has built-in completion generation via `clap_complete`. We just need to wire it up.

---

## Tasks

### Task 1: Add clap_complete to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Read current Cargo.toml**

Run: `cat Cargo.toml`
Expected: Current dependencies section visible

- [ ] **Step 2: Add clap_complete dependency**

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
clap_complete = "4.4"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
dirs = "5"
tempfile = "3.8"
```

- [ ] **Step 3: Verify build**

Run: `cargo check`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add clap_complete for shell completion generation"
```

---

### Task 2: Add completion generation subcommand

**Files:**
- Create: `src/completions.rs` (completion generation logic)
- Modify: `src/main.rs` (add generate-completions command)

- [ ] **Step 1: Create `src/completions.rs`**

```rust
//! Shell completion generation for TermCast.

use std::io;

use clap::Command;
use clap_complete::{generate, shells::*, Generator};

/// List of supported shells for completion generation.
pub const SUPPORTED_SHELLS: &[&str] = &["bash", "elvish", "fish", "powershell", "zsh"];

/// Generate completions for a specific shell.
pub fn generate_completions(shell: &str) -> Result<(), String> {
    let shell = shell.to_lowercase();

    let generator: Box<dyn Generator> = match shell.as_str() {
        "bash" => Box::new(Bash),
        "elvish" => Box::new(Elvish),
        "fish" => Box::new(Fish),
        "powershell" => Box::new(PowerShell),
        "zsh" => Box::new(Zsh),
        _ => {
            return Err(format!(
                "Unsupported shell '{}'. Supported: {}",
                shell,
                SUPPORTED_SHELLS.join(", ")
            ))
        }
    };

    let mut cmd = crate::build_cli();
    let bin_name = cmd.get_name().to_string();

    generate(*generator, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}

/// Build the CLI command structure for completion generation.
pub fn build_cli() -> Command {
    use clap::CommandFactory;

    crate::Args::command()
}
```

- [ ] **Step 2: Create completions directory**

```bash
mkdir -p completions
mkdir -p completions/bash
mkdir -p completions/zsh
mkdir -p completions/fish
mkdir -p completions/powershell
mkdir -p completions/elvish
```

- [ ] **Step 3: Add completion command to main.rs**

Add this to `src/main.rs`:

```rust
/// Generate shell completions for the specified shell.
#[derive(Parser, Debug)]
struct GenerateCompletions {
    /// Shell to generate completions for (bash, elvish, fish, powershell, zsh).
    #[arg(value_parser = crate::completions::validate_shell)]
    shell: String,

    /// Output file (default: stdout).
    #[arg(short, long)]
    output: Option<String>,
}

fn validate_shell(s: &str) -> Result<String, String> {
    let supported = &["bash", "elvish", "fish", "powershell", "zsh"];
    let s_lower = s.to_lowercase();
    if supported.contains(&s_lower.as_str()) {
        Ok(s_lower)
    } else {
        Err(format!(
            "Unsupported shell '{}'. Supported: {}",
            s,
            supported.join(", ")
        ))
    }
}
```

Actually, we need a cleaner approach. Let's create a separate binary or use a subcommand approach:

```rust
// Add this enum to Args
#[derive(Parser, Debug)]
enum Commands {
    /// Generate shell completions.
    #[command(name = "generate-completions")]
    GenerateCompletions {
        /// Shell to generate completions for.
        shell: String,
    },
}
```

But wait — we want to keep `termcast --location Oslo` working as-is. Let's instead add a hidden command:

```rust
// Add to src/main.rs after the existing Args struct

/// Subcommands for extended functionality.
#[derive(Parser, Debug)]
enum Commands {
    /// Generate shell completions for the specified shell.
    /// Run `termcast completions bash` to output bash completions.
    #[command(name = "completions", alias = "completion")]
    Completions {
        /// Shell to generate completions for (bash, elvish, fish, powershell, zsh).
        shell: String,
    },
}
```

Wait, that changes the user experience. Let's use a simpler approach with a separate binary target:

**Alternative: Create `termcast-completions` binary**

- Keep `termcast` unchanged for existing users
- Add `termcast-completions` binary that generates completions
- Both binaries share the same argument parsing via lib.rs

Actually, the cleanest approach is to add a `--generate-completions` hidden flag that clap can handle. But clap doesn't support this natively.

**Decision:** Use a separate subcommand `termcast completions <shell>`. This is the standard pattern (e.g., `rustup completions bash`).

- [ ] **Step 4: Modify `src/main.rs` to add completions subcommand**

Update the Args struct to include:
```rust
#[derive(Parser, Debug)]
enum Commands {
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for (bash, elvish, fish, powershell, zsh).
        shell: String,
    },
}
```

But we need to keep backward compatibility with `termcast --location`. Let's use clap's command grouping:

Actually, the simplest approach: modify the top-level Args to accept a subcommand variant.

```rust
#[derive(Parser, Debug)]
enum Cli {
    /// Generate shell completions.
    #[command(name = "completions")]
    Completions {
        /// Shell to generate completions for (bash, elvish, fish, powershell, zsh).
        shell: String,
    },
}

#[derive(Parser, Debug)]
struct Args {
    // Keep existing args for backward compatibility
    #[arg(short, long)]
    location: Option<String>,
    #[arg(long)]
    ambient: bool,
    #[arg(long, default_value_t = 15)]
    cache_ttl: u64,
    #[arg(long)]
    install: Option<String>,
    /// Subcommand (optional for backward compat)
    #[arg(hide = true)]
    command: Option<Cli>,
}
```

Then in main:
```rust
async fn run() -> Result<(), AppError> {
    let args = Args::parse();

    // Handle completions subcommand
    if let Some(Commands::Completions { shell }) = args.command {
        crate::completions::generate_completions(&shell)?;
        return Ok(());
    }

    // Rest of existing logic...
}
```

- [ ] **Step 5: Verify completion generation works**

Run: `cargo run -- completions bash`
Expected: Bash completion script printed to stdout

Run: `cargo run -- completions zsh`
Expected: Zsh completion script printed to stdout

Run: `cargo run -- completions fish`
Expected: Fish completion script printed to stdout

- [ ] **Step 6: Commit**

```bash
git add src/completions.rs src/main.rs
git commit -m "feat: add completions subcommand for shell integration"
```

---

### Task 3: Generate and commit pre-built completion files

**Files:**
- Create: `completions/bash/termcast.bash`
- Create: `completions/zsh/_termcast`
- Create: `completions/fish/termcast.fish`
- Create: `completions/powershell/_termcast.ps1`
- Create: `completions/elvish/termcast.elv`
- Create: `completions/README.md` (installation guide)

- [ ] **Step 1: Generate bash completions**

Run: `cargo run -- completions bash > completions/bash/termcast.bash`

- [ ] **Step 2: Generate zsh completions**

Run: `cargo run -- completions zsh > completions/zsh/_termcast`

- [ ] **Step 3: Generate fish completions**

Run: `cargo run -- completions fish > completions/fish/termcast.fish`

- [ ] **Step 4: Generate PowerShell completions**

Run: `cargo run -- completions powershell > completions/powershell/_termcast.ps1`

- [ ] **Step 5: Generate Elvish completions**

Run: `cargo run -- completions elvish > completions/elvish/termcast.elv`

- [ ] **Step 6: Create `completions/README.md`**

```markdown
# Shell Completions

Pre-generated shell completions for TermCast.

## Installation

### Bash

```bash
# User-wide (recommended)
mkdir -p ~/.local/share/bash-completion/completions
cp completions/bash/termcast.bash ~/.local/share/bash-completion/completions/

# Or copy to your bashrc directory
cp completions/bash/termcast.bash ~/.termcast.bash-completion
echo 'source ~/.termcast.bash-completion' >> ~/.bashrc
```

### Zsh

```bash
# Using oh-my-zsh plugins directory
cp completions/zsh/_termcast ~/.oh-my-zsh/custom/plugins/termcast/_termcast

# Add to .zshrc:
# plugins=(... termcast)
```

Or copy to fpath:
```bash
cp completions/zsh/_termcast ~/.zfunc/_termcast
autoload -Uz ~/.zfunc/_termcast
```

### Fish

```bash
# User-wide
mkdir -p ~/.config/fish/completions
cp completions/fish/termcast.fish ~/.config/fish/completions/
```

### PowerShell

```powershell
# Add to PowerShell profile
Copy-Item completions/powershell/_termcast.ps1 $HOME\Documents\PowerShell\Modules\termcast\termcast.psm1
Import-Module termcast
```

### Elvish

```bash
# Add to elvishrc
use ~/path/to/completions/elvish/termcast.elv
```

## Generating Fresh Completions

If you modify the CLI, regenerate completions:

```bash
cargo run -- completions bash > completions/bash/termcast.bash
cargo run -- completions zsh > completions/zsh/_termcast
cargo run -- completions fish > completions/fish/termcast.fish
cargo run -- completions powershell > completions/powershell/_termcast.ps1
cargo run -- completions elvish > completions/elvish/termcast.elv
```

## Supported Shells

- bash
- zsh
- fish
- powershell
- elvish
```

- [ ] **Step 7: Update existing `--install` output to reference completions**

The existing `--install bash` etc. should also point to these files:

```bash
# In install_shell_integration function, after printing the snippet:
println!();
println!("# For auto-completion, install from:");
println!("# completions/bash/termcast.bash");
```

- [ ] **Step 8: Commit all completions**

```bash
git add completions/
git add src/main.rs  # updated install output
git commit -m "feat: add pre-built shell completions for all supported shells"
```

---

### Task 4: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add completions section to README**

Add after "## Shell Integration" section:

```markdown
## Shell Completions

TermCast supports auto-completion for bash, zsh, fish, PowerShell, and Elvish.

### Generate Fresh Completions

```bash
./target/release/termcast completions bash    # Bash
./target/release/termcast completions zsh     # Zsh
./target/release/termcast completions fish    # Fish
./target/release/termcast completions powershell  # PowerShell
./target/release/termcast completions elvish  # Elvish
```

### Install Pre-built Completions

See [completions/README.md](completions/README.md) for shell-specific installation instructions.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add shell completions section to README"
```

---

## Verification Checklist

- [ ] `cargo build --release` compiles without warnings
- [ ] `./target/release/termcast completions bash` outputs valid bash completions
- [ ] `./target/release/termcast completions zsh` outputs valid zsh completions
- [ ] `./target/release/termcast completions fish` outputs valid fish completions
- [ ] Existing commands still work: `./target/release/termcast --location Oslo`
- [ ] Pre-built completion files exist and match `cargo run -- completions <shell>`
- [ ] README documents completion installation

---

## Not Doing (and Why)

- **Completion caching** — Users regenerate when they want, no need to cache
- **Interactive completion** — Static completion is sufficient for this CLI (no interactive subcommands)
- **Fish floating point temps** — Fish completions use integer values, which matches our output format
- **Fish adaptive completions based on location** — Overkill for v1; static completion covers the CLI surface

---

## Open Questions

1. Should we auto-install completions as part of `--install`? (Decided: No, keep `--install` focused on integration snippets, completions are separate)
2. Version sync between completion files and CLI? (Decided: Commit generated files, regenerate on version bump)