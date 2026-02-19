# mical

A simple, line-oriented configuration language.

MICAL treats configuration as a flat sequence of key-value pairs — no deep nesting, no implicit hierarchy. Keys and values live on a single line, grouped visually with prefix blocks but remaining logically flat.

```mical
host    localhost
port    8080
enabled true

server. {
  timeout 30
  retries 3
}
```

```sh
$ mical eval config.mical
{
  "host": "localhost",
  "port": 8080,
  "enabled": true,
  "server.timeout": 30,
  "server.retries": 3
}
```

## Install

```sh
cargo install mical-cli
```

## Usage

```sh
# Evaluate a file (full JSON output)
mical eval config.mical

# Query a specific key
mical eval --get host config.mical

# Query by prefix
mical eval --prefix server. config.mical

# Write output to a file
mical eval -o out.json config.mical
```

## Documentation

See the [language specification](https://ryota2357.github.io/mical/) for details on syntax, keys, values, prefix blocks, and block strings.

## License

MIT
