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
    Expr { eq:Eq}
}

fn main() {
    let v0 = Var::<MyRx>::new_var_item(1);
    let mut v1 = Var::new_var_item(1);
    let eq0 = Eq::new_eq_item(&v0, &v1);
    eq0.commit();

    println!("before set {}",v1.to_egglog());
    v1.set_num(4);
    println!("after set {}",v1.to_egglog());
    v1.stage();
    eq0.commit();

    MyRx::rx().to_dot(PathBuf::from_str("egraph").unwrap());
}

basic_rx_vt!(MyRx);
