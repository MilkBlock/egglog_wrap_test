use egglog_macros::egglog_ty;
use egglog_wrapper::basic_rx_vt;

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
    let node1 = Cons::new_value(3, &Cons::<MyRx>::new_end());
    let mut node2 = Cons::new_value(2, &node1);
    let node3 = Cons::new_value(1, &node2);
    let root = Root::new_v(&VecCon::new(vec![&node2]));
    println!("node2's current version is {}",node2.as_str());
    let mut old_version = root.clone();
    node2.set_v(4);
    println!("node2's current version is {}",node2.as_str());
    node2.set_v(6);
    println!("node2's current version is {}",node2.as_str());
    MyRx::rx().interpret("(function Latest () Root :no-merge)".to_owned());
    MyRx::rx().interpret("(function OldVersion () Root :no-merge)".to_owned());
    MyRx::rx().interpret(format!("(set (OldVersion) {})", old_version.as_str()).to_owned());
    old_version.locate_latest();
    let new_version = old_version;
    MyRx::rx().interpret(format!("(set (Latest) {})", new_version.as_str()).to_owned());
    MyRx::rx().to_dot("egraph.dot".into());
}

basic_rx_vt!(MyRx);