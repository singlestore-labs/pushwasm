use crate::arguments::{
    AggregateFunction, Functions, ImportSource, ManualAggregateImports, TableFunction, UserFunction,
};
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::process::exit;

pub trait QueryBuilder {
    fn build(&self, args: &mut Vec<String>) -> String;
}

impl QueryBuilder for ImportSource {
    fn build(&self, args: &mut Vec<String>) -> String {
        match self {
            ImportSource::Base64(s) => {
                args.push(s.clone());
                "FROM BASE64 ?"
            }
            ImportSource::Path(p) => {
                let file_content = fs::read(p).expect("Could not read file");
                let encoded = base64::engine::general_purpose::STANDARD.encode(file_content);
                args.push(encoded);
                "FROM BASE64 ?"
            }
            ImportSource::Url(u) => {
                args.push(u.to_string());
                "FROM URL ?"
            }
        }
        .to_string()
    }
}

impl QueryBuilder for UserFunction {
    fn build(&self, args: &mut Vec<String>) -> String {
        let wasm_source = self.shared.wasm_source.build(args);
        let wit_source = self
            .shared
            .wit_source
            .as_ref()
            .map(|w| format!("WITH WIT {}", w.build(args)))
            .unwrap_or_default();
        format!(
            "
            CREATE {force} FUNCTION {name}
            AS WASM ABI {abi}
            {wasm_source}
            {wit_source}
            ",
            name = self.shared.name,
            force = if self.shared.force { "OR REPLACE" } else { "" },
            abi = self.shared.abi_type,
        )
    }
}

impl QueryBuilder for TableFunction {
    fn build(&self, args: &mut Vec<String>) -> String {
        let wasm_source = self.shared.wasm_source.build(args);
        let wit_source = self
            .shared
            .wit_source
            .as_ref()
            .map(|w| format!("WITH WIT {}", w.build(args)))
            .unwrap_or_default();
        format!(
            "
            CREATE {force} FUNCTION {name}
            RETURNS TABLE
            AS WASM ABI {abi} 
            {wasm_source}
            {wit_source}
            ",
            name = self.shared.name,
            force = if self.shared.force { "OR REPLACE" } else { "" },
            abi = self.shared.abi_type,
        )
    }
}

impl QueryBuilder for AggregateFunction {
    fn build(&self, args: &mut Vec<String>) -> String {
        let imports = match self.aggregate_imports.as_ref() {
            Some(imports) => imports.clone(),
            None => {
                if let Some(wit_source) = &self.shared.wit_source {
                    // Try auto-detect from wit
                    let functions = try_extract_member_functions_from_wit(wit_source.unpack());

                    ManualAggregateImports {
                        init: functions.get(&MemberFunction::Init).cloned(),
                        iter: functions.get(&MemberFunction::Iterate).cloned(),
                        merge: functions.get(&MemberFunction::Merge).cloned(),
                        terminate: functions.get(&MemberFunction::Terminate).cloned(),
                        serialize: functions.get(&MemberFunction::Serialize).cloned(),
                        deserialize: functions.get(&MemberFunction::Deserialize).cloned(),
                    }
                } else {
                    eprintln!("Cannot auto-detect functions without WIT source. Please specify a .wit file with --wit or manually provide the required functions.");
                    exit(1);
                }
            }
        };

        // Validate the imports
        imports.exit_if_not_valid();

        let wasm_source = self.shared.wasm_source.build(args);
        let wit_source = self
            .shared
            .wit_source
            .as_ref()
            .map(|w| format!("WITH WIT {}", w.build(args)))
            .unwrap_or_default();
        let args = if self.args.len() == 1 {
            // The name of the argument is omitted for single args
            self.args[0].clone()
        } else {
            // A unique argument name is generated for each args, e.g.
            // "a int, b int, c int, ..."
            let generate_var_names = |(i, arg_type)| {
                let var_name_as_letter = ('a' as usize + i) as u8 as char;
                format!("{var_name_as_letter} {arg_type}")
            };

            let generated_args = self
                .args
                .iter()
                .enumerate()
                .map(generate_var_names)
                .collect::<Vec<_>>();

            if generated_args.len() > 26 {
                eprintln!("Too many arguments for an aggregate function. Maximum is 26.");
                exit(1);
            }

            generated_args.join(",")
        };
        format!(
            "
            CREATE {force} AGGREGATE {name}({args})
            RETURNS {return_type}
            WITH STATE {state}
            AS WASM ABI {abi} 
            {wasm_source}
            {wit_source}
            INITIALIZE WITH {init}
            ITERATE WITH {iter}
            MERGE WITH {merge}
            TERMINATE WITH {terminate}
            {serialize}
            {deserialize}
            ",
            name = self.shared.name,
            force = if self.shared.force { "OR REPLACE" } else { "" },
            return_type = self.return_type,
            state = self.state_type,
            abi = self.shared.abi_type,
            init = imports.init.expect("Missing init function"),
            iter = imports.iter.expect("Missing iterate function"),
            merge = imports.merge.expect("Missing merge function"),
            terminate = imports.terminate.expect("Missing terminate function"),
            serialize = imports
                .serialize
                .as_ref()
                .map(|s| format!("SERIALIZE WITH {}", s))
                .unwrap_or_default(),
            deserialize = imports
                .deserialize
                .as_ref()
                .map(|s| format!("DESERIALIZE WITH {}", s))
                .unwrap_or_default(),
        )
    }
}

impl QueryBuilder for Functions {
    fn build(&self, args: &mut Vec<String>) -> String {
        match self {
            Functions::Udf(f) => f.build(args),
            Functions::Tvf(f) => f.build(args),
            Functions::Udaf(f) => f.build(args),
        }
    }
}

/// Automatically try to extract the aggregate member functions from the WIT spec.
/// Note: This requires the specific members functions to contain relevant keywords in their name.
fn try_extract_member_functions_from_wit(source: String) -> HashMap<MemberFunction, String> {
    let mut functions = HashMap::new();
    for line in source.lines() {
        if line.contains("func(") {
            // Take the string up to the first colon
            if let Some(name) = line.split(':').next() {
                if let Some(func) = MemberFunction::from_name(name) {
                    if let Some(duplicate) = functions.insert(func, name.replace("-", "_")) {
                        eprintln!("Duplicate match for function name: {name} vs {duplicate}");
                        exit(1);
                    }
                }
            }
        }
    }
    functions
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum MemberFunction {
    Init,
    Iterate,
    Merge,
    Terminate,
    Serialize,
    Deserialize,
}

impl MemberFunction {
    fn from_name(name: &str) -> Option<Self> {
        if name.contains("wit") {
            return None;
        }
        if name.contains("init") || name.contains("new") {
            Some(Self::Init)
        } else if name.contains("iter") || name.contains("update") || name.contains("add") {
            Some(Self::Iterate)
        } else if name.contains("merge") {
            Some(Self::Merge)
        } else if name.contains("terminate") || name.contains("get") || name.contains("finalize") {
            Some(Self::Terminate)
        } else if name.contains("deserialize") || name.contains("decode") {
            Some(Self::Deserialize)
        } else if name.contains("serialize") || name.contains("encode") {
            Some(Self::Serialize)
        } else {
            None
        }
    }
}
