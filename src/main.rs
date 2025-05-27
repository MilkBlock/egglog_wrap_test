use std::{borrow::{Borrow, Cow}, path::PathBuf, str::FromStr};

use egglog::*;
use egglog_wrapper::{collect_type_defs, wrap::LetStmtRx, Offset, Point};
fn main() {
    let type_defs = collect_type_defs();

    println!("type_defs:{:?}",type_defs);
    let mut egraph =  EGraph::default();
    let v = egraph.parser.get_program_from_string(None, &type_defs.as_str()).unwrap();
    let rst = egraph.run_program(v);
    println!("rst:{:?}",rst);

    let a = Point::<Rx>::new_fixed_point(&Offset::new_d_vec2(1.0, 0.));

    
    // let egraph =  egraph.serialize(SerializeConfig::default());
    // let dot_path = PathBuf::from_str("./a").unwrap().with_extension("dot");
    // egraph.to_dot_file(dot_path.clone())
    //     .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    // egraph.add_arcsort(arcsort, span)
}


pub struct Rx{
    egraph: EGraph
}
impl LetStmtRx for Rx{
    fn receive(received:String) {
        println!("{}",received)
    }
}
