 # muDB 

 - DataBase is a simple, lightweight database that provides basic database functionalities, and can be created using the new function, which takes a path to the database file as an argument.
 - The database supports basic operations such as inserting key-value pairs, retrieving values, removing entries, and clearing all data.
 - It also offers advanced features like direct read/write operations at specified positions, checking if the database or buffer is empty, and optimizing the database file by removing unused space.

 ## Example

 ```rust
 let mut db = mu_db::DataBase::new("./test.db");
 // This will generate ./test.db and ./index_test.db if they don't exist.

 db.insert("key", "before_value");
 db.insert("key", "after_value");

 let value = db.get("key");
 assert_eq!(value, Some("after_value".to_string()));

 db.remove("key");

 assert_eq!(db.get("key"), None);
 assert!(db.is_empty()); // index is empty
 assert!(!db.is_buf_empty()); // db is not empty
 assert_eq!(db.buf_len(), 12); // db: `after_valuee`

 db.shrink(); // remove unused space
 assert!(db.is_buf_empty());

 db.write_at(5, "world").unwrap(); // write to db file directly without syncing index
 let data = db.read_at(5, 5).unwrap(); // read db file directly

 assert_eq!(data, "world".to_string());

 db.clear_all().unwrap(); // clear everything (index and db)

 assert!(db.is_empty());
 assert!(db.is_buf_empty());
 ```

 Please note that the mu_db is a simple, lightweight database and does not support complex database operations like transactions, joins, etc. It is best suited for simple key-value storage needs.
