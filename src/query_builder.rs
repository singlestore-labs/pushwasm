use crate::arguments::{AggregateFunction, Functions, ImportSource, TableFunction, UserFunction};
use base64::Engine;
use std::fs;

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
        let wasm_source = self.shared.wasm_source.build(args);
        let wit_source = self
            .shared
            .wit_source
            .as_ref()
            .map(|w| w.build(args))
            .unwrap_or_default();
        let args = if args.len() == 1 {
            // The name of the argument is omitted for single args
            self.args[0].clone()
        } else {
            // A unique argument name is generated for each args, e.g.
            // "a int, b int, c int, ..."
            let generate_var_names = |(i, arg_type)| {
                let var_name_as_letter = ('a' as usize + i) as u8 as char;
                format!("{var_name_as_letter} {arg_type}")
            };

            self.args
                .iter()
                .enumerate()
                .map(generate_var_names)
                .collect::<Vec<_>>()
                .join(",")
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
            init = self.init,
            iter = self.iter,
            merge = self.merge,
            terminate = self.terminate,
            serialize = self
                .serialize
                .as_ref()
                .map(|s| format!("SERIALIZE WITH {}", s))
                .unwrap_or_default(),
            deserialize = self
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
