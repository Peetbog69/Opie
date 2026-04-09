use anyhow::Result;
use clap::{Parser, Subcommand};
use opie::inference::local::LocalInference;
use opie::inference::InferenceProvider;
use opie::{Agent, Config, Session};
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
    Chat,
    
    /// Initialize config file
    Init,
    
    /// Show current configuration
    Config,
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
        Some(Commands::Chat) | None => run_chat().await,
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

async fn run_chat() -> Result<()> {
    println!("Loading configuration...");
    let config = Config::load()?;
    
    println!("Connecting to llama-server at {}...", config.server_url);
    let model = LocalInference::new(Some(config.server_url.clone()));
    
    // Health check
    model.health_check().await?;
    
    println!("Connected! (provider: {})\n", model.name());
    
    let mut session = Session::new();
    let agent = Agent::new(Box::new(model));
    
    println!("Type 'quit' or 'exit' to end the session.\n");
    
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
        
        print!("Opie: ");
        io::stdout().flush()?;
        
        agent.run(&mut session, input).await?;
        
        println!(); // Extra newline for spacing
    }
    
    Ok(())
}
