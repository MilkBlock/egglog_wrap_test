use std::{borrow::{Borrow, Cow},  path::PathBuf, str::FromStr, sync::Mutex};
use std::sync::OnceLock;

use egglog::*;
use egglog_wrapper::{collect_type_defs, wrap::LetStmtRx, Offset, Point, Points};
fn main() {
    let p1 = Point::<Rx>::new_fixed_point(&Offset::new_d_vec2(1.0, 0.));
    let p2 = Point::new_fixed_point(&Offset::new_d_vec2(1.0, 2.));
    let ps = Points::new(vec![&p1,&p2]);
    Rx::singleton().to_dot(PathBuf::from("egraph"));
}



pub struct RxInner{ egraph: EGraph }
pub struct Rx{inner:Mutex<RxInner>}
impl Rx{
    fn interpret(&self,s:String){
        let mut guard = self.inner.lock().unwrap();
        guard.egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    fn to_dot(&self,file_name:PathBuf){
        let mut guard = self.inner.lock().unwrap();
        let mut serialized = guard.egraph.serialize(SerializeConfig::default());
        // if args.serialize_split_primitive_outputs {
        //     serialized.split_classes(|id, _| egraph.from_node_id(id).is_primitive())
        // }
        // for _ in 0..args.serialize_n_inline_leaves {
        //     serialized.inline_leaves();
        // }

        // if we are splitting primitive outputs, add `-split` to the end of the file name
        // let serialize_filename = if args.serialize_split_primitive_outputs {
        //     input.with_file_name(format!(
        //         "{}-split",
        //         input.file_stem().unwrap().to_str().unwrap()
        //     ))
        // } else {
        //     input.clone()
        // };
        let dot_path = file_name.with_extension("dot");
        serialized
            .to_dot_file(dot_path.clone())
            .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    }
}
unsafe impl Send for Rx{ }
unsafe impl Sync for Rx{ }
impl LetStmtRx for Rx{
    fn receive(received:String) {
        println!("{}",received);
        Self::singleton().interpret(received);
    }
    
    fn singleton() -> &'static Self {
        static INSTANCE: OnceLock<Rx> = OnceLock::new();
        INSTANCE.get_or_init(||{
            Rx{
                inner: Mutex::new(RxInner{
                    egraph: {
                        let mut e = EGraph::default();
                        let type_defs = collect_type_defs();
                        e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
                        e
                    },
                })
            }
        })
    }
}

