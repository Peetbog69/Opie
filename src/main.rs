use anyhow::Result;
use clap::{Parser, Subcommand};
use opie::inference::local::LocalInference;
use opie::inference::InferenceProvider;
use opie::{Agent, Config, Session, SessionStorage};
use std::io::{self, Write};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "opie")]
#[command(about = "Local-first AI assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session
    Chat {
        /// Load a saved session
        #[arg(short, long)]
        load: Option<String>,
    },
    
    /// Initialize config file
    Init,
    
    /// Show current configuration
    Config,
    
    /// Save current session
    Save {
        /// Session name
        name: String,
    },
    
    /// List saved sessions
    List,
    
    /// Delete a saved session
    Delete {
        /// Session name
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Init) => init_config(),
        Some(Commands::Config) => show_config(),
        Some(Commands::Chat { load }) => run_chat(load).await,
        None => run_chat(None).await,
        Some(Commands::List) => list_sessions(),
        Some(Commands::Save { name }) => save_current_session(&name),
        Some(Commands::Delete { name }) => delete_session(&name),
    }
}

fn init_config() -> Result<()> {
    let config = Config::load()?;
    config.save()?;
    
    println!("Config initialized at ~/.opie/config.toml");
    println!("\nTo use Opie, start llama-server first:");
    println!("  llama-server -m /path/to/model.gguf -c 8192");
    println!("\nRecommended models:");
    println!("  - Llama 3.2 3B Instruct (fast, 2GB)");
    println!("  - Qwen 2.5 7B Instruct (better quality, 4GB)");
    println!("\nOr install Ollama for easier model management:");
    
    Ok(())
}

fn show_config() -> Result<()> {
    let config = Config::load()?;
    println!("{}", toml::to_string_pretty(&config)?);
    Ok(())
}

async fn run_chat(load_session: Option<String>) -> Result<()> {
    println!("Loading configuration...");
    let config = Config::load()?;
    
    println!("Connecting to llama-server at {}...", config.server_url);
    let model = LocalInference::new(Some(config.server_url.clone()));
    
    // Health check
    model.health_check().await?;
    
    println!("Connected! (provider: {})\n", model.name());
    
    let storage = SessionStorage::new()?;
    let mut session = if let Some(name) = load_session {
        println!("Loading session '{}'...", name);
        storage.load(&name)?
    } else {
        Session::new()
    };
    
    let agent = Agent::new(Box::new(model));
    
    println!("Type 'quit', 'exit', or '/save <name>' to save and exit.\n");
    
    loop {
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }
        
        // Handle /save command
        if let Some(name) = input.strip_prefix("/save ") {
            let name = name.trim();
            if !name.is_empty() {
                storage.save(&session, name)?;
                println!("✓ Session saved as '{}'", name);
                break;
            } else {
                println!("Usage: /save <name>");
                continue;
            }
        }
        
        print!("Opie: ");
        io::stdout().flush()?;
        
        agent.run(&mut session, input).await?;
        
        println!(); // Extra newline for spacing
    }
    
    Ok(())
}

fn list_sessions() -> Result<()> {
    let storage = SessionStorage::new()?;
    let sessions = storage.list()?;
    
    if sessions.is_empty() {
        println!("No saved sessions found.");
    } else {
        println!("Saved sessions:");
        for session in sessions {
            println!("  - {}", session);
        }
        println!("\nLoad with: opie chat --load <name>");
    }
    
    Ok(())
}

fn save_current_session(_name: &str) -> Result<()> {
    println!("Error: This command must be used within a chat session.");
    println!("Use '/save <name>' during a chat, or 'opie chat --load <name>' to resume.");
    Ok(())
}

fn delete_session(name: &str) -> Result<()> {
    let storage = SessionStorage::new()?;
    
    if !storage.exists(name) {
        println!("Session '{}' not found.", name);
        return Ok(());
    }
    
    print!("Delete session '{}'? (y/n): ", name);
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() == "y" {
        storage.delete(name)?;
        println!("✓ Session '{}' deleted.", name);
    } else {
        println!("Cancelled.");
    }
    
    Ok(())
}
