use base64::Engine;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::from_utf8;
use url::Url;

#[derive(Debug, Args)]
pub struct SharedFunctionOptions {
    #[clap(short, long, help = "Name of the function", required = true)]
    pub name: String,
    #[clap(long = "wasm", help = "Wasm import", value_enum, required = true)]
    pub wasm_source: ImportSource,
    #[clap(long = "wit", value_enum, help = "WIT import")]
    pub wit_source: Option<ImportSource>,
    #[clap(
        short,
        long,
        help = "Force replace existing function(s)",
        default_value = "false"
    )]
    pub force: bool,
    #[clap(short, long, help = "Prompt for password", default_value = "false")]
    pub prompt: bool,
    #[clap(long = "abi", help = "ABI type", default_value = "canonical")]
    pub abi_type: AbiType,
    #[clap(
        short,
        long = "conn",
        help = "Database connection string. Must begin with 'file://' or 'mysql://'"
    )]
    pub connection: Url,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Creates a Wasm User-defined function"
)]
pub struct UserFunction {
    #[clap(flatten)]
    pub shared: SharedFunctionOptions,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Creates a Wasm Table-valued function"
)]
pub struct TableFunction {
    #[clap(flatten)]
    pub shared: SharedFunctionOptions,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Creates a Wasm User-defined aggregate function"
)]
pub struct AggregateFunction {
    #[clap(flatten)]
    pub shared: SharedFunctionOptions,
    #[clap(
        short = 't',
        long = "type",
        help = "Return type ~ RETURNS",
        required = true
    )]
    pub return_type: String,
    #[clap(
        short,
        long = "arg",
        help = "Function argument types. A name will be automatically generated for each argument.",
        required = true,
        num_args=1..,
    )]
    pub args: Vec<String>,
    #[clap(
        short,
        long = "state",
        help = "State type ~ WITH STATE",
        required = true
    )]
    pub state_type: String,
    #[clap(flatten)]
    pub aggregate_imports: Option<ManualAggregateImports>,
}

#[derive(Args, Clone)]
pub struct ManualAggregateImports {
    #[clap(long, help = "Initialization method ~ INITIALIZE WITH")]
    pub init: Option<String>,
    #[clap(long, help = "Update method ~ ITER WITH")]
    pub iter: Option<String>,
    #[clap(long, help = "Merge method ~ MERGE WITH")]
    pub merge: Option<String>,
    #[clap(long, help = "Terminate method ~ ITER WITH")]
    pub terminate: Option<String>,
    #[clap(long, help = "Serialize method ~ SERIALIZE WITH")]
    pub serialize: Option<String>,
    #[clap(long, help = "Deserialize method ~ DESERIALIZE WITH")]
    pub deserialize: Option<String>,
}

impl ManualAggregateImports {
    pub fn exit_if_not_valid(&self) {
        let exit_with = |name| {
            eprintln!("Aggregate functions require a {name} functions");
            exit(1);
        };
        if self.init.is_none() {
            exit_with("init");
        }
        if self.iter.is_none() {
            exit_with("iter");
        }
        if self.merge.is_none() {
            exit_with("merge");
        }
        if self.terminate.is_none() {
            exit_with("terminate");
        }

        if self.serialize.is_none() != self.serialize.is_none() {
            eprintln!("Aggregate functions require both serialize and deserialize functions");
            exit(1);
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImportSource {
    Base64(String),
    Path(PathBuf),
    Url(Url),
}

impl ImportSource {
    /// Unpacks the import source into a string.
    pub fn unpack(&self) -> String {
        match self {
            ImportSource::Base64(base64) => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(base64)
                    .expect("Invalid base64");
                from_utf8(&decoded).expect("Invalid UTF-8").to_string()
            }
            ImportSource::Path(path) => read_to_string(path)
                .expect("Could not read file")
                .to_string(),
            ImportSource::Url(_) => {
                todo!("Downloading wit from URL is not supported!")
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "Push a Wasm module into SingleStoreDB.")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub functions: Functions,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum Functions {
    #[clap(name = "udf")]
    Udf(UserFunction),
    #[clap(name = "tvf")]
    Tvf(TableFunction),
    #[clap(name = "agg")]
    Udaf(AggregateFunction),
}

impl std::str::FromStr for ImportSource {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(if value.starts_with("http") {
            Self::Url(Url::parse(value).map_err(|e| format!("Could not parse URL: {}", e))?)
        } else if Path::new(&value).exists() {
            Self::Path(Path::new(value).to_path_buf())
        } else {
            Self::Base64(value.to_string())
        })
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum AbiType {
    Basic,
    Canonical,
}

impl Display for AbiType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AbiType::Basic => "BASIC",
            AbiType::Canonical => "CANONICAL",
        };
        write!(f, "{}", s)
    }
}
