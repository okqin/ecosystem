use anyhow::Context;
use std::fs;
use std::mem::size_of;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Parse json error: {0}")]
    ParseJson(#[from] serde_json::Error),

    #[error("Custom error: {0}")]
    Custom(String),
}

fn main() -> Result<(), anyhow::Error> {
    println!(
        "size of the MyError::Io is: {}",
        size_of::<std::io::Error>()
    );
    println!(
        "size of the MyError::Parse is: {}",
        size_of::<std::num::ParseIntError>()
    );
    println!(
        "size of the MyError::ParseJson is: {}",
        size_of::<serde_json::Error>()
    );
    println!("size of the MyError::Custom is: {}", size_of::<String>());
    println!("size of the MyError is: {}", size_of::<MyError>());
    println!(
        "size of the anyhow::Error is: {}",
        size_of::<anyhow::Error>()
    );
    let filename = "nonexistent.txt";
    let _fd =
        fs::File::open(filename).with_context(|| format!("Can not find file: {}", filename))?;
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("An error occurred".to_string()))
}
