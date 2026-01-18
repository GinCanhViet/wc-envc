mod engine;
mod interactive;
mod scanner;

use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;

use engine::ProcessMode;

/// wc-envc - Encrypt/decrypt .env files securely
#[derive(Parser)]
#[command(name = "wc-envc")]
#[command(version, about, long_about = None)]
#[command(after_help = "Run 'wc-envc <COMMAND> -h' for more information on a command.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt .env file
    Encrypt {
        /// Input file (optional in interactive mode)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        
        /// Password for encryption
        #[arg(short, long, env = "WC_ENVC_PASSWORD")]
        password: Option<String>,
        
        /// Input file path
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip confirmation prompts (overwrite files)
        #[arg(short, long, default_value = "false")]
        yes: bool,
    },
    
    /// Decrypt .env.enc file
    Decrypt {
        /// Input file (optional in interactive mode)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        
        /// Password for decryption
        #[arg(short, long, env = "WC_ENVC_PASSWORD")]
        password: Option<String>,
        
        /// Input file path
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip confirmation prompts (overwrite files)
        #[arg(short, long, default_value = "false")]
        yes: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!();
        eprintln!("{} {}", style("❌").red(), style(e).red());
        eprintln!();
        eprintln!("{} Run '{}' to see available commands.", 
            style("💡").yellow(),
            style("wc-envc -h").cyan()
        );
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Encrypt { file, password, input, output, yes } => {
            handle_encrypt(file, password, input, output, yes)
        }
        Commands::Decrypt { file, password, input, output, yes } => {
            handle_decrypt(file, password, input, output, yes)
        }
    }
}

fn handle_encrypt(
    file: Option<PathBuf>,
    password: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    yes: bool,
) -> Result<()> {
    // Determine input file: -i flag takes priority over positional arg
    let input_file = input.or(file);
    
    // If both password and output are provided, run in one-liner mode
    if let (Some(ref input_path), Some(ref output_path)) = (&input_file, &output) {
        interactive::run_one_liner(
            input_path.clone(),
            output_path.clone(),
            password,
            yes,
            ProcessMode::Encrypt,
        )
    } else if let Some(ref input_path) = input_file {
        // Quick mode: file specified but no output
        if password.is_some() && output.is_none() {
            // One-liner with default output
            let default_output = scanner::default_output_name(input_path, ProcessMode::Encrypt);
            interactive::run_one_liner(
                input_path.clone(),
                default_output,
                password,
                yes,
                ProcessMode::Encrypt,
            )
        } else {
            // Interactive mode with pre-selected file
            interactive::run_interactive_encrypt(Some(input_path.clone()))
        }
    } else {
        // Full interactive mode
        interactive::run_interactive_encrypt(None)
    }
}

fn handle_decrypt(
    file: Option<PathBuf>,
    password: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    yes: bool,
) -> Result<()> {
    // Determine input file: -i flag takes priority over positional arg
    let input_file = input.or(file);
    
    // If both password and output are provided, run in one-liner mode
    if let (Some(ref input_path), Some(ref output_path)) = (&input_file, &output) {
        interactive::run_one_liner(
            input_path.clone(),
            output_path.clone(),
            password,
            yes,
            ProcessMode::Decrypt,
        )
    } else if let Some(ref input_path) = input_file {
        // Quick mode: file specified but no output
        if password.is_some() && output.is_none() {
            // One-liner with default output
            let default_output = scanner::default_output_name(input_path, ProcessMode::Decrypt);
            interactive::run_one_liner(
                input_path.clone(),
                default_output,
                password,
                yes,
                ProcessMode::Decrypt,
            )
        } else {
            // Interactive mode with pre-selected file
            interactive::run_interactive_decrypt(Some(input_path.clone()))
        }
    } else {
        // Full interactive mode
        interactive::run_interactive_decrypt(None)
    }
}
