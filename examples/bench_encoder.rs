extern crate zstd;

extern crate zip;
extern crate curl;
extern crate time;
extern crate clap;
extern crate rustc_serialize;

mod corpus;
mod bench;

use std::io;
use std::str::FromStr;

use clap::{Arg, App};

use rustc_serialize::json;

use bench::time_fn;

enum Output {
    Json,
    Gnuplot,
}

struct Settings {
    output: Output,
    levels: Vec<i32>,
    all: bool,
}

fn read_settings() -> Settings {

    let matches = App::new("bench_encoder")
                      .version(env!("CARGO_PKG_VERSION"))
                      .author("Alexandre Bury <alexandre.bury@gmail.com>")
                      .about("A benchmark for the zstd-rs encoder")
                      .arg(Arg::with_name("LEVEL")
                               .help("Compression level (default: 1). Can be a comma-separated \
                                      list (1,5,6) or a range (1-6).")
                               .short("l")
                               .long("level")
                               .takes_value(true))
                      .arg(Arg::with_name("json")
                               .help("Outputs result as a JSON array")
                               .long("json"))
                      .arg(Arg::with_name("all")
                               .help("Outputs the result for each file")
                               .long("all"))
                      .get_matches();

    let output = if matches.is_present("json") {
        Output::Json
    } else {
        Output::Gnuplot
    };

    // Read compression level from CLI args
    let levels = match matches.value_of("LEVEL") {
        None => vec![1],
        Some(ref levels) if levels.contains(",") => {
            levels.split(",")
                  .map(|level| i32::from_str(level).expect("invalid level"))
                  .collect()
        }
        Some(ref levels) if levels.contains("-") => {
            let tokens: Vec<_> = levels.split("-").collect();
            if tokens.len() != 2 {
                panic!("invalid level range");
            }
            let start = i32::from_str(tokens[0]).expect("invalid level");
            let end = i32::from_str(tokens[1]).expect("invalid level");

            (start..end + 1).collect()
        }
        Some(ref level) => {
            let level = i32::from_str(level).expect("invalid level");
            vec![level]
        }

    };

    Settings {
        output: output,
        levels: levels,
        all: matches.is_present("all"),
    }
}

#[derive(RustcEncodable)]
struct BenchResult {
    /// Compression level
    level: i32,

    /// Name of the file used
    name: String,

    /// Duration in ns
    duration_ns: u64,

    /// Original size
    original: u64,

    /// Compressed size
    compressed: u64,
}


fn serialize_results(results: &[BenchResult], output: Output) {
    match output {
        Output::Gnuplot => {
            println!("Level Speed Ratio");
            for result in results {
                let speed = 1000.0 * result.original as f64 / result.duration_ns as f64;
                let ratio = result.original as f64 / result.compressed as f64;
                println!("{} {} {}", result.level, speed, ratio);
            }
        }
        Output::Json => println!("{}", json::encode(&results).unwrap()),
    }
}


fn main() {

    // Read CLI args
    let settings = read_settings();

    // Get the data corpus
    let corpus = corpus::get_corpus("data").unwrap();

    // Get all the benchmark results
    let all_results: Vec<_> = settings.levels
                                  .iter()
                                  .map(|&level| bench_corpus(&corpus, level).unwrap())
                                  .collect();

    if !settings.all {
        // Just get the average for each level
        let results: Vec<_> = all_results.iter().map(|r| average(&*r)).collect();
        serialize_results(&results, settings.output);

    } else {
        // They want everything...
    }
}

fn average(values: &[BenchResult]) -> BenchResult {
    let result = values.iter().fold((0, 0, 0), |(duration, original, compressed), value| {
        (duration + value.duration_ns,
         original + value.original,
         compressed + value.compressed)
    });
    let (duration, original, compressed) = result;

    BenchResult {
        level: values[0].level,
        name: String::from("total"),
        duration_ns: duration,
        original: original,
        compressed: compressed,
    }

}

fn bench_corpus(corpus: &corpus::Corpus, level: i32) -> io::Result<Vec<BenchResult>> {
    corpus.entries.iter().map(|entry| bench_entry(entry, level)).collect()
}


/// Run in-memory compression benchmark
fn bench_entry(entry: &corpus::CorpusEntry, level: i32) -> io::Result<BenchResult> {
    let (duration, result) = try!(time_fn(|| {
        zstd::encode_all(&entry.content, level)
    }));

    let compressed = result.len();

    Ok(BenchResult {
        name: entry.name.clone(),
        original: entry.content.len() as u64,
        compressed: compressed as u64,
        duration_ns: duration,
        level: level,
    })
}
