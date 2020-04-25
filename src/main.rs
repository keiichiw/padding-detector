extern crate proc_macro;

use bindgen::builder;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::process::Command;

fn generate_rs_file(path: &str) {
    let bindings = builder()
        .header(path)
        .ignore_functions()
        .layout_tests(false)
        .derive_default(true)
        .generate()
        .expect("failed to generate bindings");

    bindings
        .write_to_file("/tmp/output.rs")
        .expect("failed to write to file");
}

#[derive(Debug)]
struct StructDef {
    name: String,
    fields: Vec<String>,
}

fn get_struct_defs(content: &str) -> Result<Vec<StructDef>, Box<dyn Error>> {
    let ast = syn::parse_file(content)?;
    let mut defs = Vec::<StructDef>::new();

    for ast_item in ast.items.iter() {
        let item = match ast_item {
            syn::Item::Struct(item) => item,
            _ => {
                continue;
            }
        };

        let mut fields = Vec::<String>::new();
        for f in item.fields.iter() {
            match &f.ident {
                Some(ident) => fields.push(ident.to_string()),
                None => panic!("unnamed field in {}", item.ident.to_string()),
            }
        }

        defs.push(StructDef {
            name: item.ident.to_string(),
            fields,
        });
    }

    Ok(defs)
}

fn generate_snippet(def: &StructDef) -> String {
    let mut s = String::new();
    s += &format!("println!(\"Checking `struct {}`...\");\n", def.name);
    s += &format!("let x: {} = Default::default();\n", def.name);
    s += "let mut l = std::alloc::Layout::from_size_align(0, 1).unwrap();\n";
    for f in def.fields.iter() {
        s += &format!("l = extend_layout(&l, \"{}\", &x.{});\n", f, f);
    }
    s += r###"
        let pad = l.padding_needed_for(l.align());
        if pad != 0 {
            println!("{}-byte padding at the end", pad);
        }
        l = l.pad_to_align();
        assert_eq!(l.size(), std::mem::size_of_val(&x));
"###;

    s
}

fn generate_code() {
    let mut in_file = File::open("/tmp/output.rs").unwrap();
    let mut content = String::new();
    in_file.read_to_string(&mut content).unwrap();

    let structs = get_struct_defs(&content).unwrap();

    let header = "#![feature(alloc_layout_extra)]\n";
    let lib = r###"
fn extend_layout<T>(l: &std::alloc::Layout, name: &str, v: &T) -> std::alloc::Layout {
    let (new_l, offset) = l.extend(std::alloc::Layout::for_value(v)).expect("x");
    if offset != l.size() {
        println!(
            "{}-byte padding before \"{}\"",
            offset - l.size(),
            name
        );
    }
    new_l
}
"###;

    let mut main_func = String::new();
    main_func += "fn main() {\n";
    for def in structs.iter() {
        main_func += "{\n";
        main_func += &generate_snippet(def);
        main_func += "}\n";
    }
    main_func += "}\n";

    let mut out_file = File::create("/tmp/generated.rs").unwrap();

    // write the `lorem_ipsum` string to `file`, returns `io::result<()>`
    out_file.write_all(header.as_bytes()).unwrap();
    out_file.write_all(content.as_bytes()).unwrap();
    out_file.write_all(lib.as_bytes()).unwrap();
    out_file.write_all(main_func.as_bytes()).unwrap();

    let status = Command::new("rustc")
        .args(&[
            "/tmp/generated.rs",
            "-o",
            "/tmp/generated.exe",
            "-A",
            "warnings",
        ])
        .status()
        .expect("failed to execute process");
    assert!(status.success());

    let output = Command::new("/tmp/generated.exe")
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

fn main() {
    generate_rs_file("./examples/simple.h");
    generate_code();
}
