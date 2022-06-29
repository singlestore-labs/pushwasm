# pushwasm
This is a command-line utility that will allows you to create a Wasm UDF in
SingleStoreDB.  It will transfer a Wasm module, and, optionally, a WIT file,
from the local file system to the database.

## Compiling

Run:

  cargo build --release

## Usage

```
pushwasm [OPTIONS] <CONN> <WASMPATH> <FUNCNAME>

ARGS:
    <CONN>        Database connection information; must start with 'file://' or 'mysql://'.  If
                  a file is provided, the connection string will be read from it.
                  Example: mysql://user:pass@hostname:3306/dbname
    <WASMPATH>    The Wasm module path
    <FUNCNAME>    The Wasm function name

OPTIONS:
    -a, --abi <ABITYPE>    The ABI to use [default: canonical] [possible values: basic, canonical]
    -f, --force            Replace UDF if it exists already
    -h, --help             Print help information
    -p, --prompt           Prompt to enter password on console
    -w, --wit <WITPATH>    The WIT file path
```

## Examples

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string on the command line.

```bash
pushwasm \
    mysql://username:password@dbhostname:3306/mydatabase \
    --wit work/mathfuncs.wit \
    work/mathfuncs.wasm \
    power_of
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string on the command line and replacing what was there previously.

```bash
pushwasm \
    mysql://username:password@dbhostname:3306/mydatabase \
    --force \
    --wit work/mathfuncs.wit \
    work/mathfuncs.wasm \
    power_of
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string in a file.

```bash
echo "mysql://username:password@dbhostname:3306/mydatabase" > /home/fred/conn-info.txt

pushwasm \
    file:///home/fred/conn-info.txt \
    --wit work/mathfuncs.wit \
    work/mathfuncs.wasm \
    power_of
```

Create a UDF called `power_of` in SingleStoreDB from a Wasm module and WIT file
on the local file system using a connection string, but entering the password
interactively:

```bash
pushwasm \
    mysql://username:password@dbhostname:3306/mydatabase \
    --prompt
    --wit work/mathfuncs.wit \
    work/mathfuncs.wasm \
    power_of
```

## About SingleStoreDB

[Sign up](https://www.singlestore.com/try-free/) for a free SingleStore license. This allows you
   to run up to 4 nodes up to 32 gigs each for free. Grab your license key from
   [SingleStore portal](https://portal.singlestore.com/?utm_medium=osm&utm_source=github) and set it as an environment
   variable.

   ```bash
   export SINGLESTORE_LICENSE="singlestore license"
   ```

## Resources

* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)
