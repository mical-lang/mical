# Installation

MICAL can be installed using Cargo (the Rust package manager) or Nix.

## Using Cargo

MICAL is available on [crates.io](https://crates.io/crates/mical-cli). You can install the CLI tool directly using:

```bash
cargo install mical-cli
```

## Using Nix

If you use Nix, you can run or install MICAL from the GitHub repository.

### Running without installing

You can run MICAL immediately without adding it to your profile:

```bash
nix run github:mical-lang/mical
```

### Installing to your profile

To install MICAL permanently into your Nix profile:

```bash
nix profile install github:mical-lang/mical
```

## Building from Source

If you prefer to build from the source code:

```bash
git clone https://github.com/mical-lang/mical.git
cd mical
cargo install --path .
```
