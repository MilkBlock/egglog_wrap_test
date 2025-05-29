use std::{borrow::{Borrow, Cow},  path::PathBuf, str::FromStr, sync::Mutex};
use std::sync::OnceLock;

use egglog::*;
use egglog_wrapper::{collect_type_defs, wrap::LetStmtRx, AnimAtom, BRabject, BezierPathBuilder, BuiltinF, Color, Ctl, Duration, Offset, Path, Point, Points, RateCfg, Shape, VecCtl, VecWF, Weight, WeightedFn, Fn};
// fn main() {
//     let p1 = Point::<Rx>::new_fixed_point(&Offset::new_d_vec2(1.0, 0.));
//     let p2 = Point::new_fixed_point(&Offset::new_d_vec2(1.0, 2.));
//     let ps = Points::new(vec![&p1,&p2]);
//     Shape::new_polygon(points)
//     Rx::singleton().to_dot(PathBuf::from("egraph"));
// }
// use your_crate_name::*; // 请替换为实际的crate名

fn main() {
    // 创建点
    let p1 = Point::<Rx>::new_fixed_point(&Offset::new_d_vec2(1.0, 1.0));
    let p2 = Point::new_fixed_point(&Offset::new_d_vec2(1.0, 2.0));
    let p3 = Point::new_fixed_point(&Offset::new_d_vec2(2.0, 2.0));
    
    // 创建点集合
    let points = Points::new(vec![&p1, &p2, &p3]);
    
    // 创建三角形形状
    let triangle_shape = Shape::new_polygon(&points);
    
    // 创建三角形对象（红色）
    let triangle = BRabject::new_colored_shape(
        &triangle_shape,
        &Color::new_srgba(1.0, 0.0, 0.0, 1.0)
    );
    
    // 创建当前锚点
    let cur_anchor = Point::new_cur_anchor_of(&triangle);
    
    // 创建目标点（基于当前锚点偏移）
    let target_point = Point::new_offset_point(
        &Offset::new_d_vec2(1.0, 1.0),
        &cur_anchor
    );
    
    // 构建路径
    let path_end = BezierPathBuilder::new_path_end();
    let line_to = BezierPathBuilder::new_line_to(
        &target_point,
        &path_end
    );
    let start = BezierPathBuilder::new_start(
        &cur_anchor,
        &line_to
    );
    let path = Path::new_bezier_path(&start);
    
    // 创建动画原子
    let anim_atom = AnimAtom::new_anim(
        &triangle,
        &path,
        &Duration::new_duration_by_secs(3.0),
        &RateCfg::new_rate_fn(&VecWF::new(vec![
            &WeightedFn::new_wf(
                &Fn::new_builtin(&BuiltinF::new_lerp()),
                &Weight::new_w(1.0))
        ]))
    );
    
    // 构建动画序列
    let atom = Ctl::new_atom(&anim_atom);
    let seq = Ctl::new_seq(&VecCtl::new(vec![&atom]));
    
    // 构建并行时间线
    let s = VecCtl::new(vec![&seq]);
    let s2 = VecCtl::new(vec![&seq,&seq]);
    let mut timeline = Ctl::new_para(&s);
    timeline.set_vec_ctl(&s2);
    
    // 输出到dot文件
    Rx::singleton().to_dot(PathBuf::from("timeline_egraph"));
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

