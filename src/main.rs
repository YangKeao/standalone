extern crate syntax;
extern crate clap;

use clap::{Arg, App};

use std::path::Path;
use syntax::ast::{Crate, NodeId};
use syntax::parse::{self, ParseSess};
use syntax::source_map;
use syntax::visit;
use syntax::ast;
use syntax::visit::Visitor;

struct Checker {
    path: Vec<String>,
    mod_path: Vec<String>,
}

impl Checker {
    pub fn new(mod_path: Vec<String>) -> Checker{
        Checker {
            path: Vec::new(),
            mod_path: mod_path
        }
    }
}

impl<'a> Visitor<'a> for Checker {
    fn visit_item(&mut self, item: &'a ast::Item) {
        match item.node {
            ast::ItemKind::Mod(_) => {
                let name = item.ident.to_string();
                self.path.push(name);

                let mod_len = self.mod_path.len();
                let path_len = self.path.len();

                let min_len = std::cmp::min(mod_len, path_len);
                if self.path[0..min_len] == self.mod_path[0..min_len] {
                    visit::walk_item(self, item);
                }
                let _ = self.path.pop();
            }
            _ => {
                visit::walk_item(self, item);
            }
        }
    }

    fn visit_mac(&mut self, mac: &'a ast::Mac) {
        visit::walk_mac(self, mac);
    }

    // Check path only cannot detect `super::super::some::{super::super::some2}`. We will
    // visit them as 2 `super::super`. However the count of super should be 3.
    fn visit_path(&mut self, path: &'a ast::Path, _id: NodeId) {
        let mod_string = self.mod_path.join("::");
        if self.path.len() >= self.mod_path.len() {
            let mut from_root = false;

            let mut super_count = 0;

            for seg in path.segments.iter() {
                let seg_str = seg.ident.as_str();

                if seg_str == "crate" {
                    from_root = true;
                }

                if seg_str == "super" {
                    super_count += 1;
                }
            }
            
            let path_string = path.segments.iter().map(|item| {
                item.ident.as_str().get().to_owned()
            }).collect::<Vec<String>>().join("::");

            if super_count > self.path.len() - self.mod_path.len() {
                println!("{} {}", self.path.join("::"), path_string)
            }
            if from_root {
                if path_string[7..].len() < mod_string.len() || path_string[7..7+mod_string.len()] != mod_string {
                    println!("{} {}", self.path.join("::"), path_string)
                }
            }
        }
        visit::walk_path(self, path);
    }
}

fn main() {
    let matches = App::new("Standalone Checker")
        .version("1.0")
        .bin_name("cargo")
        .author("Keao Yang <keao.yang@yahoo.com>")
        .subcommand(clap::SubCommand::with_name("standalone")
            .arg(Arg::with_name("mod")
                .short("m")
                .long("mod")
                .value_name("MOD")
                .takes_value(true))
            .arg(Arg::with_name("entry")
                .short("e")
                .long("entry")
                .value_name("ENTRY")
                .takes_value(true)))
        .get_matches();

    let sub_match = matches.subcommand_matches("standalone").unwrap();
    let mod_path = sub_match.value_of("mod").unwrap_or("").split("::").map(|item| item.to_owned()).collect::<Vec<String>>();
    let entry = sub_match.value_of("entry").unwrap_or("./src/lib.rs");

    let parse_session = ParseSess::new(source_map::FilePathMapping::empty());

    syntax::with_globals(|| {
        let krate: Crate = 
            match parse::parse_crate_from_file(Path::new(entry).as_ref(), &parse_session) {
                Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
                Ok(krate) => Ok(krate),
                Err(e) => Err(Some(e))
            }.map_err(|e| println!("{:?}", e)).unwrap();

        let mut checker = Checker::new(mod_path);
        checker.visit_mod(&krate.module,
                          krate.span,
                          &krate.attrs[..],
                          NodeId::from(0_usize));
    })
}
