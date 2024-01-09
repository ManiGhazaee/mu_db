use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Result, Seek, SeekFrom, Write},
    ops::Range,
    sync::{Arc, Mutex},
};

const TEST_FILE_PATH: &str = "./test.rsdb";

#[derive(Clone)]
pub struct Index {
    entries: Vec<IndexEntry>,
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl Index {
    pub fn new() -> Self {
        let mut index_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("index.rsdb")
            .unwrap();
        let mut index_string = String::new();
        index_file.read_to_string(&mut index_string).unwrap();
        let entries = Index::parse_index(index_string);
        Index {
            entries,
            writer: Arc::new(Mutex::new(BufWriter::new(index_file))),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn insert_entry(&mut self, entry_size: usize, key: &str) -> IndexEntry {
        // get entry if exists with index:
        let mut old_entry = (0, None);
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key == key {
                old_entry = (i, Some(entry.clone()));
                break;
            }
        }

        match old_entry.1 {
            Some(old) => {
                if old.size() < entry_size {
                    self.entries.remove(old_entry.0);
                    return self.alloc_entry(entry_size, key);
                } else {
                    let entry = IndexEntry {
                        key: key.to_string(),
                        range: old.range.start..old.range.start + entry_size,
                    };
                    self.entries[old_entry.0] = entry.clone();
                    self.write_index();
                    return entry;
                }
            }
            None => return self.alloc_entry(entry_size, key),
        }
    }
    pub fn alloc_entry(&mut self, entry_size: usize, key: &str) -> IndexEntry {
        // find a empty range that new entry will fit then allocate:
        if !self.is_empty() {
            if self.entries[0].range.start >= entry_size {
                let entry = IndexEntry {
                    key: key.to_string(),
                    range: 0..entry_size,
                };
                self.entries.insert(0, entry.clone());
                self.write_index();
                return entry;
            }
            for i in 0..self.entries.len() - 1 {
                if (self.entries[i + 1].range.start - self.entries[i].range.end) >= entry_size {
                    let bind = &self.entries[i];
                    let entry = IndexEntry {
                        key: key.to_string(),
                        range: bind.range.end..bind.range.end + entry_size,
                    };
                    self.entries.insert(i + 1, entry.clone());
                    self.write_index();
                    return entry;
                }
            }
        }

        // else if entry doesnt fit:
        let range_start = if let Some(e) = self.entries.last() {
            e.range.end
        } else {
            0
        };
        let entry = IndexEntry {
            key: key.to_string(),
            range: range_start..range_start + entry_size,
        };
        self.entries.push(entry.clone());
        self.write_index();
        return entry;
    }
    pub fn remove_entry(&mut self, key: &str) -> Option<IndexEntry> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key == key {
                let removed = self.entries.remove(i);
                self.write_index();
                return Some(removed);
            }
        }
        None
    }
    pub fn get_entry(&self, key: &str) -> Option<IndexEntry> {
        self.entries.iter().find(|i| i.key == key).cloned()
    }
    pub fn write_index(&mut self) {
        let string = Index::index_to_string(self);
        let mut binding = self.writer.lock().unwrap();
        let w = binding.get_mut();
        w.seek(SeekFrom::Start(0)).unwrap();
        w.set_len(0).unwrap();
        w.write_all(string.as_bytes()).unwrap();
    }
    pub fn index_to_string(index: &Index) -> String {
        let mut str = String::new();
        for i in index.entries.iter() {
            str.push_str(&i.key);
            str.push('=');
            let range = [i.range.start.to_string(), i.range.end.to_string()].join("_");
            str.push_str(&range);
            str.push('\n');
        }
        str
    }
    pub fn parse_index(file: String) -> Vec<IndexEntry> {
        if file.is_empty() {
            Vec::new()
        } else {
            let entries: Vec<IndexEntry> = file
                .trim_end()
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
            entries
        }
    }
    pub fn clear_all(&mut self) {
        self.entries.clear();
        self.writer.lock().unwrap().get_mut().set_len(0).unwrap();
    }
}

#[derive(Clone)]
pub struct IndexEntry {
    key: String,
    range: Range<usize>,
}

impl IndexEntry {
    pub fn size(&self) -> usize {
        self.range.end - self.range.start
    }
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

        let index = Index::new();

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
    pub fn insert(&mut self, key: &str, value: &str) {
        let value_len = value.len();
        let index_entry = self.index.insert_entry(value_len, &key);
        self.write_at(index_entry.range.start.try_into().unwrap(), value)
            .unwrap();
    }
    pub fn get(&mut self, key: &str) -> Option<String> {
        let index_entry = self.index.get_entry(&key);
        match index_entry {
            Some(e) => Some(
                self.read_at(e.range.start.try_into().unwrap(), e.size())
                    .unwrap(),
            ),
            None => None,
        }
    }
    pub fn clear_all(&mut self) -> Result<()> {
        self.reader
            .lock()
            .unwrap()
            .get_mut()
            .seek(SeekFrom::Start(0))?;
        self.reader.lock().unwrap().get_mut().set_len(0)?;

        self.writer
            .lock()
            .unwrap()
            .get_mut()
            .seek(SeekFrom::Start(0))?;
        self.writer.lock().unwrap().get_mut().set_len(0)?;

        self.index.clear_all();

        Ok(())
    }
    pub fn remove(&mut self, key: &str) {
        self.index.remove_entry(&key);
    }
}
