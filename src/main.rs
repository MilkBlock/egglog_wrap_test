use egglog::{EGraph, TermDag, Value};

fn main() {
    let mut egraph = EGraph::default();
    egraph
        .parse_and_run_program(
            None,
            "(datatype Op (Add i64 i64))
            (let expr (Add 1 1))",
        )
        .unwrap();
    let mut termdag = TermDag::default();
    let (sort, value) = egraph.eval_expr(&egglog::var!("expr")).unwrap();
    let (_, _extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
    println!("{:?}", termdag);
}
