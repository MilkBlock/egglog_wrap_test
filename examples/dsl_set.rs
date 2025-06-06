use egglog_macros::egglog_ty;
// use egglog_wrapper::wrap::Sort;

#[egglog_ty]
enum Cons{
    Value { v : i64, con : Box<Cons>},
    End {}
}

#[egglog_ty]
struct VecCon{
    v : Vec<Cons>
}

#[egglog_ty]
enum Root{
    V {v : VecCon}
}

fn main(){
    let node1 = Cons::new_value(3, &Cons::<Rx>::new_end());
    let node2 = Cons::new_value(2, &node1);
    let mut node3 = Cons::new_value(1, &node2);
    let _root = Root::new_v(&VecCon::new(vec![&node3]));
    node3.set_v(5);
    {node2};
    Rx::singleton().to_dot("egraph.dot".into());
}