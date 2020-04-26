use bindgen::builder;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::process::Command;

fn run_bindgen(path: &str) {
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

fn read_all(path: &str) -> String {
    let mut in_file = File::open(path).expect(&format!("Failed to open {}", path));
    let mut content = String::new();
    in_file
        .read_to_string(&mut content)
        .expect(&format!("Failed to read {}", path));
    content
}

fn generate_code() {
    let content = read_all("/tmp/output.rs");
    let defs = collect_type_defs(&content);

    let header = "#![feature(alloc_layout_extra)]\n";
    let lib = read_all("./data/boilerplate.rs");

    let mut main_func = String::new();
    main_func += "fn main() {\n";

    // Add code to check structs
    for def in defs.structs.iter() {
        if def.name.starts_with("_") {
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
        if def.name.starts_with("_") {
            continue;
        }

        let mut valid_fields = Vec::<String>::new();
        for f in def.fields.iter() {
            if !f.starts_with("_") {
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

    let mut out_file = File::create("/tmp/generated.rs").unwrap();

    // write the `lorem_ipsum` string to `file`, returns `io::result<()>`
    out_file.write_all(header.as_bytes()).unwrap();
    out_file.write_all(content.as_bytes()).unwrap();
    out_file.write_all(lib.as_bytes()).unwrap();
    out_file.write_all(main_func.as_bytes()).unwrap();
}

fn exec_code(rs_path: &str) {
    let status = Command::new("rustc")
        .args(&[rs_path, "-o", "/tmp/generated.exe", "-A", "warnings"])
        .status()
        .expect("failed to execute process");
    assert!(status.success());

    let output = Command::new("/tmp/generated.exe")
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

fn main() {
    run_bindgen("./examples/simple.h");
    generate_code();
    exec_code("/tmp/generated.rs");
}
