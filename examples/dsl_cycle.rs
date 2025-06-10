use std::path::{MAIN_SEPARATOR, PathBuf};

use egglog_macros::egglog_ty;
use egglog_wrapper::{Path, basic_rx_no_vt};

#[egglog_ty]
enum A {
    ACon { b: B },
}

#[egglog_ty]
enum B {
    BCon { a: A },
    Empty {},
}

fn main() {
    let mut a = A::new_a_con(&B::<MyRx>::new_empty());
    let b = B::<MyRx>::new_empty();
    a.set_b(&B::new_b_con(&a));
    MyRx::rx().to_dot(PathBuf::from("egraph"));
}

basic_rx_no_vt!(MyRx);
