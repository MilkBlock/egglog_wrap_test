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
}

fn main() {
    let eq0 = Eq::<MyRx>::new_eq_item(&Var::new_var_item(3), &Var::new_var_item(4));
    let eq1 = Eq::<MyRx>::new_eq_item(&Var::new_var_item(4), &Var::new_var_item(5));
    let eq2 = Eq::<MyRx>::new_eq_item(&Var::new_var_item(3), &Var::new_var_item(5));
    eq2.commit();
    eq1.commit();

    MyRx::rx().to_dot(PathBuf::from_str("egraph").unwrap());
}

basic_rx_vt!(MyRx);
