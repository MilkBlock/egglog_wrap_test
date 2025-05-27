use egglog_macros::egglog_ty;
pub use derive_more;
pub mod wrap;

// Type aliases for Vec types
#[egglog_ty]
struct VecCtl {v:Vec<Ctl>}

#[allow(unused)]
#[egglog_ty]
struct VecWF {v:Vec<WeightedFn>}


#[allow(unused)]
#[egglog_ty]
struct VecHitBox {v:Vec<HitBox>}

#[allow(unused)]
#[egglog_ty]
struct Points { v:Vec<Point>}

#[allow(unused)]
#[egglog_ty]
enum Ctl {
    Para{vec_ctl:VecCtl},
    Seq{vec_ctl:VecCtl},
    Await{ctl:Box<Ctl>},
    Atom{anim_atom:AnimAtom},
}
#[allow(unused)]
#[egglog_ty]
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
#[egglog_ty]
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
#[egglog_ty]
enum Color {
    Srgba {
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    },
}

#[allow(unused)]
#[egglog_ty]
enum Shape {
    Polygon {
        points: Points,
    },
}

#[allow(unused)]
#[egglog_ty]
enum Duration {
    DurationBySecs {
        seconds: f64,
    },
    DurationByMili {
        milliseconds: f64,
    },
}

#[allow(unused)]
#[egglog_ty]
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
#[egglog_ty]
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
#[egglog_ty]
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
#[egglog_ty]
enum Weight {
    W {
        value: f64,
    },
}

#[allow(unused)]
#[egglog_ty]
enum BuiltinF {
    Lerp{},
    Stay{},
}

#[allow(unused)]
#[egglog_ty]
enum Fn {
    Builtin {
        function: BuiltinF,
    },
    WasmGuestExtern {
        name: String,
    },
}

#[allow(unused)]
#[egglog_ty]
enum WeightedFn {
    WF{f:Fn, w:Weight},  // 作为元组字段
}

#[allow(unused)]
#[egglog_ty]
enum RateCfg{
    RateFn {
        wfs : VecWF
    }
}



#[allow(unused)]
#[egglog_ty]
enum Path{
    BezierPath {
        bezier_path_builder:BezierPathBuilder
    }
}

#[allow(unused)]
#[egglog_ty]
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