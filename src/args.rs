use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Private repository toggle
    #[arg(short, long, action)]
    pub private: bool,

    /// Pre-select template (config name or /owner/repo)
    #[arg(short, long)]
    pub template: Option<String>,

    /// Config path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// New repo name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Repo description
    #[arg(long)]
    pub description: Option<String>,

    /// Explicitly define owner
    #[arg(short, long)]
    pub owner: Option<String>,
}
