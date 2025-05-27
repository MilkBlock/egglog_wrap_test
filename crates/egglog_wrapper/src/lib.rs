use std::fmt::Display;
use derive_more::with_trait::Deref;

use egglog_macros::ToEgglog;
pub use derive_more;


pub trait LetStmtRx{
    fn receive(received:String);
}

pub trait ToEgglog{
    const SORT_DEF:Sort;
}
pub struct Sort(pub &'static str);

inventory::collect!(Sort);

// Type aliases for Vec types
#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
struct VecCtl {v:Vec<Ctl>}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
struct VecWF {v:Vec<WeightedFn>}


#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
struct VecHitBox {v:Vec<HitBox>}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
struct Points { v:Vec<Point>}

#[allow(unused)]
// Main types
#[derive(Debug, Clone,ToEgglog)]
enum Ctl {
    Para{vec_ctl:VecCtl},
    Seq{vec_ctl:VecCtl},
    Await{ctl:Box<Ctl>},
    Atom{anim_atom:AnimAtom},
}
#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum AnimAtom {
    Anim {
        object: BRabject,
        path: Path,
        duration: Duration,
        rate_cfg: RateCfg,
    },
    ConstructAnim {
        from: BRabject,
        to: BRabject,
        path: Path,
        duration: Duration,
        rate_cfg: RateCfg,
    },
    DestructAnim {
        from: BRabject,
        to: BRabject,
        path: Path,
        duration: Duration,
        rate_cfg: RateCfg,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum BRabject {
    ColoredShape {
        shape: Shape,
        color: Color,
    },
    Text {
        position: Point,
        content: String,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Color {
    Srgba {
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Shape {
    Polygon {
        points: Points,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Duration {
    DurationBySecs {
        seconds: f64,
    },
    DurationByMili {
        milliseconds: f64,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum BezierPathBuilder {
    Quadratic {
        control: Point,
        end: Point,
        rest: Box<BezierPathBuilder>,
    },
    Cubic {
        control1: Point,
        control2: Point,
        end: Point,
        rest: Box<BezierPathBuilder>,
    },
    LineTo {
        to: Point,
        rest: Box<BezierPathBuilder>,
    },
    Start {
        at: Point,
        rest: Box<BezierPathBuilder>,
    },
    PathEnd{},
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Offset {
    DVec3 {
        x: f64,
        y: f64,
        z: f64,
    },
    DVec2 {
        x: f64,
        y: f64,
    },
}



#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Point {
    FixedPoint {
        offset: Offset,
    },
    OffsetPoint {
        offset: Offset,
        base: Box<Point>,
    },
    CurAnchorOf {
        object: Box<BRabject>,
    },
    PointAtIdx {
        shape: Shape,
        index: i64,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Weight {
    W {
        value: f64,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum BuiltinF {
    Lerp{},
    Stay{},
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Fn {
    Builtin {
        function: BuiltinF,
    },
    WasmGuestExtern {
        name: String,
    },
}
#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum WeightedFn {
    WF{f:Fn, w:Weight},  // 作为元组字段
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum RateCfg{
    RateFn {
        wfs : VecWF
    }
}



#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum Path{
    BezierPath {
        bezier_path_builder:BezierPathBuilder
    }
}

#[allow(unused)]
#[derive(Debug, Clone, ToEgglog)]
enum HitBox{
    ShapedBox {
        shape:Shape
    },
    HitBoxs {
        histboxs: VecHitBox
    }
}

pub fn collect_type_defs() -> String{
    let mut s = "".to_owned();
    for sort in inventory::iter::<Sort>{
        s.push_str(sort.0);
    }
    format!("(set-option interactive_mode 1)(datatype* {} )", s)
}