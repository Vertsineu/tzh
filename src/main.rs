use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Read};

mod config;
mod translator;

use config::Config;
use translator::Translator;

#[derive(Parser)]
#[command(name = "tzh")]
#[command(about = "AI-powered translation tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate text
    Translate {
        /// Text to translate (multiple words will be joined with spaces). If no text is provided, reads from stdin.
        text: Vec<String>,
        /// Target language (e.g., zh, en, ja, ko, fr, de, es)
        #[arg(short, long, default_value = "zh")]
        to: String,
        /// Source language (auto-detect if not specified)
        #[arg(short, long)]
        from: Option<String>,
        /// Plain output (only show translation result, no formatting)
        #[arg(short, long)]
        plain: bool,
    },
    /// Configure the translator
    Config {
        /// Set API endpoint
        #[arg(long)]
        endpoint: Option<String>,
        /// Set model name
        #[arg(long)]
        model: Option<String>,
        /// Set API key
        #[arg(long)]
        api_key: Option<String>,
        /// Set temperature (0.0 to 2.0)
        #[arg(long)]
        temperature: Option<f32>,
        /// Set max tokens (None for unlimited)
        #[arg(long)]
        max_tokens: Option<i32>,
    },
    /// Show current configuration
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Translate {
            text,
            to,
            from,
            plain,
        } => {
            let translator = Translator::new(&config);

            // Get the text to translate either from arguments or stdin
            let input_text = if text.is_empty() {
                // Read from stdin if no text arguments provided
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            } else {
                // Join all text arguments with spaces
                text.join(" ")
            };

            if input_text.is_empty() {
                eprintln!("{}", "No text provided to translate".red());
                return Ok(());
            }

            if !plain {
                println!("{}", "Translating...".blue());
            }

            match translator
                .translate(&input_text, &to, from.as_deref())
                .await
            {
                Ok(result) => {
                    if plain {
                        // Plain output: only show the translation result
                        println!("{}", result);
                    } else {
                        // Formatted output: show original and translation with colors
                        println!("\n{}", "Original:".green().bold());
                        println!("{}", input_text);
                        println!("\n{}", format!("Translation ({}):", to).green().bold());
                        println!("{}", result.bright_white());
                    }
                }
                Err(e) => {
                    // In plain mode, output error to stderr without formatting
                    eprintln!("Translation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Config {
            endpoint,
            api_key,
            model,
            temperature,
            max_tokens,
        } => {
            if let Some(endpoint) = endpoint {
                config.set_endpoint(&endpoint);
                println!("{} {}", "Endpoint set to:".green(), endpoint);
            }

            if let Some(model) = model {
                config.set_model(&model);
                println!("{} {}", "Model set to:".green(), model);
            }

            if let Some(api_key) = api_key {
                config.set_api_key(&api_key);
                println!("{}", "API key updated".green());
            }

            if let Some(temperature) = temperature {
                config.set_temperature(temperature);
                println!("{} {}", "Temperature set to:".green(), temperature);
            }

            if let Some(max_tokens) = max_tokens {
                config.set_max_tokens(Some(max_tokens));
                println!("{} {}", "Max tokens set to:".green(), max_tokens);
            }

            config.save()?;
        }
        Commands::Status => {
            println!("{}", "Current Configuration:".blue().bold());
            println!("Endpoint: {}", config.endpoint());
            println!("Model: {}", config.model());
            println!("Temperature: {}", config.temperature());
            println!(
                "Max tokens: {}",
                config
                    .max_tokens()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "Unlimited".to_string())
            );
            println!(
                "API key: {}",
                if config.has_api_key() {
                    "Set".green()
                } else {
                    "Not set".red()
                }
            );
        }
    }

    Ok(())
}
