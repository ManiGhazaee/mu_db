#![allow(dead_code)]

use std::{io::Result, time::Instant};

use crate::db::DataBase;

mod db;

fn main() -> Result<()> {
    let mut db = DataBase::new();

    let inst = Instant::now();

    db.write_at(0, "x\nx\n")?;
    db.write_at(6, "x\nx\n")?;
    db.write_at(0, "y\n")?;

    println!("{}", inst.elapsed().as_millis());

    Ok(())
}
