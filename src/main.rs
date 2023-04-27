mod arguments;
mod query_builder;

use crate::arguments::{Cli, Functions};
use crate::query_builder::QueryBuilder;
use anyhow::Context;
use clap::Parser;
use mysql::prelude::*;
use mysql::*;
use rpassword::prompt_password;
use std::fs;
use std::process::exit;
use url::Url;

fn parse_url(s: &str) -> Url {
    let url = Url::parse(s);
    if let Err(e) = url {
        eprintln!("Error parsing connection specifier: {}", e);
        exit(1);
    }
    url.unwrap()
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Extract the shared config
    let shared = match &args.functions {
        Functions::Udf(f) => &f.shared,
        Functions::Tvf(f) => &f.shared,
        Functions::Udaf(f) => &f.shared,
    };

    // Convert the connection specifier URL to a connection string.  If it
    // begins with file://, we'll read the connection string from a file.
    let url = &shared.connection;
    let mut conn_url = match shared.connection.scheme() {
        "mysql" => url.clone(),
        "file" => {
            let file_res = fs::read_to_string(url.path());
            if let Err(e) = &file_res {
                eprintln!(
                    "Error reading connection info from file '{}': {}",
                    url.path(),
                    e
                );
                exit(1);
            }
            let new_url = parse_url(file_res.unwrap().as_str());
            if new_url.scheme() != "mysql" {
                eprintln!("File-based URL must begin with 'mysql://'.");
                exit(1);
            }
            new_url
        }
        _ => {
            eprintln!("Invalid config specification.  Must begin with 'file://' or 'mysql://'");
            exit(1);
        }
    };

    // Prompt for password, if requested.
    if shared.prompt {
        let pass = prompt_password("Password: ").context("Error reading password")?;
        conn_url
            .set_password(Some(&pass))
            .map_err(|()| anyhow::anyhow!("Couldn't read password"))
            .context("Could not set password")?;
    };

    // Get the final connection string.
    let conn_str: String = conn_url.into();

    // Open the SQL connection using the connection string.
    let opts = Opts::from_url(&conn_str)?;
    let mut conn = Conn::new(opts).context("Error opening SQL connection")?;

    // Generate the query body & params
    let mut params = vec![];
    let query_str = args.functions.build(&mut params);

    // Execute the query
    conn.exec::<String, _, _>(query_str, params)
        .map_err(|e| anyhow::anyhow!("Error executing query: {}", e))
        .context("Could not create function")?;

    println!("Wasm function was created successfully.");

    Ok(())
}
