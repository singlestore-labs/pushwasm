# pushwasm
This is a command-line utility that will allows you to create a Wasm UDF in
SingleStoreDB.  It will transfer a Wasm module, and, optionally, a WIT file,
from the local file system to the database.

## Compiling

Run:

  cargo build --release

## Usage

```shell
Push a Wasm module into SingleStoreDB.

pushwasm <COMMAND>

Commands:
  udf   Creates a Wasm User-defined function
  tvf   Creates a Wasm Table-valued function
  udaf  Creates a Wasm User-defined aggregate function
  help  Print this message or the help of the given subcommand(s)
```

```shell
Creates a Wasm User-defined function

Usage: pushwasm <COMMAND> [OPTIONS] --name <NAME> --wasm <WASM_SOURCE> --conn <CONNECTION>

Options:
  -n, --name <NAME>
          Name of the function

      --wasm <WASM_SOURCE>
          Wasm import

      --wit <WIT_SOURCE>
          WIT import

  -f, --force
          Force replace existing function(s)

  -p, --prompt
          Prompt for password

      --abi <ABI_TYPE>
          ABI type
          
          [default: canonical]
          [possible values: basic, canonical]

  -c, --conn <CONNECTION>
          Database connection string. Must begin with 'file://' or 'mysql://'
```

## Examples

Create a [UDF](https://docs.singlestore.com/managed-service/en/reference/code-engine---powered-by-wasm/create-wasm-udfs.html) called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string on the command line.

```shell
pushwasm udf \
    -n power_of \
    --wasm work/mathfuncs.wasm \ 
    --wit work/mathfuncs.wit \
    --abi canonical \
    --conn 'mysql://username:password@dbhostname:3306/mydatabase'
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string on the command line and replacing what was there previously.

```shell
pushwasm udf \
    -n power_of \
    --force \
    --wasm work/mathfuncs.wasm \ 
    --wit work/mathfuncs.wit \
    --abi canonical \
    --conn 'mysql://username:password@dbhostname:3306/mydatabase'
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string in a file.

```shell
echo "mysql://username:password@dbhostname:3306/mydatabase" > /home/fred/conn-info.txt

pushwasm udf \
    -n power_of \
    --wasm work/mathfuncs.wasm \ 
    --wit work/mathfuncs.wit \
    --abi canonical \
    --conn 'file:///home/fred/conn-info.txt'
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string, but entering the password
interactively:

```shell
pushwasm udf \
    -n power_of \
    --wasm work/mathfuncs.wasm \ 
    --wit work/mathfuncs.wit \
    --abi canonical \
    --prompt \
    --conn 'mysql://username:password@dbhostname:3306/mydatabase'
```

Creates a UDAF called `sum_of` in SingleStoreDB from a Wasm module and WIT file, by auto-deducing the relevant
functions from WIT file:

```shell
pushwasm udaf \
    -n sum_of \
    --wasm work/aggfuncs.wasm \
    --wit work/aggfuncs.wit \
    --abi canonical \
    --conn 'mysql://username:password@dbhostname:3306/mydatabase' \
    --state 'int not null' \
    --type 'int not null' \ 
    --arg 'int not null'
```

Creates a [UDAF](https://docs.singlestore.com/managed-service/en/reference/sql-reference/procedural-sql-reference/create-aggregate.html) called `sum_of` in SingleStoreDB from a Wasm module and WIT file using HANDLE state & by manually
specifying the member functions:

```shell
pushwasm udaf \
    -n sum_of \
    --wasm work/aggfuncs.wasm \
    --wit work/aggfuncs.wit \
    --abi canonical \
    --conn 'mysql://username:password@dbhostname:3306/mydatabase' \
    --state handle \
    --type 'int not null' \ 
    --arg 'int not null' \
    --init handle_init \ 
    --iter handle_add \ 
    --merge handle_merge \ 
    --terminate handle_get \ 
    --serialize handle_serialize \
     --deserialize handle_deserialize
```


## About SingleStoreDB

[Sign up](https://www.singlestore.com/try-free/) for a free SingleStore license. This allows you
   to run up to 4 nodes up to 32 gigs each for free. Grab your license key from
   [SingleStore portal](https://portal.singlestore.com/?utm_medium=osm&utm_source=github) and set it as an environment
   variable.

   ```shell
   export SINGLESTORE_LICENSE="singlestore license"
   ```

## Resources

* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)
