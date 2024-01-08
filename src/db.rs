use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read, Result, Seek, SeekFrom, Write},
    ops::Range,
    sync::{Arc, Mutex},
};

const TEST_FILE_PATH: &str = "./test.rsdb";

#[derive(Clone)]
pub struct Index {
    entries: Vec<IndexEntry>,
}

impl Index {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn insert_entry(&mut self, entry_size: usize, key: String) -> IndexEntry {
        for i in 0..self.entries.len() - 1 {
            if (self.entries[i + 1].range.start - self.entries[i].range.end) >= entry_size {
                let bind = &self.entries[i];
                let entry = IndexEntry {
                    key,
                    range: bind.range.end..bind.range.end + entry_size,
                };
                self.entries.insert(i + 1, entry.clone());
                return entry;
            }
        }

        let range_start = if let Some(e) = self.entries.last() {
            e.range.end
        } else {
            0
        };
        let entry = IndexEntry {
            key,
            range: range_start..range_start + entry_size,
        };
        self.entries.push(entry.clone());
        return entry;
    }
    pub fn remove_entry(&mut self, key: String) -> Option<IndexEntry> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key == key {
                return Some(self.entries.remove(i));
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct IndexEntry {
    key: String,
    range: Range<usize>,
}

pub struct DataBase {
    index: Index,
    reader: Arc<Mutex<BufReader<File>>>,
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl DataBase {
    pub fn new() -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(TEST_FILE_PATH)
            .unwrap();

        let file_clone = file.try_clone().unwrap();

        let index_file = fs::read_to_string("index.rsdb").unwrap_or(String::new());
        let index = DataBase::parse_index(index_file);

        DataBase {
            index,
            reader: Arc::new(Mutex::new(BufReader::new(file))),
            writer: Arc::new(Mutex::new(BufWriter::new(file_clone))),
        }
    }
    pub fn read_at(&mut self, start: u64, size: usize) -> Result<String> {
        let mut v = vec![0; size];
        let mut br = self.reader.lock().unwrap();
        br.seek(SeekFrom::Start(start))?;
        br.read_exact(&mut v)?;
        Ok(String::from_utf8_lossy(&v).into())
    }
    pub fn write_at(&mut self, start: u64, content: &str) -> Result<()> {
        let mut bw = self.writer.lock().unwrap();
        bw.seek(SeekFrom::Start(start))?;
        bw.write_all(content.as_bytes())?;
        bw.flush()?;
        Ok(())
    }
    pub fn parse_index(file: String) -> Index {
        if file.is_empty() {
            Index {
                entries: Vec::new(),
            }
        } else {
            let entries: Vec<IndexEntry> = file
                .split("\n")
                .map(|i| {
                    let entry: Vec<&str> = i.split("=").collect();
                    let range: Vec<&str> = entry[1].split("_").collect();
                    let range: Range<usize> = Range {
                        start: range[0].parse().unwrap(),
                        end: range[1].parse().unwrap(),
                    };
                    IndexEntry {
                        key: entry[0].to_string(),
                        range,
                    }
                })
                .collect();
            Index { entries }
        }
    }
}
