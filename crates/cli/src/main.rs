use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mamba")]
#[command(version, about = "Mamba - Python syntax, Rust speed", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file to compile and run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Mamba file to binary
    Build {
        /// Input file
        file: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run an existing binary
    Run {
        /// Binary file to run
        file: PathBuf,
    },

    /// Check syntax without compiling
    Check {
        /// File to check
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    let cli = Cli::parse();

    // TODO: Implement actual compilation pipeline
    println!("{}", "Mamba v0.1.0".green().bold());
    println!("{}", "Python syntax. Rust speed. One tool.".dimmed());
    println!();

    match &cli.command {
        Some(Commands::Build { file, output }) => {
            println!("Building: {}", file.display());
            if let Some(out) = output {
                println!("Output: {}", out.display());
            }
            println!("{}", "Not yet implemented".yellow());
        }
        Some(Commands::Run { file }) => {
            println!("Running: {}", file.display());
            println!("{}", "Not yet implemented".yellow());
        }
        Some(Commands::Check { file }) => {
            println!("Checking: {}", file.display());
            println!("{}", "Not yet implemented".yellow());
        }
        None => {
            if let Some(file) = &cli.file {
                println!("Compiling and running: {}", file.display());
                println!("{}", "Not yet implemented".yellow());
            } else {
                println!("{}", "No file specified. Use --help for usage.".red());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
