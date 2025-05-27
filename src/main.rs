use std::{borrow::{Borrow, Cow}, path::PathBuf, str::FromStr};

use egglog::*;
use egglog_wrapper::{collect_type_defs, BRabjectNode, BRabjectSym, LetStmtRx, OffsetNode, OffsetSym, PointNode, PointsNode, ShapeNode};
fn main() {
    let type_defs = collect_type_defs();

    println!("type_defs:{:?}",type_defs);
    let mut egraph =  EGraph::default();
    let v = egraph.parser.get_program_from_string(None, &type_defs.as_str()).unwrap();
    let rst = egraph.run_program(v);
    println!("rst:{:?}",rst);
    
    let p = PointNode::new_fixed_point::<Rx>(&OffsetNode::new_d_vec2::<Rx>(0., 0.));
    // let st:&str = p.as_str();
    // // println!("{}",p.to_egglog());
    // println!("{}", st);
    // let a = [1,2];
    // // a.iter().collect::<Vec<_>>();

    // Rx::receive("aaa".to_string());

    // for c in v.unwrap(){
    //     println!("{:?}",c)
    // }
    // println!("{:?}",v);
    // let rst  = egraph.parse_and_run_program(None, program).unwrap();
    // println!("{:?}",rst);
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