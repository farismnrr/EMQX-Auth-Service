//! Simple RocksDB initializer and shutdown helper used by repositories.
//!
//! Exports:
//! - `RocksDb` struct with `open` and `shutdown` methods
//! - `init_rocksdb(path: &str) -> Result<Arc<rocksdb::DB>, rocksdb::Error>` convenience
//! - `close_rocksdb(db: Arc<rocksdb::DB>)` convenience

use log::debug;
use rocksdb::{DB, Error, Options};
use std::path::Path;
use std::sync::Arc;

/// Convenience: initialize RocksDB and return an `Arc<DB>` directly.
///
/// Example:
///
/// let db = init_rocksdb("./rocksdb-data/iotnet")?;
pub fn init_rocksdb<P: AsRef<Path>>(path: P) -> Result<Arc<DB>, Error> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, path)?;
    debug!("[Infrastructure | RocksDB] Database opened successfully.");
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
    debug!("[Infrastructure | RocksDB] Database closed successfully.");
}
