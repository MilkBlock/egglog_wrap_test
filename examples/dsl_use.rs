use std::path::PathBuf;

use egglog_wrapper::{wrap::{LetStmtRx, Rx}, AnimAtom, BRabject,BRabjectInstance, BezierPathBuilder, BuiltinF, Color, Ctl, Duration, Fn, Offset, Path, Point, Points, RateCfg, Shape, VecCtl, VecWF, Weight, WeightedFn};

fn main() {
    // 创建点
    let p1 = Point::<Rx>::new_fixed_point(&Offset::new_d_vec2(1.0, 1.0));
    let p2 = Point::new_fixed_point(&Offset::new_d_vec2(1.0, 2.0));
    let p3 = Point::new_offset_point(&Offset::new_d_vec2(1.0, 2.0),&p2);

    // 创建点集合
    let points = Points::new(vec![&p1, &p2, &p3]);
    
    // 创建三角形形状
    let triangle_shape = Shape::new_polygon(&points);
    
    // 创建三角形对象（红色）
    let triangle = BRabject::new_colored_shape(
        &triangle_shape,
        &Color::new_srgba(1.0, 0.0, 0.0, 1.0)
    );
    let triangle_instance = BRabjectInstance::new_instance(&triangle);
    
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
        &triangle_instance,
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



