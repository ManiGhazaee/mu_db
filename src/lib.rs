use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Result, Seek, SeekFrom, Write},
    ops::Range,
    path::Path,
    sync::{Arc, Mutex},
};

pub struct DataBase {
    index: Index,
    reader: Arc<Mutex<BufReader<File>>>,
    writer: Arc<Mutex<BufWriter<File>>>,
}

#[derive(Clone)]
pub struct Index {
    entries: Vec<IndexEntry>,
    writer: Arc<Mutex<BufWriter<File>>>,
}

#[derive(Clone)]
pub struct IndexEntry {
    key: String,
    range: Range<usize>,
}

impl DataBase {
    /// Creates a new instance of the database or uses the existing db file,
    /// at the given path.
    /// # Examples
    ///
    pub fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        let file_clone = file.try_clone().unwrap();

        let _path = Path::new(path);
        let db_file_name = _path.file_name().and_then(|i| i.to_str()).unwrap();
        let index_file_path = format!(
            "{}/{}",
            _path.parent().unwrap().to_str().unwrap(),
            format!("index_{}", db_file_name)
        );

        let index = Index::new(&index_file_path);

        DataBase {
            index,
            reader: Arc::new(Mutex::new(BufReader::new(file))),
            writer: Arc::new(Mutex::new(BufWriter::new(file_clone))),
        }
    }
    /// Reads data directly from the database file at the specified position (`start`) and size (`size`).
    pub fn read_at(&mut self, start: u64, size: usize) -> Result<String> {
        let mut v = vec![0; size];
        let mut br = self.reader.lock().unwrap();
        br.seek(SeekFrom::Start(start))?;
        br.read_exact(&mut v)?;
        Ok(String::from_utf8_lossy(&v).into())
    }
    /// Writes data directly to the database file at the specified position with any length.
    pub fn write_at(&mut self, start: u64, content: &str) -> Result<()> {
        let mut bw = self.writer.lock().unwrap();
        bw.seek(SeekFrom::Start(start))?;
        bw.write_all(content.as_bytes())?;
        bw.flush()?;
        Ok(())
    }
    /// Inserts a key-value pair into the database, replacing old value if key exists.
    pub fn insert(&mut self, key: &str, value: &str) {
        let value_len = value.len();
        let index_entry = self.index.insert_entry(value_len, &key);
        self.write_at(index_entry.range.start.try_into().unwrap(), value)
            .unwrap();
    }
    /// Retrieves the value associated with the given key from the database.
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
    /// Clears all data in the database.
    pub fn clear_all(&mut self) -> Result<()> {
        self.set_len(0);
        self.index.clear_all();

        Ok(())
    }
    /// Sets the length of the database file directly, truncating or extending it as necessary.
    pub fn set_len(&mut self, len: u64) {
        let mut binding_r = self.reader.lock().unwrap();
        let mut binding_w = self.writer.lock().unwrap();
        let r = binding_r.get_mut();
        let w = binding_w.get_mut();
        r.seek(SeekFrom::Start(0)).unwrap();
        w.seek(SeekFrom::Start(0)).unwrap();
        r.set_len(len).unwrap();
        w.set_len(len).unwrap();
    }
    /// Removes the entry associated with the given key from the index if the key exists.
    /// This method does not remove the value in the database file. To completely remove the value,
    /// you need to use (`.shrink()`) after removing the entry.
    pub fn remove(&mut self, key: &str) {
        self.index.remove_entry(&key);
    }
    /// Optimizes the database file by removing any unused space.
    pub fn shrink(&mut self) {
        if self.index.is_empty() {
            return;
        }

        let old_entries = self.index.shrink_entries();

        for (old, new) in old_entries.iter().zip(self.index.entries.clone()) {
            if old.range.start != new.range.start {
                let old_string = self
                    .read_at(old.range.start.try_into().unwrap(), old.size())
                    .unwrap();
                self.write_at(new.range.start.try_into().unwrap(), &old_string)
                    .unwrap();
            }
        }

        self.set_len(
            (self.index.entries.last().unwrap().range.end)
                .try_into()
                .unwrap(),
        );
    }
}

impl Index {
    pub fn new(path: &str) -> Self {
        let mut index_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
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
    pub fn get_all_entries(&self) -> Vec<IndexEntry> {
        self.entries.clone()
    }
    pub fn set_all_entries(&mut self, entries: Vec<IndexEntry>) {
        self.entries = entries;
        self.write_index();
    }
    /// Returns old `self.entries`
    pub fn shrink_entries(&mut self) -> Vec<IndexEntry> {
        let old = self.entries.clone();
        if old.is_empty() {
            return old;
        }

        let first = &mut self.entries[0].range;
        if first.start != 0 {
            first.end -= first.start;
            first.start = 0;
        }
        for i in 0..self.entries.len() - 1 {
            let curr = self.entries[i].range.clone();
            let next = &mut self.entries[i + 1].range;
            let diff = next.start - curr.end;
            if diff != 0 {
                next.end -= diff;
                next.start -= diff;
            }
        }

        self.write_index();
        return old;
    }
}

impl IndexEntry {
    pub fn size(&self) -> usize {
        self.range.end - self.range.start
    }
}
