use typst::syntax::{LinkedNode, Source, SyntaxKind, SyntaxNode};

use crate::console_log;

trait VerboseLinkedNode<'b> {
    fn prev_sibling_verbose(&self) -> Option<LinkedNode<'b>>;
    // fn next_sibling(&self) -> Option<LinkedNode>;
}


impl<'b> VerboseLinkedNode<'b> for LinkedNode<'b> {
    fn prev_sibling_verbose(&self) -> Option<LinkedNode<'b>> {
        let parent = self.parent()?;
        let index = self.index().checked_sub(1)?;
        parent.children().nth(index)
    }
}

/// Returns the number of new lines a node contains.
/// Is only relevant for `Space` and `Parbreak` nodes.
pub fn new_lines(node: &LinkedNode) -> usize {
    match node.kind() {
        SyntaxKind::Space | SyntaxKind::Parbreak => node.text().matches('\n').count(),
        _ => 0,
    }
}

/// Collects all previous nodes of a given node that match the specified kinds.
/// It stops collecting when it encounters a `Space` or `Parbreak` node with more than `max_nl` new lines.
pub fn collect_prev_nodes<'b>(
    node: &'b LinkedNode<'b>,
    kinds: &[SyntaxKind],
    max_nl: usize,
) -> Vec<LinkedNode<'b>> {
    let mut prev_nodes = Vec::new();
    let mut current_node: Option<LinkedNode<'b>> = node.prev_sibling_verbose();

    while let Some(n) = current_node {
        console_log!("Collecting previous node: {:?}", n);
        match n.kind() {
            SyntaxKind::Space | SyntaxKind::Parbreak => {
                if new_lines(&n) > max_nl {
                    break;
                }
            }
            _ => {}
        }
        current_node = n.prev_sibling_verbose();
        if kinds.contains(&n.kind()) {
            prev_nodes.push(n);
        }
    }
    prev_nodes.reverse();
    prev_nodes
}

/// Checks if a node is a `LetBinding` or an `Ident` that is a child of a `LetBinding`.
/// Returns the `LetBinding` node if found.
pub fn is_let_binding<'b>(node: &'b LinkedNode<'b>) -> Option<&'b LinkedNode<'b>> {
    if node.kind() == SyntaxKind::Ident
    {
        if let Some(parent) = node.parent() {
            if parent.kind() == SyntaxKind::LetBinding {
                return Some(&parent);
            } else if parent.kind() == SyntaxKind::Closure {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == SyntaxKind::LetBinding {
                        return Some(&grandparent);
                    }
                }
            }
        }
        None
    } else if node.kind() == SyntaxKind::LetBinding {
        Some(node)
    } else {
        None
    }
}

/// Finds a child node of a given node by its kind.
pub fn child_by_kind<'b>(node: &'b LinkedNode<'b>, kind: SyntaxKind) -> Option<LinkedNode<'b>> {
    node.children().find(|child| child.kind() == kind)
}

pub fn children_by_kind<'b>(node: &'b LinkedNode<'b>, kinds: &[SyntaxKind]) -> Vec<LinkedNode<'b>> {
    node.children()
        .filter(|child| kinds.contains(&child.kind()))
        .collect()
}

pub fn extract_text(nodes: &[LinkedNode]) -> Vec<String> {
    nodes.iter().map(|node| node.text().to_string()).collect()
}

pub fn parse_let_binding<'b>(
    node: &'b LinkedNode<'b>,
) -> Option<((String, Vec<String>), Vec<(String, Vec<String>)>)> {
    if let Some(binding) = is_let_binding(node) {
        let possible_binding_docs = collect_prev_nodes(binding, &[SyntaxKind::LineComment], 1);
        let (name, args) = if let Some(closure) = child_by_kind(binding, SyntaxKind::Closure) {
            let name =
                child_by_kind(&closure, SyntaxKind::Ident).and_then(|n| Some(n.text().to_string()))?;
            let mut ret = Vec::new();
            if let Some(params) = child_by_kind(&closure, SyntaxKind::Params) {
                    let args = children_by_kind(&params, &[SyntaxKind::Ident, SyntaxKind::Named]);
                    if !args.is_empty() {
                        ret = args.iter()
                                .map(|n| {
                                    let txt = n.text().to_string();
                                    let possible_arg_docs =
                                        collect_prev_nodes(n, &[SyntaxKind::LineComment], 1);
                                    let possible_arg_docs_text = extract_text(&possible_arg_docs);

                                    (txt, possible_arg_docs_text)
                                })
                                .collect::<Vec<(String, Vec<String>)>>();
                    }
                }
            (name, ret)
        } else {
            let name =
                child_by_kind(binding, SyntaxKind::Ident).and_then(|n| Some(n.text().to_string()))?;
            (name, Vec::new())
        };

        Some(((name, extract_text(&possible_binding_docs)), args))
    } else {
        None
    }
}

fn recursive_print_ast(node: &SyntaxNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let mut debug_text = node.text().clone();
    if debug_text.is_empty() {
        debug_text = node.clone().into_text();
    }
    console_log!(
        "{}Node: {:?}, Kind: {:?}",
        indent_str,
        debug_text,
        node.kind()
    );

    for child in node.children() {
        recursive_print_ast(&child, indent + 1);
    }
}

pub fn debug_print_ast(source: Source) {
    let root = source.root();
    console_log!("AST Debug Print:");
    recursive_print_ast(&root, 0);
    console_log!("End of AST Debug Print");
}
