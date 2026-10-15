//! Checks that every public method in `src/` is annotated with `no_panic` and
//! listed in `tests/no_panic.rs`, which only checks the routines it lists.

use std::collections::BTreeSet;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Attribute, ImplItem, Meta, Type, Visibility};

/// Public methods found in the library source, with whether each is annotated.
#[derive(Default)]
struct PublicMethods(Vec<(String, bool)>);

impl<'ast> Visit<'ast> for PublicMethods {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !has_attribute(&item.attrs, "cfg", "test") {
            visit::visit_item_mod(self, item);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let Type::Path(owner) = &*item.self_ty else {
            return;
        };
        let owner = &owner.path.segments.last().unwrap().ident;
        for item in &item.items {
            if let ImplItem::Fn(f) = item {
                if matches!(f.vis, Visibility::Public(_)) {
                    let annotated =
                        has_attribute(&f.attrs, "cfg_attr", r#"feature="no-panic",no_panic"#);
                    self.0
                        .push((format!("{owner}::{}", f.sig.ident), annotated));
                }
            }
        }
    }
}

/// `Type::routine` paths referenced by the driver.
#[derive(Default)]
struct ListedRoutines(BTreeSet<String>);

impl<'ast> Visit<'ast> for ListedRoutines {
    fn visit_expr_path(&mut self, expr: &'ast syn::ExprPath) {
        let segments: Vec<_> = expr
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        if let [.., owner, routine] = segments.as_slice() {
            self.0.insert(format!("{owner}::{routine}"));
        }
        visit::visit_expr_path(self, expr);
    }
}

/// True if an attribute is exactly `#[name(args)]`, ignoring whitespace.
fn has_attribute(attrs: &[Attribute], name: &str, args: &str) -> bool {
    attrs.iter().any(|attr| match &attr.meta {
        Meta::List(list) => {
            list.path.is_ident(name) && list.tokens.to_string().replace(' ', "") == args
        }
        _ => false,
    })
}

fn parse(path: &Path) -> syn::File {
    syn::parse_file(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_public_routine_is_annotated_and_listed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut methods = PublicMethods::default();
    for entry in std::fs::read_dir(root.join("src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            methods.visit_file(&parse(&path));
        }
    }
    assert!(!methods.0.is_empty(), "no public methods found");

    let mut listed = ListedRoutines::default();
    listed.visit_file(&parse(&root.join("tests/no_panic.rs")));

    let unannotated: Vec<_> = methods
        .0
        .iter()
        .filter(|(_, annotated)| !annotated)
        .map(|(name, _)| name)
        .collect();
    assert!(
        unannotated.is_empty(),
        "missing the no_panic attribute: {unannotated:?}"
    );

    let unlisted: Vec<_> = methods
        .0
        .iter()
        .map(|(name, _)| name)
        .filter(|name| !listed.0.contains(*name))
        .collect();
    assert!(
        unlisted.is_empty(),
        "not listed in tests/no_panic.rs: {unlisted:?}"
    );
}
