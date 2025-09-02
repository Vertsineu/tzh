use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, BufRead, BufReader, Read, Write};

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
    #[command(alias = "t")]
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
        /// Translate line by line for streaming output
        #[arg(short, long)]
        stream: bool,
    },
    /// Interactive translation mode (translate each line as you type)
    #[command(alias = "i")]
    Interactive {
        /// Target language (e.g., zh, en, ja, ko, fr, de, es)
        #[arg(short, long, default_value = "zh")]
        to: String,
        /// Source language (auto-detect if not specified)
        #[arg(short, long)]
        from: Option<String>,
    },
    /// Configure the translator
    #[command(alias = "c")]
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
    #[command(alias = "s")]
    Status,
}

fn has_blank(text: &str) -> bool {
    text.as_bytes().iter().any(|&b| b.is_ascii_whitespace())
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
            stream,
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

            // Create callback for translation results
            let callback = |original: &str, translation: &str| {
                if translation.is_empty() {
                    println!(); // Empty line
                    return;
                }

                if plain {
                    // Plain mode: just output the translation
                    println!("{}", translation);
                } else {
                    println!(); // Add separator between lines
                    println!("{}", "Original:".green().bold());
                    println!("{}", original);
                    println!("{}", format!("Translation ({}):", to).green().bold());
                    println!("{}", translation.bright_white());
                }
            };

            // Check whether is a word or phrase
            if has_blank(&input_text) {
                // Split input text into lines if streaming
                let lines: Vec<&str> = if stream {
                    input_text.lines().map(|line| line.trim()).collect()
                } else {
                    vec![input_text.trim()]
                };

                for line in lines {
                    // Translate each line
                    match translator
                        .translate_line(line, &to, from.as_deref(), &callback)
                        .await
                    {
                        Ok(()) => { /* Nothing to do, because callback has done everything */ }
                        Err(e) => {
                            eprintln!("Translation failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                // Translate single word
                match translator
                    .translate_word(&input_text, &to, from.as_deref(), callback)
                    .await
                {
                    Ok(()) => { /* Nothing to do, because callback has done everything */ }
                    Err(e) => {
                        eprintln!("Translation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Interactive { to, from } => {
            let translator = Translator::new(&config);

            println!(
                "{}",
                "Interactive translation mode (Ctrl+C to exit)"
                    .blue()
                    .bold()
            );
            println!("{} {}", "Target language:".green(), to);
            if let Some(ref from_lang) = from {
                println!("{} {}", "Source language:".green(), from_lang);
            } else {
                println!("{}", "Source language: auto-detect".green());
            }
            print!("{} ", "tzh>".green().bold());
            io::stdout().flush().unwrap();

            let stdin = io::stdin();
            let reader = BufReader::new(stdin);

            for line in reader.lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(e) => {
                        eprintln!("Error reading input: {}", e);
                        break;
                    }
                };

                let text = line.trim();

                // Skip empty lines
                if text.is_empty() {
                    print!("{} ", "tzh>".green().bold());
                    io::stdout().flush().unwrap();
                    continue;
                }

                // Create callback for translation results
                let callback = |_original: &str, translation: &str| {
                    if translation.is_empty() {
                        return;
                    }

                    println!("{}", translation.bright_white());
                };

                // Translate the input
                let result = if has_blank(text) {
                    translator
                        .translate_line(text, &to, from.as_deref(), callback)
                        .await
                } else {
                    translator
                        .translate_word(text, &to, from.as_deref(), callback)
                        .await
                };

                if let Err(e) = result {
                    eprintln!("{} {}", "Translation failed:".red(), e);
                }

                // Show prompt for next input
                print!("{} ", "tzh>".green().bold());
                io::stdout().flush().unwrap();
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
