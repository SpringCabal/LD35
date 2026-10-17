use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::gl;
use spring_addons::ui::Widget;

use crate::state::Global;

struct Decal {
    texture: &'static str,
    spirit: bool,
    x1: f32,
    z1: f32,
    x2: f32,
    z2: f32,
}

const DECALS: &[Decal] = &[
    Decal {
        texture: "b_ok",
        spirit: false,
        x1: 899.9806,
        z1: 1729.8711,
        x2: 946.3828,
        z2: 1772.4429,
    },
    Decal {
        texture: "b_ok",
        spirit: false,
        x1: 1003.2117,
        z1: 1711.6957,
        x2: 1_049.614,
        z2: 1754.2675,
    },
    Decal {
        texture: "b_ok",
        spirit: false,
        x1: 1237.0103,
        z1: 1714.2366,
        x2: 1283.4124,
        z2: 1756.8084,
    },
    Decal {
        texture: "b_ok",
        spirit: false,
        x1: 1136.7727,
        z1: 1714.4651,
        x2: 1_183.175,
        z2: 1757.0369,
    },
    Decal {
        texture: "b_bad",
        spirit: false,
        x1: 1462.9656,
        z1: 1581.8551,
        x2: 1509.3679,
        z2: 1624.4269,
    },
    Decal {
        texture: "s_ok",
        spirit: true,
        x1: 3_647.426,
        z1: 1459.3376,
        x2: 3693.8281,
        z2: 1501.9094,
    },
    Decal {
        texture: "s_bad",
        spirit: true,
        x1: 3_770.721,
        z1: 1465.0059,
        x2: 3824.6894,
        z2: 1514.8742,
    },
    Decal {
        texture: "s_ok",
        spirit: true,
        x1: 3865.8481,
        z1: 1462.4469,
        x2: 3912.2505,
        z2: 1505.0187,
    },
    Decal {
        texture: "s_ok",
        spirit: true,
        x1: 4010.8628,
        z1: 1_453.642,
        x2: 4058.8506,
        z2: 1499.3044,
    },
    Decal {
        texture: "s_ok",
        spirit: true,
        x1: 3958.9128,
        z1: 716.4401,
        x2: 4006.9006,
        z2: 762.1025,
    },
    Decal {
        texture: "b_bad",
        spirit: false,
        x1: 4361.7236,
        z1: 1062.6791,
        x2: 4_409.712,
        z2: 1108.3416,
    },
    Decal {
        texture: "s_bad",
        spirit: true,
        x1: 6_299.203,
        z1: 1582.4793,
        x2: 6347.1904,
        z2: 1628.1417,
    },
    Decal {
        texture: "s_bad",
        spirit: true,
        x1: 6_279.833,
        z1: 1_688.462,
        x2: 6327.8203,
        z2: 1734.1245,
    },
    Decal {
        texture: "s_bad",
        spirit: true,
        x1: 6175.0254,
        z1: 1577.9121,
        x2: 6223.0127,
        z2: 1623.5746,
    },
    Decal {
        texture: "s_bad",
        spirit: true,
        x1: 6222.2607,
        z1: 1666.8832,
        x2: 6_270.248,
        z2: 1712.5457,
    },
    Decal {
        texture: "b_ok",
        spirit: false,
        x1: 6517.4815,
        z1: 1555.8472,
        x2: 6565.4697,
        z2: 1601.5096,
    },
];

const GROUND_OFFSET: f32 = 1.5;

struct Quad {
    spirit: bool,
    texture: String,
    vertices: Vec<spring::gfx::GfxVertexData>,
}

impl Quad {
    fn new(spring: &spring::Spring<'_, spring::Ui>, decal: &Decal) -> Self {
        let corners = [
            (decal.x1, decal.z1, 0.0, 0.0),
            (decal.x2, decal.z1, 1.0, 0.0),
            (decal.x2, decal.z2, 1.0, 1.0),
            (decal.x1, decal.z2, 0.0, 1.0),
        ];
        let vertices = corners
            .iter()
            .map(|(x, z, u, v)| {
                let y = spring.terrain().get_ground_height(*x, *z).unwrap_or(0.0) + GROUND_OFFSET;
                spring::gfx::GfxVertexData {
                    vertex: vec![*x, y, *z],
                    normal: vec![0.0, 1.0, 0.0],
                    tex_coord: vec![*u, *v],
                    color: vec![1.0, 1.0, 1.0, 1.0],
                    has_vertex: true,
                    has_normal: false,
                    has_tex_coord: true,
                    has_color: true,
                }
            })
            .collect();
        Self {
            spirit: decal.spirit,
            texture: format!("LuaUI/images/ground/{}.png", decal.texture),
            vertices,
        }
    }

    fn draw(&self, spring: &spring::Spring<'_, spring::Ui>) {
        if spring.gfx().bind_texture(&self.texture, 0, true).is_err() {
            return;
        }
        let _ = spring.gfx().shape(gl::TRIANGLE_FAN, &self.vertices);
        let _ = spring.gfx().bind_texture(&self.texture, 0, false);
    }
}

#[derive(Default)]
pub struct GfxBloodWidget {
    quads: RefCell<Vec<Quad>>,
}

impl GfxBloodWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxBloodWidget {
    fn name(&self) -> &'static str {
        "GfxBlood"
    }

    fn draw_world_pre_unit(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        let spirit = ctx.global().player.access().spirit;
        let mut quads = self.quads.borrow_mut();
        if quads.is_empty() {
            *quads = DECALS
                .iter()
                .map(|decal| Quad::new(spring, decal))
                .collect();
        }
        let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
            enable: true,
            set_func: false,
            func: 0,
        });
        let _ = spring.gfx().blending(true);
        let _ = spring.gfx().blend_func(gl::SRC_ALPHA, gl::ONE);
        let _ = spring.gfx().use_shader(0);
        for quad in quads.iter().filter(|quad| quad.spirit == spirit) {
            quad.draw(spring);
        }
        let _ = spring.gfx().pop_attrib();
    }
}
