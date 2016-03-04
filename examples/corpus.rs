use std::io::{self, Read, Write};
use std::fs;

use curl::http;
use zip;
use zip::result::ZipResult;

pub struct CorpusEntry {
    pub name: String,
    pub content: Vec<u8>,
}

pub struct Corpus {
    pub entries: Vec<CorpusEntry>,
}

/// Get the corpus.
///
/// Reads from disk if present.
/// Downloads it (and cache it to disk) otherwise.
pub fn get_corpus(dir: &str) -> ZipResult<Corpus> {
    match fs::metadata(dir) {
        Ok(ref metadata) if metadata.is_dir() => Ok(try!(read_corpus(dir))),
        Err(_) => fetch_corpus(dir),
        Ok(_) => panic!("`{}` already exists as a file", dir),
    }
}

/// Reads the corpus from the disk cache.
pub fn read_corpus(dir: &str) -> io::Result<Corpus> {
    let mut entries = Vec::new();

    for entry in try!(fs::read_dir(dir)) {
        let entry = try!(entry);
        let name = entry.file_name().to_str().unwrap().to_owned();
        let mut file = try!(fs::File::open(&entry.path()));
        let mut buffer = Vec::with_capacity(try!(entry.metadata()).len() as usize);
        try!(file.read_to_end(&mut buffer));

        entries.push(CorpusEntry {
            name: name,
            content: buffer,
        });
    }

    Ok(Corpus { entries: entries })
}

/// Downloads the corpus and store it to disk.
pub fn fetch_corpus(dir: &str) -> ZipResult<Corpus> {
    println!("Downloading archive...");
    let resp = http::handle()
                   .get("http://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip")
                   .exec()
                   .unwrap();

    // Download the zip file to memory
    println!("Buffering...");
    let mut buffer = Vec::new();
    try!(resp.get_body().read_to_end(&mut buffer));

    // Extract it
    println!("Beginning extraction.");
    let mut archive = try!(zip::ZipArchive::new(io::Cursor::new(&buffer)));

    let mut entries = Vec::new();

    try!(fs::create_dir_all(dir));

    for i in 0..archive.len() {
        let mut segment = try!(archive.by_index(i));
        // Now save it to memory
        let mut entry_content = Vec::with_capacity(segment.size() as usize);
        try!(segment.read_to_end(&mut entry_content));

        // Also dump it to disk
        println!("Saving {}", segment.name());
        let mut target = try!(fs::File::create(format!("{}/{}", dir, segment.name())));
        try!(target.write_all(&entry_content));

        entries.push(CorpusEntry {
            name: segment.name().to_owned(),
            content: entry_content,
        });
    }

    Ok(Corpus { entries: entries })
}
