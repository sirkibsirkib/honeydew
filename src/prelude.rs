pub use {
    crate::{
        basic::{
            iter_pairs_mut,
            Dim::{self, *},
            DimMap,
            Direction::{self, *},
            Sign::{self, *},
            SignMap,
        },
        wrap_fields::WrapInt,
    },
    core::{
        cmp::Ordering,
        f32::consts::PI as PI_F32,
        ops::{Index, IndexMut, Range},
    },
    gfx_2020::{glam::Vec2Swizzles, Mat4, Vec2, Vec3},
    ordered_float::OrderedFloat,
    std::collections::HashSet,
};

macro_rules! dim_map {
    ($func:expr) => {{
        DimMap {
            arr: [
                {
                    let idx = 0;
                    $func
                },
                {
                    let idx = 1;
                    $func
                },
            ],
        }
    }};
}
