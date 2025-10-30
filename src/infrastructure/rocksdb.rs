//! Simple RocksDB initializer and shutdown helper used by repositories.
//!
//! Exports:
//! - `RocksDb` struct with `open` and `shutdown` methods
//! - `init_rocksdb(path: &str) -> Result<Arc<rocksdb::DB>, rocksdb::Error>` convenience
//! - `close_rocksdb(db: Arc<rocksdb::DB>)` convenience

use rocksdb::{Options, DB, Error};
use std::path::Path;
use std::sync::Arc;

/// Lightweight wrapper around a RocksDB instance.
///
/// Repositories can either keep an `Arc<rocksdb::DB>` obtained from `RocksDb::open(...).get_db()`
/// or use the convenience `init_rocksdb` function below.
pub struct RocksDb {
    db: Arc<DB>,
}

impl RocksDb {
    /// Open or create a RocksDB instance at `path`.
    ///
    /// Returns a wrapper containing an `Arc<DB>` so it can be cloned and shared across threads.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path)?;
        Ok(RocksDb { db: Arc::new(db) })
    }

    /// Get a cloneable handle to the underlying `rocksdb::DB`.
    pub fn get_db(&self) -> Arc<DB> {
        Arc::clone(&self.db)
    }

    /// Consume this wrapper and attempt to close the DB immediately.
    ///
    /// If other `Arc` clones exist elsewhere, RocksDB will remain open until the last clone
    /// is dropped. We try to `Arc::try_unwrap` to drop the DB now; if that fails we simply
    /// return since the DB will close later when clones are dropped.
    pub fn shutdown(self) {
        // try to take ownership of DB to drop it immediately
        if let Ok(db) = Arc::try_unwrap(self.db) {
            drop(db);
        }
    }
}

/// Convenience: initialize RocksDB and return an `Arc<DB>` directly.
///
/// Example:
///
/// let db = init_rocksdb("./rocksdb-data/iotnet")?;
pub fn init_rocksdb<P: AsRef<Path>>(path: P) -> Result<Arc<DB>, Error> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, path)?;
    Ok(Arc::new(db))
}

/// Convenience: attempt to close a DB held in an `Arc`.
///
/// Returns `Ok(())` if the DB was dropped immediately or will be closed later when
/// the last clone is dropped. No error is returned if other clones exist; callers
/// should ensure they drop their clones when they want the DB to close.
pub fn close_rocksdb(db: Arc<DB>) {
    // try to unwrap and drop now; otherwise let it close when last clone is dropped
    if let Ok(db_inner) = Arc::try_unwrap(db) {
        drop(db_inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn open_and_close() {
        let tmp = env::temp_dir().join("iotnet_test_rocksdb");
        // ignore errors on cleanup; this is a simple smoke test
        let _ = std::fs::remove_dir_all(&tmp);

        let client = RocksDb::open(&tmp).expect("open rocksdb");
        let handle = client.get_db();
        assert!(handle.path().exists());

        client.shutdown();
        // after shutdown the files may still exist until last clone is dropped
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
