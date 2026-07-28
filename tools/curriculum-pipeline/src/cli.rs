use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "curriculum-pipeline",
    version,
    about = "Sarvo Shala curriculum authoring pipeline"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Extract {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Generate {
        #[arg(long)]
        text: PathBuf,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        grade: i64,
        #[arg(long = "chapter-title")]
        chapter_title: String,
        #[arg(long = "source-authority")]
        source_authority: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, help = "Use claude CLI instead of Anthropic API directly")]
        use_claude: bool,
    },
    Validate {
        #[arg(long)]
        file: PathBuf,
    },
    Compile {
        #[arg(long = "content-dir")]
        content_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}
