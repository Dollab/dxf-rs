// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
extern crate serde;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{
    BufWriter,
    Write,
};
use dxf::Drawing;

fn main() {
    let args: Vec<String> = env::args().collect();
    let dxf_path = &args[1];
    let mut json_path = dxf_path.clone();
    json_path.push_str(".json");

    let drawing = Drawing::load_file(&dxf_path).unwrap();
    let json = serde_json::to_string(&drawing).unwrap();

    let file = File::create(&json_path).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(json.as_bytes()).unwrap();
}