extern crate zstd;

extern crate zip;
extern crate curl;
extern crate time;
extern crate clap;
extern crate rustc_serialize;

mod corpus;
mod bench;

use bench::time_fn;

use std::io;

struct BenchResult {
    name: String,

    duration_ns: u64,

    original: u64,
}

fn main() {
    let compressed = {
        let corpus = corpus::get_corpus("data").unwrap();
        compress_corpus(&corpus).unwrap()
    };

    let mut results = bench_corpus(&compressed).unwrap();
    let avg = average(&results);
    results.push(avg);
    serialize_results(&results);
}

fn serialize_results(results: &[BenchResult]) {
    println!("Name Speed");
    for result in results {
        let speed = 1000.0 * result.original as f64 / result.duration_ns as f64;
        println!("{} {}", &result.name, speed);
    }
}

fn average(values: &[BenchResult]) -> BenchResult {
    let result = values.iter().fold((0,0), |(duration, original), value| {
        (duration + value.duration_ns,
         original + value.original)
    });

    let (duration, original) = result;

    BenchResult {
        name: String::from("total"),
        duration_ns: duration,
        original: original,
    }
}

fn compress_corpus(corpus: &corpus::Corpus) -> io::Result<corpus::Corpus> {
    Ok(corpus::Corpus {
        entries: corpus.entries
                       .iter()
                       .map(|entry: &corpus::CorpusEntry| {
                           let content = zstd::encode_all(&entry.content, 1).unwrap();
                           corpus::CorpusEntry {
                               name: entry.name.clone(),
                               content: content,
                           }
                       })
                       .collect(),
    })
}


fn bench_corpus(corpus: &corpus::Corpus) -> io::Result<Vec<BenchResult>> {
    corpus.entries.iter().map(bench_entry).collect()
}

fn bench_entry(entry: &corpus::CorpusEntry) -> io::Result<BenchResult> {

    let (duration, result) = try!(time_fn(|| zstd::decode_all(&entry.content)));

    let original = result.len();

    Ok(BenchResult {
        name: entry.name.clone(),
        duration_ns: duration,
        original: original as u64,
    })
}
