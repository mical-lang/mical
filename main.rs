use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use mical_cli_syntax::ast::{AstNode as _, SourceFile};

#[derive(Parser)]
#[command(name = "mical", version, about = "Mical configuration language tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Evaluate a .mical file and output the result
    Eval(EvalArgs),

    /// (internal) Debug utilities — not for end users
    #[command(hide = true)]
    Dev(DevArgs),
}

#[derive(Args)]
struct EvalArgs {
    /// Path to the .mical file to evaluate
    file: PathBuf,

    /// Write the result to a file instead of stdout
    #[arg(short = 'o', long = "output-path")]
    output_path: Option<PathBuf>,

    /// Output format (currently only "json")
    #[arg(short = 'f', long = "format", default_value = "json")]
    format: OutputFormat,

    #[command(flatten)]
    query: QueryArgs,
}

#[derive(Args)]
#[group(multiple = false)]
struct QueryArgs {
    /// Return the value(s) for an exact key match
    #[arg(long)]
    get: Option<String>,

    /// Return all entries whose key starts with the given prefix
    #[arg(long)]
    prefix: Option<String>,
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("unsupported format: '{s}' (supported: json)")),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => f.write_str("json"),
        }
    }
}

#[derive(Args)]
struct DevArgs {
    /// Path to the .mical file
    file: PathBuf,

    /// Print the CST (concrete syntax tree)
    #[arg(long)]
    cst: bool,

    /// Print the AST (abstract syntax tree)
    #[arg(long)]
    ast: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval(args) => cmd_eval(args),
        Command::Dev(args) => cmd_dev(args),
    }
}

// ---------------------------------------------------------------------------
// eval
// ---------------------------------------------------------------------------

fn cmd_eval(args: EvalArgs) -> ExitCode {
    let source = match fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {e}", args.file.display());
            return ExitCode::FAILURE;
        }
    };

    let (green, syntax_errors) = mical_cli_parser::parse(mical_cli_lexer::tokenize(&source));
    let syntax_node = mical_cli_syntax::SyntaxNode::new_root(green);
    let source_file = match SourceFile::cast(syntax_node) {
        Some(sf) => sf,
        None => {
            eprintln!("error: failed to parse source file");
            return ExitCode::FAILURE;
        }
    };

    for err in &syntax_errors {
        eprintln!("syntax error: {err}");
    }

    let (config, config_errors) = mical_cli_config::Config::from_source_file(source_file);
    for err in &config_errors {
        eprintln!("config error: {err}");
    }

    let json_output = match (&args.query.get, &args.query.prefix) {
        (Some(key), None) => {
            let values: Vec<_> = config.query(key).map(|v| v.to_json()).collect();
            match values.len() {
                0 => serde_json::Value::Null,
                1 => values.into_iter().next().unwrap(),
                _ => serde_json::Value::Array(values),
            }
        }
        (None, Some(prefix)) => {
            let mut map = serde_json::Map::new();
            let mut last_key: Option<String> = None;
            for (k, v) in config.query_prefix(prefix) {
                let json_val = v.to_json();
                match last_key.as_deref() {
                    Some(prev) if prev == k => {
                        // Duplicate key — promote to array
                        let entry = map.get_mut(k).unwrap();
                        match entry {
                            serde_json::Value::Array(arr) => arr.push(json_val),
                            other => {
                                let prev_val = std::mem::replace(other, serde_json::Value::Null);
                                *other = serde_json::Value::Array(vec![prev_val, json_val]);
                            }
                        }
                    }
                    _ => {
                        map.insert(k.to_owned(), json_val);
                    }
                }
                last_key = Some(k.to_owned());
            }
            serde_json::Value::Object(map)
        }
        (None, None) => config.to_json(),
        _ => unreachable!("clap ensures mutual exclusivity"),
    };

    let output_str = match args.format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&json_output).expect("JSON serialization failed")
        }
    };

    match args.output_path {
        Some(path) => {
            if let Err(e) = fs::write(&path, format!("{output_str}\n")) {
                eprintln!("error: cannot write to '{}': {e}", path.display());
                return ExitCode::FAILURE;
            }
        }
        None => {
            println!("{output_str}");
        }
    }

    if !syntax_errors.is_empty() || !config_errors.is_empty() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

// ---------------------------------------------------------------------------
// dev
// ---------------------------------------------------------------------------

fn cmd_dev(args: DevArgs) -> ExitCode {
    let source = match fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {e}", args.file.display());
            return ExitCode::FAILURE;
        }
    };

    let (green, syntax_errors) = mical_cli_parser::parse(mical_cli_lexer::tokenize(&source));
    let syntax_node = mical_cli_syntax::SyntaxNode::new_root(green);

    // Default: print both if neither flag is given
    let print_both = !args.cst && !args.ast;

    if args.cst || print_both {
        println!("=== CST ===");
        print_cst(&syntax_node, 0);
    }

    if args.ast || print_both {
        if args.cst || print_both {
            println!();
        }
        println!("=== AST ===");
        match SourceFile::cast(syntax_node.clone()) {
            Some(sf) => println!("{sf:#?}"),
            None => eprintln!("error: failed to cast to SourceFile"),
        }
    }

    if !syntax_errors.is_empty() {
        println!();
        println!("=== Syntax Errors ===");
        for err in &syntax_errors {
            println!("  {err}");
        }
    }

    ExitCode::SUCCESS
}

fn print_cst(node: &mical_cli_syntax::SyntaxNode, indent: usize) {
    let padding = "  ".repeat(indent);
    println!("{padding}{:?}@{:?}", node.kind(), node.text_range());
    for child in node.children_with_tokens() {
        match child {
            mical_cli_syntax::SyntaxElement::Node(n) => print_cst(&n, indent + 1),
            mical_cli_syntax::SyntaxElement::Token(t) => {
                let padding = "  ".repeat(indent + 1);
                println!("{padding}{:?}@{:?} {:?}", t.kind(), t.text_range(), t.text());
            }
        }
    }
}
