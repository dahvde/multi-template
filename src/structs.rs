use crate::args::Args;
use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};

/// Structured template repo data
///
/// Used to structure yaml configuration
/// file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub link: String,
    pub name: String,
}

/// Vectorized Repo data
///
/// Defines a vector for looping through the data
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub configs: Vec<Repo>,
}

/// Repository settings data
///
/// Holds required values to generate a new repo from a
/// template.
#[derive(Debug)]
pub struct RepoCommand {
    pub name: String,
    pub private: bool,
    pub owner: String,
    pub desc: String,
}

impl RepoCommand {
    /// Prompts the user with undefined arguments
    pub fn new(args: Args) -> Result<RepoCommand, Box<dyn std::error::Error>> {
        let name = match args.name.as_deref() {
            Some(e) => e.to_string(),
            None => Text::new("Name:").prompt()?,
        };
        let desc = match args.description.as_deref() {
            Some(e) => e.to_string(),
            None => Text::new("Description:").prompt()?,
        };
        let private = match args.private {
            true => true,
            _ => Confirm::new("Private (y/n):").prompt()?,
        };
        let owner = match args.owner.as_deref() {
            Some(e) => e.to_string(),
            None => Text::new("Owner:").with_placeholder("You").prompt()?,
        };

        Ok(RepoCommand {
            name,
            desc,
            private,
            owner,
        })
    }

    /// Concatenates the RepoCommand values into an
    /// executable string
    pub fn concat(&self) -> Vec<String> {
        let mut args = vec![];

        args.push("-F".to_string());
        args.push(format!("name={}", self.name));
        args.push("-F".to_string());
        args.push(format!("description={}", self.desc));
        args.push("-F".to_string());
        args.push(format!("private={}", self.private));

        if self.owner.len() > 0 {
            args.push("-F".to_string());
            args.push(format!("owner={}", self.owner));
        }

        args
    }
}
