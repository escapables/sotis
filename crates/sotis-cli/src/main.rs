use clap::{Parser, Subcommand};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
#[command(name = "sotis", about = "Fuzzy file search for Linux")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search files by content and/or filename.
    Search {
        /// The search query.
        query: String,

        /// Search filenames only.
        #[arg(short = 'n', long)]
        name_only: bool,

        /// Search file content only.
        #[arg(short = 'c', long)]
        content_only: bool,
    },

    /// Build or update the search index.
    Index {
        /// Incremental update instead of full rebuild.
        #[arg(long)]
        update: bool,
    },

    /// Add a folder to the index configuration.
    Add {
        /// Path to the folder.
        path: String,
    },

    /// Remove a folder from the index configuration.
    Remove {
        /// Path to the folder.
        path: String,
    },

    /// Show index status and statistics.
    Status,

    /// Show or edit configuration.
    Config,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            name_only,
            content_only,
        } => {
            let _mode = match (name_only, content_only) {
                (true, _) => sotis_core::search::SearchMode::FilenameOnly,
                (_, true) => sotis_core::search::SearchMode::ContentOnly,
                _ => sotis_core::search::SearchMode::Combined,
            };
            println!("Searching for: {query}");
            Ok(())
        }
        Commands::Index { update } => {
            if update {
                println!("Updating index...");
            } else {
                println!("Rebuilding index...");
            }
            Ok(())
        }
        Commands::Add { path } => {
            println!("Adding folder: {path}");
            Ok(())
        }
        Commands::Remove { path } => {
            println!("Removing folder: {path}");
            Ok(())
        }
        Commands::Status => {
            println!("Index status: not yet implemented");
            Ok(())
        }
        Commands::Config => {
            let config_path = sotis_core::config::config_path();
            println!("Config path: {}", config_path.display());
            Ok(())
        }
    }
}
