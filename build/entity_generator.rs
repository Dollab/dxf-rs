// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;

use ::{get_expected_type, get_reader_function};

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;

pub fn generate_entities() {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `src/build` directory.

use ::{CodePair, Color, Point, Vector};
use ::helper_functions::*;

use enums::*;
use enum_primitive::FromPrimitive;

use std::io;

// Used to turn Option<T> into io::Result.
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Some(v) => v,
            None => return Err(io::Error::new(io::ErrorKind::InvalidData, \"unexpected enum value\"))
        }
    )
}
".trim_left());
    fun.push_str("\n");
    generate_base_entity(&mut fun, &element);
    generate_entity_types(&mut fun, &element);

    fun.push_str("impl EntityType {\n");
    generate_from_type_string(&mut fun, &element);
    generate_try_apply_code_pair(&mut fun, &element);
    fun.push_str("}\n");

    let mut file = File::create("src/entities.rs").ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_base_entity(fun: &mut String, element: &Element) {
    let entity = &element.children[0];
    if name(&entity) != "Entity" { panic!("Expected first entity to be 'Entity'."); }
    fun.push_str("#[derive(Clone)]\n");
    fun.push_str("pub struct EntityCommon {\n");
    for c in &entity.children {
        let t = if allow_multiples(&c) { format!("Vec<{}>", typ(c)) } else { typ(c) };
        match c.name.as_str() {
            "Field" => {
                fun.push_str(format!("    pub {name}: {typ},\n", name=name(c), typ=t).as_str());
            },
            "Pointer" => {
                fun.push_str(format!("    // TODO: '{}' pointer here\n", name(c)).as_str());
            },
            "WriteOrder" => (),
            _ => panic!("unexpected element under Entity: {}", c.name),
        }
    }

    fun.push_str("}\n");
    fun.push_str("\n");

    fun.push_str("#[derive(Clone)]\n");
    fun.push_str("pub struct Entity {\n");
    fun.push_str("    pub common: EntityCommon,\n");
    fun.push_str("    pub specific: EntityType,\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    fun.push_str("impl Entity {\n");
    fun.push_str("    pub fn new(specific: EntityType) -> Self {\n");
    fun.push_str("        Entity {\n");
    fun.push_str("            common: EntityCommon::new(),\n");
    fun.push_str("            specific: specific,\n");
    fun.push_str("        }\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    fun.push_str("impl EntityCommon {\n");
    fun.push_str("    pub fn new() -> Self {\n");
    fun.push_str("        EntityCommon {\n");
    for c in &entity.children {
        match c.name.as_str() {
            "Field" => {
                fun.push_str(format!("            {name}: {val},\n", name=name(c), val=default_value(&c)).as_str());
            },
            "Pointer" => {
                fun.push_str(format!("            // TODO: '{}' pointer here\n", name(c)).as_str());
            },
            "WriteOrder" => (),
            _ => panic!("unexpected element under Entity: {}", c.name),
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");

    fun.push_str("    pub fn apply_individual_pair(&mut self, pair: &CodePair) -> io::Result<()> {\n");
    fun.push_str("        match pair.code {\n");
    for c in &entity.children {
        if c.name == "Field" { // TODO: support pointers
            let read_fun = if allow_multiples(&c) {
                format!(".push({})", get_field_reader(&c))
            }
            else {
                format!(" = {}", get_field_reader(&c))
            };
            fun.push_str(format!("            {code} => {{ self.{field}{read_fun} }},\n", code=code(c), field=name(c), read_fun=read_fun).as_str());
        }
    }

    fun.push_str("            _ => (), // unknown code, just ignore\n");
    fun.push_str("        }\n");
    fun.push_str("        Ok(())\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push_str("\n");
}

fn generate_entity_types(fun: &mut String, element: &Element) {
    fun.push_str("#[derive(Clone)]\n");
    fun.push_str("pub enum EntityType {\n");
    for c in &element.children {
        if c.name != "Entity" { panic!("expected top level entity"); }
        if name(c) != "Entity" && name(c) != "DimensionBase" && attr(&c, "BaseClass") != "DimensionBase" {
            // TODO: handle dimensions
            // TODO: handle complex subtypes: e.g., lwpolyline has vertices
            fun.push_str(format!("    {typ}({typ}),\n", typ=name(c)).as_str());
        }
    }

    fun.push_str("}\n");
    fun.push_str("\n");

    // individual structs
    for c in &element.children {
        if c.name != "Entity" { panic!("expected top level entity"); }
        if name(c) != "Entity" && name(c) != "DimensionBase" && attr(&c, "BaseClass") != "DimensionBase" {
            // TODO: handle dimensions
            // TODO: handle complex subtypes: e.g., lwpolyline has vertices

            // definition
            fun.push_str("#[derive(Clone)]\n");
            fun.push_str(format!("pub struct {typ} {{\n", typ=name(c)).as_str());
            for f in &c.children {
                let t = if allow_multiples(&f) { format!("Vec<{}>", typ(f)) } else { typ(f) };
                let acc = if attr(&f, "Accessibility") == "private" { "" } else { "pub " };
                match f.name.as_str() {
                    "Field" => {
                        fun.push_str(format!("    {acc}{name}: {typ},\n", acc=acc, name=name(f), typ=t).as_str());
                    },
                    "Pointer" => {
                        fun.push_str(format!("    // TODO: '{}' pointer here\n", name(f)).as_str());
                    },
                    "WriteOrder" => (), // TODO:
                    _ => panic!("unexpected element {} under Entity", f.name),
                }
            }

            fun.push_str("}\n");
            fun.push_str("\n");

            // implementation
            fun.push_str(format!("impl {typ} {{\n", typ=name(c)).as_str());
            fun.push_str("    pub fn new() -> Self {\n");
            fun.push_str(format!("        {typ} {{\n", typ=name(c)).as_str());
            for f in &c.children {
                match f.name.as_str() {
                    "Field" => {
                        fun.push_str(format!("            {name}: {val},\n", name=name(f), val=default_value(&f)).as_str());
                    },
                    "Pointer" => {
                        fun.push_str(format!("            // TODO: '{}' pointer here\n", name(f)).as_str());
                    },
                    "WriteOrder" => (), // TODO:
                    _ => panic!("unexpected element {} under Entity", f.name),
                }
            }

            fun.push_str("        }\n");
            fun.push_str("    }\n");

            // flags
            generate_flags_methods(fun, &c);

            fun.push_str("}\n");
            fun.push_str("\n");
        }
    }
}

fn generate_flags_methods(fun: &mut String, element: &Element) {
    for field in &element.children {
        if field.name == "Field" {
            for flag in &field.children {
                if flag.name == "Flag" {
                    let flag_name = name(&flag);
                    let mask = attr(&flag, "Mask");
                    fun.push_str(format!("    pub fn get_{name}(&self) -> bool {{\n", name=flag_name).as_str());
                    fun.push_str(format!("        self.{name} & {mask} != 0\n", name=name(&field), mask=mask).as_str());
                    fun.push_str("    }\n");
                    fun.push_str(format!("    pub fn set_{name}(&mut self, val: bool) {{\n", name=flag_name).as_str());
                    fun.push_str("        if val {\n");
                    fun.push_str(format!("            self.{name} |= {mask};\n", name=name(&field), mask=mask).as_str());
                    fun.push_str("        }\n");
                    fun.push_str("        else {\n");
                    fun.push_str(format!("            self.{name} &= !{mask};\n", name=name(&field), mask=mask).as_str());
                    fun.push_str("        }\n");
                    fun.push_str("    }\n");
                }
            }
        }
    }
}

fn generate_from_type_string(fun: &mut String, element: &Element) {
    fun.push_str("    pub fn from_type_string(type_string: &str) -> Option<EntityType> {\n");
    fun.push_str("        match type_string {\n");
    for c in &element.children {
        if name(c) != "Entity" && name(c) != "DimensionBase" && !attr(&c, "TypeString").is_empty() {
            let type_string = attr(&c, "TypeString");
            let type_strings = type_string.split(',').collect::<Vec<_>>();
            for t in type_strings {
                fun.push_str(format!("            \"{type_string}\" => Some(EntityType::{typ}({typ}::new())),\n", type_string=t, typ=name(c)).as_str());
            }
        }
    }

    fun.push_str("            _ => None,\n");
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_try_apply_code_pair(fun: &mut String, element: &Element) {
    fun.push_str("    pub fn try_apply_code_pair(&mut self, pair: &CodePair) -> io::Result<bool> {\n");
    fun.push_str("        match self {\n");
    for c in &element.children {
        if c.name != "Entity" { panic!("expected top level entity"); }
        if name(c) != "Entity" && name(c) != "DimensionBase" && attr(&c, "BaseClass") != "DimensionBase" {
            if generate_reader_function(&c) {
                // TODO: handle dimensions
                // TODO: handle complex subtypes: e.g., lwpolyline has vertices
                fun.push_str(format!("            &mut EntityType::{typ}(ref mut ent) => {{\n", typ=name(c)).as_str());
                fun.push_str("                match pair.code {\n");
                let mut seen_codes = HashSet::new();
                for f in &c.children {
                    if f.name == "Field" && generate_reader(&f) { // TODO: support pointers
                        for (i, &cd) in codes(&f).iter().enumerate() {
                            if !seen_codes.contains(&cd) {
                                seen_codes.insert(cd); // TODO: allow for duplicates
                                let reader = get_field_reader(&f);
                                let codes = codes(&f);
                                let write_cmd = match codes.len() {
                                    1 => {
                                        let read_fun = if allow_multiples(&f) {
                                            format!(".push({})", reader)
                                        }
                                        else {
                                            format!(" = {}", reader)
                                        };
                                        format!("ent.{field}{read_fun}", field=name(&f), read_fun=read_fun)
                                    },
                                    _ => {
                                        let suffix = match i {
                                            0 => "x",
                                            1 => "y",
                                            2 => "z",
                                            _ => panic!("impossible"),
                                        };
                                        format!("ent.{field}.{suffix} = {reader}", field=name(&f), suffix=suffix, reader=reader)
                                    }
                                };
                                fun.push_str(format!("                    {code} => {{ {cmd}; }},\n", code=cd, cmd=write_cmd).as_str());
                            }
                        }
                    }
                }

                fun.push_str("                    _ => return Ok(false),\n");
                fun.push_str("                }\n");
                fun.push_str("            },\n");
            }
            else {
                fun.push_str(format!("            &mut EntityType::{typ}(_) => {{ panic!(\"this case should have been covered in a custom reader\"); }},\n", typ=name(&c)).as_str());
            }
        }
    }

    fun.push_str("        }\n");
    fun.push_str("        return Ok(true);\n");
    fun.push_str("    }\n");
}

fn load_xml() -> Element {
    let file = File::open("spec/EntitiesSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}

fn attr(element: &Element, name: &str) -> String {
    match &element.attributes.get(name) {
        &Some(v) => v.clone(),
        &None => String::new(),
    }
}

fn allow_multiples(element: &Element) -> bool {
    attr(element, "AllowMultiples") == "true"
}

fn name(element: &Element) -> String {
    attr(element, "Name")
}

fn typ(element: &Element) -> String {
    attr(element, "Type")
}

fn code(element: &Element) -> i32 {
    attr(element, "Code").parse::<i32>().unwrap()
}

fn codes(element: &Element) -> Vec<i32> {
    let code_overrides = attr(&element, "CodeOverrides");
    if code_overrides.is_empty() {
        return vec![code(&element)];
    }
    else {
        return code_overrides.split(",").map(|c| c.parse::<i32>().unwrap()).collect::<Vec<_>>();
    }
}

fn get_field_reader(element: &Element) -> String {
    let expected_type = get_expected_type(code(&element)).ok().unwrap();
    let reader_fun = get_reader_function(&expected_type);
    let mut read_converter = attr(&element, "ReadConverter");
    if read_converter.is_empty() {
        read_converter = String::from("{}");
    }
    let read_cmd = format!("{reader}(&pair.value)", reader=reader_fun);
    read_converter.replace("{}", read_cmd.as_str())
}

fn generate_reader(element: &Element) -> bool {
    attr(&element, "GenerateReader") != "false"
}

fn generate_reader_function(element: &Element) -> bool {
    attr(&element, "GenerateReaderFunction") != "false"
}

fn default_value(element: &Element) -> String {
    attr(&element, "DefaultValue")
}