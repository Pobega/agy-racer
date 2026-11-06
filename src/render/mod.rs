use crate::math::Vec3;
use raylib::ffi;
use raylib::prelude::*;

pub mod scenery;
pub mod ship;
pub mod textures;
pub mod track;

pub use scenery::draw_sky_backdrop;
pub use ship::draw_cel_ship;
pub use textures::RenderAssets;
pub use track::draw_world_3d;

pub const FOG_START_DISTANCE: f32 = 165.0;
pub const FOG_RANGE: f32 = 440.0;
pub const FOG_MAX_DENSITY: f32 = 0.84;

pub const TRACK_DRAW_DISTANCE: f32 = 620.0;
pub const CURVED_SURFACE_THRESHOLD: f32 = 0.12;
pub const CURVED_CROSS_SECTION_SLICES: usize = 10;
pub const FLAT_CROSS_SECTION_SLICES: usize = 4;
pub const TRACK_UV_V_SCALE: f32 = 0.08;

pub const CEL_BRIGHT_DOT_THRESHOLD: f32 = 0.45;
pub const CEL_BRIGHT_MULTIPLIER: f32 = 1.15;
pub const CEL_MID_MULTIPLIER: f32 = 0.94;
pub const CEL_SHADOW_MULTIPLIER: f32 = 0.72;
pub const PIPE_ENCLOSED_CURVATURE: f32 = 0.8;
pub const PIPE_AO_MULTIPLIER: f32 = 0.78;

pub const KEEL_STRUT_STRIDE: usize = 4;
pub const KEEL_MAX_DISTANCE: f32 = 380.0;
pub const KEEL_SPINE_DEPTH: f32 = 3.2;
pub const KEEL_ANCHOR_HALF_WIDTH: f32 = 5.0;
pub const KEEL_ANCHOR_DROP: f32 = 0.25;
pub const PYLON_STRIDE: usize = 24;
pub const GROUND_FLOOR_Y: f32 = -45.0;
pub const START_ARCH_HEIGHT: f32 = 12.0;

pub const BARRIER_WALL_HEIGHT: f32 = 2.1;
pub const BARRIER_CURVATURE_THRESHOLD: f32 = 0.25;
pub const THRUSTER_LIGHT_RADIUS: f32 = 22.0;
pub const THRUSTER_LIGHT_BASE_INTENSITY: f32 = 0.65;
pub const THRUSTER_LIGHT_BOOST_MULT: f32 = 1.2;
pub const BACKFACE_SHADE_DIVISOR: u8 = 2;

#[inline]
pub fn rgb_color(rgb: [u8; 3]) -> Color {
    Color::new(rgb[0], rgb[1], rgb[2], 255)
}

#[inline]
pub fn rgba_color(rgb: [u8; 3], alpha: u8) -> Color {
    Color::new(rgb[0], rgb[1], rgb[2], alpha)
}

#[inline]
pub fn scale_color_brightness(color: Color, factor: f32) -> Color {
    Color::new(
        ((color.r as f32 * factor).clamp(0.0, 255.0)) as u8,
        ((color.g as f32 * factor).clamp(0.0, 255.0)) as u8,
        ((color.b as f32 * factor).clamp(0.0, 255.0)) as u8,
        color.a,
    )
}

#[inline]
pub fn apply_distance_fog(color: Color, fog_color: Color, distance: f32) -> Color {
    let factor = ((distance - FOG_START_DISTANCE) / FOG_RANGE).clamp(0.0, FOG_MAX_DENSITY);
    let inv = 1.0 - factor;
    Color::new(
        (color.r as f32 * inv + fog_color.r as f32 * factor) as u8,
        (color.g as f32 * inv + fog_color.g as f32 * factor) as u8,
        (color.b as f32 * inv + fog_color.b as f32 * factor) as u8,
        color.a,
    )
}

#[inline]
pub fn toon_lighting_band(surface_normal: Vec3, sun_dir: Vec3) -> f32 {
    let ndotl = surface_normal.dot(sun_dir);
    if ndotl > CEL_BRIGHT_DOT_THRESHOLD {
        CEL_BRIGHT_MULTIPLIER
    } else if ndotl > 0.0 {
        CEL_MID_MULTIPLIER
    } else {
        CEL_SHADOW_MULTIPLIER
    }
}

#[inline]
pub fn pipe_ambient_occlusion(curvature: f32) -> f32 {
    if curvature > PIPE_ENCLOSED_CURVATURE {
        PIPE_AO_MULTIPLIER
    } else {
        1.0
    }
}

#[inline]
pub fn ship_thruster_light_boost(surface_pos: Vec3, player_pos: Vec3, is_boosting: bool) -> f32 {
    let dist = surface_pos.distance_to(player_pos);
    if dist >= THRUSTER_LIGHT_RADIUS {
        return 0.0;
    }
    let norm = 1.0 - dist / THRUSTER_LIGHT_RADIUS;
    let base = norm * norm * THRUSTER_LIGHT_BASE_INTENSITY;
    if is_boosting {
        base * THRUSTER_LIGHT_BOOST_MULT
    } else {
        base
    }
}

#[derive(Debug, Clone)]
pub struct TexturedQuad3D {
    pub corners: [Vector3; 4],
    pub uvs: [(f32, f32); 4],
}

impl TexturedQuad3D {
    #[inline]
    pub fn new(corners: [Vector3; 4], uvs: [(f32, f32); 4]) -> Self {
        Self { corners, uvs }
    }
}

pub(crate) fn draw_textured_quad_3d(
    _d: &mut RaylibMode3D<RaylibDrawHandle>,
    tex_id: u32,
    quad: &TexturedQuad3D,
    tint: Color,
) {
    unsafe {
        ffi::rlSetTexture(tex_id);
        ffi::rlBegin(ffi::RL_QUADS as i32);
        ffi::rlColor4ub(tint.r, tint.g, tint.b, tint.a);

        ffi::rlTexCoord2f(quad.uvs[0].0, quad.uvs[0].1);
        ffi::rlVertex3f(quad.corners[0].x, quad.corners[0].y, quad.corners[0].z);
        ffi::rlTexCoord2f(quad.uvs[3].0, quad.uvs[3].1);
        ffi::rlVertex3f(quad.corners[3].x, quad.corners[3].y, quad.corners[3].z);
        ffi::rlTexCoord2f(quad.uvs[2].0, quad.uvs[2].1);
        ffi::rlVertex3f(quad.corners[2].x, quad.corners[2].y, quad.corners[2].z);
        ffi::rlTexCoord2f(quad.uvs[1].0, quad.uvs[1].1);
        ffi::rlVertex3f(quad.corners[1].x, quad.corners[1].y, quad.corners[1].z);

        ffi::rlColor4ub(
            tint.r / BACKFACE_SHADE_DIVISOR,
            tint.g / BACKFACE_SHADE_DIVISOR,
            tint.b / BACKFACE_SHADE_DIVISOR,
            tint.a,
        );
        ffi::rlTexCoord2f(quad.uvs[0].0, quad.uvs[0].1);
        ffi::rlVertex3f(quad.corners[0].x, quad.corners[0].y, quad.corners[0].z);
        ffi::rlTexCoord2f(quad.uvs[1].0, quad.uvs[1].1);
        ffi::rlVertex3f(quad.corners[1].x, quad.corners[1].y, quad.corners[1].z);
        ffi::rlTexCoord2f(quad.uvs[2].0, quad.uvs[2].1);
        ffi::rlVertex3f(quad.corners[2].x, quad.corners[2].y, quad.corners[2].z);
        ffi::rlTexCoord2f(quad.uvs[3].0, quad.uvs[3].1);
        ffi::rlVertex3f(quad.corners[3].x, quad.corners[3].y, quad.corners[3].z);

        ffi::rlEnd();
        ffi::rlSetTexture(0);
    }
}
