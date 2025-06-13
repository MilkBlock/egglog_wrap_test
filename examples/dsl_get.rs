use std::{
    sync::OnceLock,
    thread::{self, JoinHandle},
    time::Duration,
};

use egglog_macros::{egglog_func, egglog_ty};
use egglog_wrapper::{basic_rx_no_vt, collect_string_type_defs, rx::RxNoVT};
// use egglog_wrapper::wrap::Sort;

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

#[egglog_func(output= Root)]
struct F {}

fn main() {
    let node1 = Cons::new_value(3, &Cons::<MyRx>::new_end());
    let mut node2 = Cons::new_value(2, &node1);
    let node3 = Cons::new_value(1, &node2);
    let _root = Root::new_v(&VecCon::new(vec![&node2]));
    println!("node2's current version is {}", node2.cur_sym());
    node2.set_v(4);
    println!("node2's current version is {}", node2.cur_sym());
    let root = Root::new_v(&VecCon::new(vec![&node3]));
    node2.set_v(6);
    println!("node2's current version is {}", node2.cur_sym());
    F::set((), &root);
    MyRx::rx().to_dot("egraph.dot".into());
}

basic_rx_no_vt!(MyRx);
