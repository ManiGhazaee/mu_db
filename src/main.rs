#![allow(dead_code)]

use std::{io::Result, time::Instant};

use toy_db::DataBase;

fn main() -> Result<()> {
    let mut db = DataBase::new("./test.db");


    // let mut str = String::new();

    let inst = Instant::now();
    // db.clear_all().unwrap();

    db.insert("1", "one".repeat(200000).as_str());
    // db.insert("2", "two".repeat(300000).as_str());
    // db.insert("3", "three".repeat(4000000).as_str());
    // db.remove("3");
    // db.insert("4", "four".repeat(50000).as_str());
    // db.insert("5", "five".repeat(800000).as_str());
    // db.insert("6", "six".repeat(900000).as_str());
    println!("{}micros", inst.elapsed().as_micros());

    // str.push_str(db.get("3").unwrap().as_str());
    // str.push_str(db.get("4").unwrap().as_str());
    // str.push_str(db.get("5").unwrap().as_str());
    // str.push_str(db.get("6").unwrap().as_str());

    let inst = Instant::now();
    db.shrink();
    println!("{}micros", inst.elapsed().as_micros());

    // let mut str_after = String::new();

    // str_after.push_str(db.get("3").unwrap().as_str());
    // str_after.push_str(db.get("4").unwrap().as_str());
    // str_after.push_str(db.get("5").unwrap().as_str());
    // str_after.push_str(db.get("6").unwrap().as_str());

    // assert_eq!(str, str_after);
    // println!("{:?}", x1);
    // println!("{:?}", x2);
    // println!("{:?}", x3);

    Ok(())
}
