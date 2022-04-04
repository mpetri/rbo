// Note: this requires the `derive` feature

use std::io::BufRead;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Persistence value p where 0 <= p < 1.0
    #[clap(short, default_value_t = 0.9)]
    persistence: f64,

    /// first ranked list (\n delimited items)
    first_ranked_list_file: std::path::PathBuf,

    /// second ranked list (\n delimited items)
    second_ranked_list_file: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let first = std::fs::File::open(args.first_ranked_list_file)?;
    let first = std::io::BufReader::new(first);
    let first = first.lines().collect::<Result<Vec<String>, _>>()?;

    let second = std::fs::File::open(args.second_ranked_list_file)?;
    let second = std::io::BufReader::new(second);
    let second = second.lines().collect::<Result<Vec<String>, _>>()?;

    let rbo_res = rbo::rbo(&first, &second, args.persistence)?;

    println!("{}", rbo_res);

    Ok(())
}
