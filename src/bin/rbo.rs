// Note: this requires the `derive` feature

use std::io::BufRead;

const HELP: &str = "\
rbo
Rank-Biased Overlap (RBO): a similarity measure for indefinite ranked lists. see

@article{wmz10:acmtois,
    author = \"Webber, William and Moffat, Alistair and Zobel, Justin\",
    title = \"A similarity measure for indefinite rankings\",
    journal = \"ACM Transactions on Information Systems\",
    year = {2010},
}

for details.

USAGE:
    rbo [-p] <FIRST_RANKED_LIST_FILE> <SECOND_RANKED_LIST_FILE>

ARGS:
    <FIRST_RANKED_LIST_FILE>     first ranked list 
    <SECOND_RANKED_LIST_FILE>    second ranked list 

OPTIONS:
    -p <PERSISTENCE>        Persistence value p where 0 <= p < 1.0 [default: 0.9]
";

#[derive(Debug)]
struct AppArgs {
    p: f64,
    first_ranked_list_file: std::path::PathBuf,
    second_ranked_list_file: std::path::PathBuf,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let args = AppArgs {
        // Parses a required value that implements `FromStr`.
        // Returns an error if not present.
        p: pargs.opt_value_from_str("-p")?.unwrap_or(0.9),
        // Parses an optional value from `&OsStr` using a specified function.
        first_ranked_list_file: pargs.free_from_str()?,
        // Parses a required free-standing/positional argument.
        second_ranked_list_file: pargs.free_from_str()?,
    };
    Ok(args)
}

fn main() -> anyhow::Result<()> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let first = std::fs::File::open(args.first_ranked_list_file)?;
    let first = std::io::BufReader::new(first);
    let first = first.lines().collect::<Result<Vec<String>, _>>()?;

    let second = std::fs::File::open(args.second_ranked_list_file)?;
    let second = std::io::BufReader::new(second);
    let second = second.lines().collect::<Result<Vec<String>, _>>()?;

    let rbo_res = rbo::rbo(&first, &second, args.p)?;

    println!("{}", rbo_res);

    Ok(())
}
