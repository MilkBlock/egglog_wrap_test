use egglog_macros::egglog_ty;
use egglog_wrapper::basic_tx_no_vt;

#[egglog_ty]
enum Cons {
    Value { v: i64, con: Box<Cons> },
    End {},
}

#[egglog_ty]
struct VecCon {
    v: Vec<Cons>,
}

#[egglog_ty]
enum Root {
    V { v: VecCon },
}

fn main() {
    let node1 = Cons::new_value(3, &Cons::<MyRx>::new_end());
    let mut node2 = Cons::new_value(2, &node1);
    let node3 = Cons::new_value(1, &node2);
    let root = Root::new_v(&VecCon::new(vec![&node1, &node2, &node3]));
    node2.set_v(5);
    MyRx::tx().to_dot("egraph.dot".into());
}

basic_tx_no_vt!(MyRx);
