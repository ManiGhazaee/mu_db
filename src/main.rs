#![allow(dead_code)]

use std::{io::Result, time::Instant};

use crate::db::DataBase;

mod db;

fn main() -> Result<()> {
    let mut db = DataBase::new();

    let inst = Instant::now();

    db.insert("x1", "yyyyyy");
    let x1 = db.get("x1");
    db.insert("x2", "xxxxxxxx");
    let x2 = db.get("x2");
    db.insert("x3", "zzzzzzzzzzzzzz");
    let x3 = db.get("x3");
    db.remove("x3");
    db.insert("x4", "uuuuuuuuu");
    db.remove("x1");
    db.insert("x5", "iii");
    db.remove("key");
    db.insert("new_key", "bluh");

    println!("{}micros", inst.elapsed().as_micros());
    println!("{:?}", x1);
    println!("{:?}", x2);
    println!("{:?}", x3);

    Ok(())
}
