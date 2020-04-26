#[macro_use]
extern crate clap;

use std::fs::{self, File};
use std::io::prelude::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use ansi_term::Colour::Red;
use bindgen::builder;
use clap::App;

fn run_bindgen(path: &Path, outdir: Option<&str>) -> PathBuf {
    let bindings = builder()
        .header(path.to_str().unwrap())
        .ignore_functions()
        .layout_tests(false)
        .derive_default(true)
        .generate()
        .expect("failed to generate bindings");

    let mut out_file = match outdir {
        Some(dir) => Path::new(dir).to_path_buf(),
        None => std::env::temp_dir(),
    };
    out_file.push("bindings.rs");

    bindings
        .write_to_file(&out_file)
        .expect("failed to write to file");

    out_file
}

#[derive(Debug)]
struct StructDef {
    name: String,
    fields: Vec<String>,
}

#[derive(Debug)]
struct UnionDef {
    name: String,
    fields: Vec<String>,
}

struct TypeDefs {
    structs: Vec<StructDef>,
    unions: Vec<UnionDef>,
}

fn collect_type_defs(content: &str) -> TypeDefs {
    let ast = syn::parse_file(content).expect("Failed to construct an AST.");
    let mut structs = Vec::<StructDef>::new();
    let mut unions = Vec::<UnionDef>::new();

    for ast_item in ast.items.iter() {
        match ast_item {
            syn::Item::Struct(item) => {
                let mut fields = Vec::<String>::new();
                for f in item.fields.iter() {
                    match &f.ident {
                        Some(ident) => fields.push(ident.to_string()),
                        None => {
                            println!("ignore an unnamed field in struct {}", item.ident);
                        }
                    }
                }
                structs.push(StructDef {
                    name: item.ident.to_string(),
                    fields,
                });
            }
            syn::Item::Union(item) => {
                let mut fields = Vec::<String>::new();
                for f in item.fields.named.iter() {
                    match &f.ident {
                        Some(ident) => {
                            fields.push(ident.to_string());
                        }
                        None => {
                            println!("ignore an unnamed field in union {}", item.ident);
                        }
                    }
                }
                unions.push(UnionDef {
                    name: item.ident.to_string(),
                    fields,
                });
            }
            _ => {
                continue;
            }
        };
    }

    TypeDefs { structs, unions }
}

fn read_all(path: &Path) -> String {
    let mut in_file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {:?}", path));
    let mut content = String::new();
    in_file
        .read_to_string(&mut content)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", path));
    content
}

fn generate_code(bindings_rs: &Path) -> PathBuf {
    let content = read_all(bindings_rs);
    let defs = collect_type_defs(&content);

    let header = "#![feature(alloc_layout_extra)]\n";
    let lib = read_all(Path::new("./data/boilerplate.rs"));

    let mut main_func = String::new();
    main_func += "\nfn main() {\n";

    // Add code to check structs
    for def in defs.structs.iter() {
        if def.name.starts_with('_') {
            continue;
        }
        main_func += &format!(
            "    check_struct!({}, {});\n",
            def.name,
            def.fields.join(", ")
        );
    }

    // Add code to check unions
    for def in defs.unions.iter() {
        if def.name.starts_with('_') {
            continue;
        }

        let mut valid_fields = Vec::<String>::new();
        for f in def.fields.iter() {
            if !f.starts_with('_') {
                valid_fields.push(f.to_string());
            }
        }
        main_func += &format!(
            "    check_union!({}, {});\n",
            def.name,
            valid_fields.join(", ")
        );
    }

    main_func += "}\n";

    let mut out_path = bindings_rs.parent().unwrap().to_path_buf();
    out_path.push("generated.rs");
    let mut out_file = File::create(&out_path).unwrap();

    out_file.write_all(header.as_bytes()).unwrap();
    out_file.write_all(content.as_bytes()).unwrap();
    out_file.write_all(lib.as_bytes()).unwrap();
    out_file.write_all(main_func.as_bytes()).unwrap();

    out_path
}

fn exec_code(rs_path: &Path) {
    let mut exe_path = rs_path.parent().unwrap().to_path_buf();
    exe_path.push("generated.out");

    let status = Command::new("rustc")
        .args(&[
            rs_path.to_str().unwrap(),
            "-o",
            exe_path.to_str().unwrap(),
            "-A",
            "warnings",
        ])
        .status()
        .expect("failed to execute process");
    assert!(status.success());

    let raw_output = Command::new(&exe_path)
        .output()
        .expect("failed to execute process");
    let mut output = String::from_utf8(raw_output.stdout).unwrap();
    output = output.replace("Found:", &Red.paint(" Found:").to_string());
    print!("{}", output);
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    let input = Path::new(matches.value_of("INPUT").unwrap());
    let outdir = match matches.value_of("OUTDIR") {
        Some(dir) => {
            if let Err(e) = fs::create_dir_all(dir) {
                println!("Failed to create an output directory {}: {}", dir, e);
                return;
            }
            Some(dir)
        }
        None => None,
    };

    let bindings_rs = run_bindgen(input, outdir);
    let generated_rs = generate_code(&bindings_rs);
    exec_code(&generated_rs);
}
