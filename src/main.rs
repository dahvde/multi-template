mod args;
mod structs;

use crate::args::Args;
use crate::structs::{Config, RepoCommand, Repo};

use clap::Parser;
use inquire::Select;
use serde_yaml;
use serde_json::{self, Value};
use std::path::PathBuf;
use std::process::Command;

/// Structure for syntax highlighting.
pub enum Syntax {
    Number,
    Bool,
    String,
    Error,
    Key,
    Null,
    Custom(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(); // Grab cmd arguments

    // Fetch config file directory
    let cfg_path = match args.config.as_deref() {
        Some(e) => e.to_path_buf(),
        None    => bin_dir_file("config.yaml")?,
    };

    // Read and parse file
    let file = std::fs::File::open(cfg_path)?;
    let cfg: Config = serde_yaml::from_reader(file)?;

    let mut vec_name = vec![];
    let mut vec_link = vec![];

    // Split up template values
    for x in &cfg.configs {
        vec_name.push(x.name.as_str());
        vec_link.push(x.link.as_str());
    }

    // Check template argument
    let repo_name = match args.template.as_deref() {
        Some(e) => e,
        None    => Select::new("Template:", vec_name.to_owned()).prompt()?,
    };
   
    // Define selected template
    let repo: Repo = Repo {
        name: repo_name.to_string(),
        link: match vec_name.iter().position(|&x| x == repo_name) {
            Some(e)  => vec_link[e as usize].to_string(),
            None    => repo_name.to_string(),
        }
    };

    // Initialize new repository request
    let mut cmd_str = vec![
                "api".to_string(), 
                format!("/repos/{}/generate", repo.link),
                "-X".to_string(), "POST".to_string(),
        ];
    cmd_str.extend(RepoCommand::new(args)?.concat());

    // Generate new repository
    let output = Command::new("gh")
                        .args(&cmd_str)
                        .output()?;

    // Convert json response to a Value
    let mut res: Value = serde_json::from_str(String::from_utf8(output.stdout)?.as_str())?;

    // Check if response was a success
    match res.get("id") {
        Some(_) => {
            // Explicity defined json keys
            let keep_fields = vec![
                "id", "clone_url", "full_name",
                "name", "owner", "ssh_url", "private",
                "default_branch"];
            
            // Retain explicity defined key values pairs
            if let Some(obj) = res.as_object_mut() {
                obj.retain(|key, _| keep_fields.contains(&key.as_str()));
                obj.get_mut("owner")
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .retain(|key, _| key.as_str() == "login");
            }

            println!(
                "\x1b[1m{}\x1b[0m\n",
                cfmt(Syntax::Custom(32), 
                &format!("Repo {} Created", res.get("name").unwrap()
            )));

            // Output organized successul response data
            print_key_value_pairs(&res, "".to_string(), 0);

            // Output commands to clone the generated repository
            println!("\n{}",  cfmt(Syntax::Custom(35), &"\nTo clone the repo use:"));
            array_print(&[
                cfmt(Syntax::Custom(32), &"git"),
                cfmt(Syntax::Custom(33), &"clone"),
                cfmt(Syntax::Custom(37), &res.get("ssh_url").unwrap().to_string().trim_matches('"')),
            ], " ");
            println!("{}", cfmt(Syntax::Custom(34), &"or"));
            array_print(&[
                cfmt(Syntax::Custom(32), &"git"),
                cfmt(Syntax::Custom(33), &"clone"),
                cfmt(Syntax::Custom(37), &res.get("clone_url").unwrap().to_string().trim_matches('"')),
            ], " ");
        },
        None => {
            println!("{}", cfmt(Syntax::Error, &"Error Occured"));
            // Output organized error response data
            print_key_value_pairs(&res, "".to_string(), 1);
        }
    };

    Ok(())
}

/// Outputs each value from array
///
/// Iterates through the String slice array
/// and appends the spacer to seperate each
/// array value
/// ```
/// Example:
/// array_print(&["I walked", "my dog"], " ")
/// // outputs
/// "I walked my dog"
/// ```
fn array_print(values: &[String], spacer: &str) {
    println!("{}", values.into_iter().map(|x| x.to_owned() + spacer).collect::<String>());
}

/// Returns formatted string with color annotation
///
///
/// Using the Syntax enum a color is selected to join
/// with the `data<&T>`.
/// ```
/// cfmt(Syntax::String, &"Bollocks")
/// // returns
/// "\x1b[0;32mBollocks\x1b[0m"
/// ```
fn cfmt<T: std::fmt::Display>(num: Syntax, data: &T) -> String {
    let color = match num {
            Syntax::String => 32,
            Syntax::Number => 35,
            Syntax::Bool => 33,
            Syntax::Error => 31,
            Syntax::Null => 35,
            Syntax::Key => 34,
            Syntax::Custom(val) => val,
    };
    format!("\x1b[1;{}m{}\x1b[0m", color, data)
}

/// Prints out json data from recursive use
/// 
/// Recurses through `data` of `&Value` and prints out
/// pretty formatted json in a yaml like pattern with
fn print_key_value_pairs(data: &Value, prefix: String, depth: usize) {
    match data {
        Value::Null => println!("{}{}", tab(depth), cfmt(Syntax::Null, &"null")),
        Value::Bool(b) => println!("{}{}{}", tab(depth), prefix, cfmt(Syntax::Bool, b)),
        Value::Number(n) => println!("{}{}{}", tab(depth), prefix, cfmt(Syntax::Number, n)),
        Value::String(s) => println!("{}{}{}", tab(depth), prefix, cfmt(Syntax::String, s)),
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                let index_fmt = format!("{}| ", cfmt(Syntax::Custom(35), &i));
                print_key_value_pairs(item, index_fmt, depth);
            }
        }
        Value::Object(obj) => {
            for (key, value) in obj.iter() {
                let mut local_depth = depth as i32;
                print!("{}{}: {}", tab(depth), cfmt(Syntax::Key, key), 
                    match value {
                        Value::Object(_) | Value::Array(_) => "\n",
                        _ => {
                            local_depth = -1;
                            ""
                        },
                    }
                );

                print_key_value_pairs(value, "".to_string(), (local_depth+1) as usize); 
            }
        }
    }
}

/// Returns indented tab with
/// specififed size
///
/// Uses a depth value of `usize`
/// to repeat a string defined as
/// tab
fn tab(depth: usize) -> String {
    return cfmt(Syntax::Custom(30), &"| ".repeat(depth));
}

/// Returns path to a file
///
/// Starting from the bin's local directiory a 
/// `PathBuf` is created to reference the specified 
/// `file_name`
fn bin_dir_file(file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let file_split: Vec<&str> = file_name.split(".").collect();

    match dir.rfind("/") {
        Some(e) => {
            let mut buf = PathBuf::new();
            buf.push(dir[0..e].to_string());
            buf.push(file_split[0]);
            buf.set_extension(file_split[1]);
            Ok(buf)
        }
        None => Err("Failed to format dir")?,
    }
}

