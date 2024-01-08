#![allow(dead_code)]

use std::{io::Result, time::Instant};

use crate::db::DataBase;

mod db;

fn main() -> Result<()> {
    let mut db = DataBase::new();

    let inst = Instant::now();

    // db.insert("key".to_string(), "hello world".to_string());
    // db.insert("bluh".to_string(), "hello flkjs".to_string());
    // db.insert("value".to_string(), "xxxxxxxxxxxxslkjfksjfworld".to_string());
    let x = db.get("key".to_string());

    println!("{}", inst.elapsed().as_millis());
    println!("{}", x.unwrap());

    Ok(())
}
