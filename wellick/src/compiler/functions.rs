use crate::parser::ast;
use std::collections::HashMap;

pub(crate) fn build_fn_map(nodes: &Vec<ast::FnDecl>) -> HashMap<String, ast::FnDecl> {
    nodes
        .iter()
        .map(|node| (node.name.clone(), node.clone()))
        .collect()
}
