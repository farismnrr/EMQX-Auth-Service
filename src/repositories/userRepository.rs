use rocksdb::Error;
use rocksdb::DB;
use std::sync::Arc;

/// Simple user model used by the repository create function.
/// We only store `username`, `password` and `is_deleted` as requested.
pub struct User {
    pub username: String,
    pub password: String,
    pub is_deleted: bool,
}

impl User {
    pub fn new(username: impl Into<String>, password: impl Into<String>, is_deleted: bool) -> Self {
        User {
            username: username.into(),
            password: password.into(),
            is_deleted,
        }
    }
}

/// Create a new user record in RocksDB.
///
/// - `db`: shared `Arc<rocksdb::DB>` obtained from `init_rocksdb` or `RocksDb::get_db()`.
/// - `user`: the `User` to store.
///
/// The function stores the user under the key `user:{username}` and a simple JSON-like value
/// containing only `username`, `password`, and `is_deleted`.
/// Returns the underlying RocksDB `Error` on failure.
pub fn create_user(db: &Arc<DB>, user: &User) -> Result<(), Error> {
    let key = format!("user:{}", user.username);

    // Build a small JSON-like string manually to avoid adding serde as a dependency.
    let value = format!(
        r#"{{"username":"{}","password":"{}","is_deleted":{}}}"#,
        user.username,
        user.password,
        if user.is_deleted { "true" } else { "false" }
    );

    db.put(key.as_bytes(), value.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::rocksdb::init_rocksdb;
    use std::env;

    #[test]
    fn create_user_smoke() {
        let tmp = env::temp_dir().join("iotnet_test_user_repo");
        let _ = std::fs::remove_dir_all(&tmp);

        let db = init_rocksdb(&tmp).expect("open rocksdb");
        let user = User::new("u1", "Alice", Some("alice@example.com"));
        create_user(&db, &user).expect("create user");

        // read back to ensure it exists
        let key = format!("user:{}", user.id);
        let got = db.get(key.as_bytes()).expect("get");
        assert!(got.is_some());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
