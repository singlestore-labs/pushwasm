use clap::{arg, Arg, ArgAction, Command};
use mysql::*;
use mysql::prelude::*;
use rpassword::prompt_password;
use std::fs;
use std::path::PathBuf;
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

fn main() {
    // Parse arguments.
    let matches = Command::new("pushwasm")
        .about("Push a Wasm module into SingleStoreDB.")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .arg(
            Arg::new("WITPATH")
                .short('w')
                .long("wit")
                .help("The WIT file path")
                .takes_value(true)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("ABITYPE")
                .short('a')
                .long("abi")
                .help("The ABI to use")
                .takes_value(true)
                .value_parser(["basic", "canonical"])
                .default_value("canonical")
        )
        .arg(
            Arg::new("FORCE")
                .short('f')
                .long("force")
                .help("Replace UDF/TVF if it exists already")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("TVF")
                .short('t')
                .long("tvf")
                .help("Deploy a TVF instead of a UDF")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("PROMPT")
                .short('p')
                .long("prompt")
                .help("Prompt to enter password on console")
                .action(ArgAction::SetTrue),
        )
        .arg(arg!(<CONN> "The database connection string")
            .help("Database connection information; must start with 'file://' \
                   or 'mysql://'.  If a file is provided, the connection \
                   string will be read from it.\nExample: \
                   mysql://user:pass@hostname:3306/dbname"))
        .arg(arg!(<WASMPATH> "The Wasm module path")
            .value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(<FUNCNAME> "The Wasm function name"))
        .get_matches();

    let conn_spec = matches.get_one::<String>("CONN").expect("required");
    let func_name = matches.get_one::<String>("FUNCNAME").expect("required");
    let wasm_path = matches.get_one::<PathBuf>("WASMPATH").expect("required");
    let wit_path = matches.get_one::<PathBuf>("WITPATH");
    let abi = matches.get_one::<String>("ABITYPE");
    let force = *matches.get_one::<bool>("FORCE").unwrap_or_else(|| &false);
    let tvf = *matches.get_one::<bool>("TVF").unwrap_or_else(|| &false);
    let prompt = *matches.get_one::<bool>("PROMPT").unwrap_or_else(|| &false);
    let has_wit = wit_path.is_some();
    let func_kind = if tvf { "TVF" } else { "UDF" };
    
    // Convert the connection specifier URL to a connection string.  If it 
    // begins with file://, we'll read the connection string from a file.
    let url = parse_url(conn_spec);
    let mut conn_url: Url = match url.scheme() {
        "mysql" => url.clone(),
        "file" => {
            let file_res = fs::read_to_string(url.path());
            if let Err(e) = &file_res {
                eprintln!("Error reading connection info from file '{}': {}", url.path(), e);
                exit(1);
            }
            let new_url = parse_url(file_res.unwrap().as_str());
            if new_url.scheme() != "mysql" {
                eprintln!("File-based URL must begin with 'mysql://'.");
                exit(1);
            }
            new_url
        },
        _ => {
            eprintln!("Invalid config specification.  Must begin with 'file://' or 'mysql://'");
            exit(1);
        }
    };

    // Prompt for password, if requested.
    let mut password: Option<String> = None;
    if prompt {
        let pass_res = prompt_password("Password: ");
        if let Err(e) = pass_res {
            eprintln!("Error reading password: {}", e);
            exit(1);
        }
        password = Some(pass_res.unwrap());
    }
    if let Some(p) = password {
        if let Err(_) = conn_url.set_password(Some(&p)) {
            eprintln!("Error setting password");
            exit(1);
        }
    }

    // Get the final connection string.
    let conn_str: String = conn_url.into();

    // Generate the CREATE FUNCTION template.
    let mut stmt_str = String::from("CREATE ");
    if force {
        stmt_str += "OR REPLACE ";
    }
    stmt_str += format!("FUNCTION {} ", func_name).as_str();
    if tvf {
        stmt_str += "RETURNS TABLE ";
    }
    stmt_str += format!("AS WASM ABI {} FROM BASE64 ?", abi.unwrap()).as_str();
    if has_wit {
        stmt_str = stmt_str + " WITH WIT FROM BASE64 ?";
    }

    // Read the Wasm module data and base-64 encode it.
    let file_res = fs::read(wasm_path);
    if let Err(e) = &file_res {
        eprintln!("Error reading Wasm file '{}': {}", wasm_path.to_str().unwrap(), e);
        exit(1);
    }
    let encoded_wasm = base64::encode(file_res.unwrap());

    // Read the WIT data and base-64 encode it.
    let mut encoded_wit = String::from("");
    if let Some(wit_path) = wit_path {
        let file_res = fs::read(wit_path);
        if let Err(e) = &file_res {
            eprintln!("Error reading WIT file '{}': {}", wit_path.to_str().unwrap(), e);
            exit(1);
        }
        encoded_wit = base64::encode(file_res.unwrap());
    }

    // Open the SQL connection using the connection string.
    let pool = Pool::new(conn_str.as_str());
    if let Err(e) = pool {
        eprintln!("Error opening SQL connection: {}", e);
        exit(1);
    }
    let conn = pool.unwrap().get_conn();
    if let Err(e) = conn {
        eprintln!("Error opening SQL connection: {}", e);
        exit(1);
    }
    let mut conn = conn.unwrap();

    // Execute the CREATE FUNCTION statement with the encoded Wasm and WIT data.
    let mut p = vec![encoded_wasm];
    if has_wit {
        p.push(encoded_wit);
    }
    //let p = (encoded_wasm, encoded_wit);
    let exec_res = conn.exec::<String, _, _>(stmt_str, p);
    if let Err(e) = exec_res {
        eprintln!("Error while creating {}: {}", &func_kind, e);
        exit(1);
    }
    println!("Wasm {} '{}' was created successfully.", &func_kind, &func_name);
}
