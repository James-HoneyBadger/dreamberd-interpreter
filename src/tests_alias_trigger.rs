// Tests for keyword aliasing and synthetic trigger builtin
use crate::processor::{tokenize, generate_syntax_tree};
use crate::interpreter::Interpreter;
use crate::alias::canonicalize_tokens;

#[test]
fn test_alias_creation_and_listing() {
    let code = "alias(\"function\", \"zz\")! list_aliases()!"; // call list afterward
    let mut tokens = tokenize("alias_list", code).expect("tokenize");
    canonicalize_tokens(&mut tokens);
    let statements = generate_syntax_tree("alias_list", tokens, code).expect("syntax tree");
    let mut interpreter = Interpreter::new("alias_list".to_string(), code.to_string());
    interpreter.interpret_code_statements(statements).expect("execute");
    // Ensure alias file now contains mapping (directly call list_aliases builtin)
    // Invoke list_aliases via a function call expression to get the map value
    let list_code = "list_aliases()!";
    let mut list_tokens = tokenize("alias_list2", list_code).expect("tokenize2");
    canonicalize_tokens(&mut list_tokens);
    let list_statements = generate_syntax_tree("alias_list2", list_tokens, list_code).expect("syntax tree2");
    interpreter.interpret_code_statements(list_statements).expect("exec list");
    // Directly inspect alias persistence through the alias module
    let map = crate::alias::list_aliases();
    assert!(map.get("zz").map(|v| v == "function").unwrap_or(false), "alias zz -> function should exist");
}

#[test]
fn test_trigger_after_event_executes_body() {
    let code = "after \"keydown:A\" { const x = 5! } trigger(\"keydown:A\")!";
    let mut tokens = tokenize("trigger_test", code).expect("tokenize");
    canonicalize_tokens(&mut tokens);
    let statements = generate_syntax_tree("trigger_test", tokens, code).expect("syntax tree");
    let mut interpreter = Interpreter::new("trigger_test".to_string(), code.to_string());
    interpreter.interpret_code_statements(statements).expect("execution");
    // Variable x should have been created by after body through synthetic trigger
    let mut found = false;
    for ns in interpreter.namespaces.iter() {
        if let Some(entry) = ns.get("x") {
            if let crate::builtin::NamespaceEntry::Variable(var) = entry { if var.value().is_some() { found = true; } }
        }
    }
    assert!(found, "variable x should be created by after statement body after trigger");
}
