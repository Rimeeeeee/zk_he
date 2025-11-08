use rocksdb::{DB, Options};
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pub db: Arc<rocksdb::DB>,
}

impl Database {
    pub fn init(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).expect("failed to open RocksDB");
        Database { db: Arc::new(db) }
    }

    pub fn put(&self, key: &str, value: &[u8]) {
        self.db.put(key.as_bytes(), value).unwrap();
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.db.get(key.as_bytes()).unwrap()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.db.get(key.as_bytes()).unwrap().is_some()
    }

    pub fn scan_prefix(&self, prefix: &str) -> Vec<(String, Vec<u8>)> {
        self.db
            .iterator(rocksdb::IteratorMode::Start)
            .filter_map(|item| {
                let (key, val) = item.ok()?;
                let key_str = String::from_utf8(key.to_vec()).ok()?;
                if key_str.starts_with(prefix) {
                    Some((key_str, val.to_vec()))
                } else {
                    None
                }
            })
            .collect()
    }
}
