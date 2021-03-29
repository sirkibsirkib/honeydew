pub use {
    crate::{
        axes::{
            Dim::{self, *},
            DimMap,
            Direction::{self, *},
            Sign::{self, *},
            SignMap,
        },
        game::{Pos, Vel},
        rng::Rng,
        wrap_int::WrapInt,
    },
    core::{
        cmp::Ordering,
        f32::consts::PI as PI_F32,
        ops::{Index, IndexMut, Range},
        time::Duration,
    },
    gfx_2020::{glam::Vec2Swizzles, ClearColor, Mat4, Vec2, Vec3},
    ordered_float::OrderedFloat,
    serde::{Deserialize, Serialize},
    std::array::IntoIter as ArrIter,
};
pub fn unspecified_sock_addr() -> std::net::SocketAddrV4 {
    std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 0)
}
