use std::{path::PathBuf, str::FromStr};

use egglog_macros::egglog_ty;
use egglog_wrapper::basic_rx_vt;

#[egglog_ty]
enum Eq {
    EqItem { v1: Var, v2: Var },
}
#[egglog_ty]
enum Var {
    VarItem { num: i64 },
    Expr { eq: Eq },
}

fn main() {
    let mut v0 = Var::<MyRx>::new_var_item(1);
    let mut v1 = Var::new_var_item(1);
    let mut eq0 = Eq::new_eq_item(&v0, &v1);
    eq0.commit();
    MyRx::rx().to_dot(PathBuf::from_str("egraph0").unwrap());

    v1.set_num(4).stage();
    eq0.commit();
    println!("version of root is {}", eq0.cur_sym());
    MyRx::rx().to_dot(PathBuf::from_str("egraph1").unwrap());

    v0.set_num(4).stage();
    eq0.commit();
    eq0.locate_latest();
    println!("new version of root is {}", eq0.cur_sym());
    MyRx::rx().to_dot(PathBuf::from_str("egraph2").unwrap());

    eq0.locate_prev();
    println!("prev version of root is {}", eq0.cur_sym());
}

basic_rx_vt!(MyRx);
