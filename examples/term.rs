use egglog::{ast::{Parser, Symbol}, match_term_app, span, Term, TermDag};

fn main(){
    let s = r#"(f (g x y) x y (g x y))"#;
    let (td, t) = parse_term(s);
    match_term_app!(t; {
        ("f", [_, x, _, _]) => {
            let span = span!();
            assert_eq!(
                td.term_to_expr(td.get(*x), span.clone()),
                egglog::ast::GenericExpr::Var(span, Symbol::new("x"))
            )
        }
        (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
    })
}
fn parse_term(s: &str) -> (TermDag, Term) {
    let e = Parser::default().get_expr_from_string(None, s).unwrap();
    let mut td = TermDag::default();
    let t = td.expr_to_term(&e);
    (td, t)
}
