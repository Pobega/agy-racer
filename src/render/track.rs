use crate::config::BiomePalette;
use crate::math::Vec3;
use crate::physics::Ship;
use crate::render::scenery::draw_scenery_props;
use crate::render::textures::RenderAssets;
use crate::render::{
    apply_distance_fog, draw_textured_quad_3d, pipe_ambient_occlusion, rgb_color,
    ship_thruster_light_boost, toon_lighting_band, TexturedQuad3D, BARRIER_CURVATURE_THRESHOLD,
    BARRIER_WALL_HEIGHT, CURVED_CROSS_SECTION_SLICES, CURVED_SURFACE_THRESHOLD,
    FLAT_CROSS_SECTION_SLICES, GROUND_FLOOR_Y, KEEL_ANCHOR_DROP, KEEL_ANCHOR_HALF_WIDTH,
    KEEL_MAX_DISTANCE, KEEL_SPINE_DEPTH, KEEL_STRUT_STRIDE, PYLON_STRIDE, START_ARCH_HEIGHT,
    TRACK_DRAW_DISTANCE, TRACK_UV_V_SCALE,
};
use crate::track::{BakedSample, BakedSegment, BakedTrack, SurfaceFrame, SurfaceType};
use raylib::prelude::*;

pub const DEFAULT_SUN_DIR: Vec3 = Vec3::new(0.4, 0.85, -0.35);

pub const BARRIER_RAIL_RADIUS: f32 = 0.28;
pub const BARRIER_RAIL_SIDES: i32 = 5;
pub const BARRIER_RAIL_COLOR: Color = Color::new(12, 14, 22, 255);

pub const GIRDER_STRUT_RADIUS: f32 = 0.28;
pub const GIRDER_SPINE_RADIUS: f32 = 0.45;
pub const GIRDER_SIDES: i32 = 4;
pub const GIRDER_COLOR: Color = Color::new(16, 18, 28, 255);

pub const PYLON_TOP_OFFSET: f32 = 0.6;
pub const PYLON_RADIUS_TOP: f32 = 1.3;
pub const PYLON_RADIUS_BOTTOM: f32 = 2.1;
pub const PYLON_SIDES: i32 = 6;
pub const PYLON_COLOR: Color = Color::new(72, 84, 108, 255);

pub const ARCH_POST_RADIUS: f32 = 0.45;
pub const ARCH_CROSSBAR_RADIUS: f32 = 0.55;
pub const ARCH_SIDES: i32 = 6;

pub const SHADOW_PULSE_FREQ: f32 = 18.0;
pub const SHADOW_PULSE_AMP: f32 = 0.15;
pub const SHADOW_PULSE_BASE: f32 = 0.85;
pub const SHADOW_NORMAL_OFFSET: f32 = 0.08;
pub const SHADOW_NOSE_FWD: f32 = 2.3;
pub const SHADOW_WING_BACK: f32 = 1.4;
pub const SHADOW_WING_SPAN: f32 = 1.5;
pub const SHADOW_COLOR: Color = Color::new(0, 0, 0, 165);

pub const GLOW_BACK_OFFSET: f32 = 2.1;
pub const GLOW_NORMAL_OFFSET_START: f32 = 0.06;
pub const GLOW_NORMAL_OFFSET_END: f32 = 0.12;
pub const GLOW_RADIUS_BASE: f32 = 1.6;
pub const GLOW_SIDES: i32 = 8;
pub const GLOW_BOOST_COLOR: Color = Color::new(40, 245, 255, 115);
pub const GLOW_NORMAL_COLOR: Color = Color::new(255, 130, 35, 95);

#[derive(Debug, Clone, Copy)]
pub struct TrackRenderContext<'a> {
    pub cam_pos: Vec3,
    pub sun_dir: Vec3,
    pub player_pos: Vec3,
    pub player_boosting: bool,
    pub fog_col: Color,
    pub biome: &'a BiomePalette,
}

#[derive(Debug, Clone, Copy)]
pub struct DeckSliceParams {
    pub sl: usize,
    pub slices: usize,
    pub v0: f32,
    pub v1: f32,
    pub tex_id: u32,
    pub baked_ao: f32,
    pub dist: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TrackBarrierFrames<'a> {
    pub left0: &'a SurfaceFrame,
    pub right0: &'a SurfaceFrame,
    pub left1: &'a SurfaceFrame,
    pub right1: &'a SurfaceFrame,
}

#[derive(Debug, Clone, Copy)]
pub struct BarrierParams {
    pub v0: f32,
    pub v1: f32,
    pub has_curved_walls: bool,
    pub tex_wall_id: u32,
    pub dist: f32,
}

pub fn select_surface_texture_and_tint<'a>(
    surface: SurfaceType,
    _sample_index: usize,
    assets: &'a RenderAssets,
    biome: &BiomePalette,
) -> (&'a Texture2D, Color) {
    let tex = match surface {
        SurfaceType::Normal => &assets.tex_road_cel,
        SurfaceType::BoostPad => &assets.tex_road_boost,
        SurfaceType::RechargeStrip => &assets.tex_road_recharge,
        SurfaceType::LowGripIce => &assets.tex_road_ice,
    };
    let tint = match surface {
        SurfaceType::Normal => Color::WHITE,
        SurfaceType::BoostPad => rgb_color(biome.neon_primary),
        SurfaceType::RechargeStrip => rgb_color(biome.neon_secondary),
        SurfaceType::LowGripIce => Color::new(210, 240, 255, 255),
    };
    (tex, tint)
}

fn compute_deck_vertex_color(
    c00: &SurfaceFrame,
    s0: &BakedSample,
    is_outer_curb: bool,
    cel_band: f32,
    ctx: &TrackRenderContext,
    dist: f32,
) -> Color {
    let neon1 = rgb_color(ctx.biome.neon_primary);
    let neon2 = rgb_color(ctx.biome.neon_secondary);

    let (mut light_r, mut light_g, mut light_b) =
        if is_outer_curb && s0.surface == SurfaceType::Normal {
            (
                cel_band * (0.55 + neon2.r as f32 / 510.0),
                cel_band * (0.55 + neon2.g as f32 / 510.0),
                cel_band * (0.55 + neon2.b as f32 / 510.0),
            )
        } else {
            let tr = 0.55 + (ctx.biome.sky_top[0] as f32 + neon1.r as f32 * 0.5) / 420.0;
            let tg = 0.58 + (ctx.biome.sky_top[1] as f32 + neon1.g as f32 * 0.5) / 420.0;
            let tb = 0.68 + (ctx.biome.sky_top[2] as f32 + neon1.b as f32 * 0.5) / 420.0;
            (cel_band * tr, cel_band * tg, cel_band * tb)
        };

    let atten = ship_thruster_light_boost(c00.surface_pos, ctx.player_pos, ctx.player_boosting);
    if atten > 0.0 {
        if ctx.player_boosting {
            light_g += atten * 0.7;
            light_b += atten * 1.1;
        } else {
            light_r += atten * 0.8;
            light_g += atten * 0.45;
        }
    }

    apply_distance_fog(
        Color::new(
            (235.0 * light_r).min(255.0) as u8,
            (235.0 * light_g).min(255.0) as u8,
            (245.0 * light_b).min(255.0) as u8,
            255,
        ),
        ctx.fog_col,
        dist,
    )
}

fn draw_track_deck_slice(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    s0: &BakedSample,
    s1: &BakedSample,
    params: &DeckSliceParams,
    ctx: &TrackRenderContext,
) {
    let u0 = params.sl as f32 / params.slices as f32;
    let u1 = (params.sl + 1) as f32 / params.slices as f32;

    let c00 = s0.evaluate_cross_section((u0 - 0.5) * s0.width);
    let c01 = s0.evaluate_cross_section((u1 - 0.5) * s0.width);
    let c10 = s1.evaluate_cross_section((u0 - 0.5) * s1.width);
    let c11 = s1.evaluate_cross_section((u1 - 0.5) * s1.width);

    let cel_band = toon_lighting_band(c00.normal, ctx.sun_dir) * params.baked_ao;
    let is_outer_curb = (params.sl == 0 || params.sl == params.slices - 1) && !c00.is_cylinder;
    let vert_col = compute_deck_vertex_color(&c00, s0, is_outer_curb, cel_band, ctx, params.dist);

    let quad = TexturedQuad3D::new(
        [
            c00.surface_pos.to_rl(),
            c10.surface_pos.to_rl(),
            c11.surface_pos.to_rl(),
            c01.surface_pos.to_rl(),
        ],
        [
            (u0, params.v0),
            (u0, params.v1),
            (u1, params.v1),
            (u1, params.v0),
        ],
    );
    draw_textured_quad_3d(d, params.tex_id, &quad, vert_col);
}

fn draw_track_side_barriers(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    frames: &TrackBarrierFrames,
    params: &BarrierParams,
    ctx: &TrackRenderContext,
) {
    let wall_tint = apply_distance_fog(Color::WHITE, ctx.fog_col, params.dist);

    if frames.left0.walls.has_left() && !params.has_curved_walls {
        let t0 = frames.left0.surface_pos + frames.left0.normal * BARRIER_WALL_HEIGHT;
        let t1 = frames.left1.surface_pos + frames.left1.normal * BARRIER_WALL_HEIGHT;
        let quad = TexturedQuad3D::new(
            [
                t0.to_rl(),
                t1.to_rl(),
                frames.left1.surface_pos.to_rl(),
                frames.left0.surface_pos.to_rl(),
            ],
            [
                (params.v0, 0.0),
                (params.v1, 0.0),
                (params.v1, 1.0),
                (params.v0, 1.0),
            ],
        );
        draw_textured_quad_3d(d, params.tex_wall_id, &quad, wall_tint);
        d.draw_cylinder_ex(
            t0.to_rl(),
            t1.to_rl(),
            BARRIER_RAIL_RADIUS,
            BARRIER_RAIL_RADIUS,
            BARRIER_RAIL_SIDES,
            BARRIER_RAIL_COLOR,
        );
    }

    if frames.right0.walls.has_right() && !params.has_curved_walls {
        let t0 = frames.right0.surface_pos + frames.right0.normal * BARRIER_WALL_HEIGHT;
        let t1 = frames.right1.surface_pos + frames.right1.normal * BARRIER_WALL_HEIGHT;
        let quad = TexturedQuad3D::new(
            [
                t1.to_rl(),
                t0.to_rl(),
                frames.right0.surface_pos.to_rl(),
                frames.right1.surface_pos.to_rl(),
            ],
            [
                (params.v1, 0.0),
                (params.v0, 0.0),
                (params.v0, 1.0),
                (params.v1, 1.0),
            ],
        );
        draw_textured_quad_3d(d, params.tex_wall_id, &quad, wall_tint);
        d.draw_cylinder_ex(
            t0.to_rl(),
            t1.to_rl(),
            BARRIER_RAIL_RADIUS,
            BARRIER_RAIL_RADIUS,
            BARRIER_RAIL_SIDES,
            BARRIER_RAIL_COLOR,
        );
    }
}

fn draw_under_track_keel_truss(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    s0: &BakedSample,
    s1: &BakedSample,
) {
    let keel0 = (s0.pos - s0.up * KEEL_SPINE_DEPTH).to_rl();
    let keel1 = (s1.pos - s1.up * KEEL_SPINE_DEPTH).to_rl();
    let strut_l = (s0.pos - s0.right * KEEL_ANCHOR_HALF_WIDTH - s0.up * KEEL_ANCHOR_DROP).to_rl();
    let strut_r = (s0.pos + s0.right * KEEL_ANCHOR_HALF_WIDTH - s0.up * KEEL_ANCHOR_DROP).to_rl();
    d.draw_cylinder_ex(
        strut_l,
        keel0,
        GIRDER_STRUT_RADIUS,
        GIRDER_STRUT_RADIUS,
        GIRDER_SIDES,
        GIRDER_COLOR,
    );
    d.draw_cylinder_ex(
        strut_r,
        keel0,
        GIRDER_STRUT_RADIUS,
        GIRDER_STRUT_RADIUS,
        GIRDER_SIDES,
        GIRDER_COLOR,
    );
    d.draw_cylinder_ex(
        keel0,
        keel1,
        GIRDER_SPINE_RADIUS,
        GIRDER_SPINE_RADIUS,
        GIRDER_SIDES,
        GIRDER_COLOR,
    );
}

fn draw_support_pylon(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    s0: &BakedSample,
    fog_col: Color,
    dist: f32,
) {
    let pylon_col = apply_distance_fog(PYLON_COLOR, fog_col, dist);
    let ground_p = Vec3::new(s0.pos.x, GROUND_FLOOR_Y, s0.pos.z).to_rl();
    d.draw_cylinder_ex(
        (s0.pos - s0.up * PYLON_TOP_OFFSET).to_rl(),
        ground_p,
        PYLON_RADIUS_TOP,
        PYLON_RADIUS_BOTTOM,
        PYLON_SIDES,
        pylon_col,
    );
}

fn draw_start_finish_arch(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    s0: &BakedSample,
    left0: &SurfaceFrame,
    right0: &SurfaceFrame,
    ctx: &TrackRenderContext,
    dist: f32,
) {
    let neon1 = rgb_color(ctx.biome.neon_primary);
    let neon2 = rgb_color(ctx.biome.neon_secondary);
    let arch_col = apply_distance_fog(neon1, ctx.fog_col, dist);
    let bl = left0.surface_pos.to_rl();
    let br = right0.surface_pos.to_rl();
    let tl = (left0.surface_pos + s0.up * START_ARCH_HEIGHT).to_rl();
    let tr = (right0.surface_pos + s0.up * START_ARCH_HEIGHT).to_rl();
    d.draw_cylinder_ex(bl, tl, ARCH_POST_RADIUS, ARCH_POST_RADIUS, ARCH_SIDES, arch_col);
    d.draw_cylinder_ex(br, tr, ARCH_POST_RADIUS, ARCH_POST_RADIUS, ARCH_SIDES, arch_col);
    d.draw_cylinder_ex(
        tl,
        tr,
        ARCH_CROSSBAR_RADIUS,
        ARCH_CROSSBAR_RADIUS,
        ARCH_SIDES,
        neon2,
    );
}

fn draw_track_segment(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    seg: &BakedSegment,
    ctx: &TrackRenderContext,
    assets: &RenderAssets,
) {
    let sample_count = seg.samples.len();
    if sample_count < 2 {
        return;
    }

    for i in 0..sample_count {
        let next_i = if i + 1 < sample_count {
            i + 1
        } else if seg.closed_loop {
            0
        } else {
            break;
        };

        let s0 = &seg.samples[i];
        let s1 = &seg.samples[next_i];
        let dist = s0.pos.distance_to(ctx.cam_pos);
        if dist > TRACK_DRAW_DISTANCE {
            continue;
        }

        if s0.has_floor {
            let slices = if s0.curvature.abs() > CURVED_SURFACE_THRESHOLD
                || s1.curvature.abs() > CURVED_SURFACE_THRESHOLD
            {
                CURVED_CROSS_SECTION_SLICES
            } else {
                FLAT_CROSS_SECTION_SLICES
            };

            let v0 = s0.s * TRACK_UV_V_SCALE;
            let v1 = if next_i == 0 {
                seg.total_length * TRACK_UV_V_SCALE
            } else {
                s1.s * TRACK_UV_V_SCALE
            };

            let (tex, _) = select_surface_texture_and_tint(s0.surface, i, assets, ctx.biome);
            let tex_id = tex.id;
            let baked_ao = pipe_ambient_occlusion(s0.curvature);

            for sl in 0..slices {
                let params = DeckSliceParams {
                    sl,
                    slices,
                    v0,
                    v1,
                    tex_id,
                    baked_ao,
                    dist,
                };
                draw_track_deck_slice(d, s0, s1, &params, ctx);
            }

            let left0 = s0.evaluate_cross_section(-s0.width * 0.5);
            let right0 = s0.evaluate_cross_section(s0.width * 0.5);
            let left1 = s1.evaluate_cross_section(-s1.width * 0.5);
            let right1 = s1.evaluate_cross_section(s1.width * 0.5);
            let has_curved_walls = s0.curvature.abs() > BARRIER_CURVATURE_THRESHOLD
                || s1.curvature.abs() > BARRIER_CURVATURE_THRESHOLD;

            let frames = TrackBarrierFrames {
                left0: &left0,
                right0: &right0,
                left1: &left1,
                right1: &right1,
            };
            let barrier_params = BarrierParams {
                v0,
                v1,
                has_curved_walls,
                tex_wall_id: assets.tex_wall_hazard.id,
                dist,
            };
            draw_track_side_barriers(d, &frames, &barrier_params, ctx);

            if i % KEEL_STRUT_STRIDE == 0 && dist < KEEL_MAX_DISTANCE {
                draw_under_track_keel_truss(d, s0, s1);
            }

            if i % PYLON_STRIDE == 0 && !left0.is_cylinder {
                draw_support_pylon(d, s0, ctx.fog_col, dist);
            }

            if seg.id == 0 && i == 0 {
                draw_start_finish_arch(d, s0, &left0, &right0, ctx, dist);
            }
        }
    }
}

fn draw_ship_ground_shadow_and_glow(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    ships: &[Ship],
    track: &BakedTrack,
    time_sec: f32,
) {
    let pulse = (time_sec * SHADOW_PULSE_FREQ).sin() * SHADOW_PULSE_AMP + SHADOW_PULSE_BASE;
    for ship in ships {
        if !ship.is_airborne {
            let seg = track.segment_or_main(ship.segment_id);
            let sample = seg.sample_at(ship.s);
            let frame = sample.evaluate_cross_section(ship.r);
            let shadow_center = frame.surface_pos + frame.normal * SHADOW_NORMAL_OFFSET;
            let f = ship.world_forward;
            let r = ship.world_right;
            let s_nose = (shadow_center + f * SHADOW_NOSE_FWD).to_rl();
            let s_wl = (shadow_center - f * SHADOW_WING_BACK - r * SHADOW_WING_SPAN).to_rl();
            let s_wr = (shadow_center - f * SHADOW_WING_BACK + r * SHADOW_WING_SPAN).to_rl();
            d.draw_triangle3D(s_nose, s_wl, s_wr, SHADOW_COLOR);
            d.draw_triangle3D(s_nose, s_wr, s_wl, SHADOW_COLOR);

            let glow_pos =
                (shadow_center - f * GLOW_BACK_OFFSET + frame.normal * GLOW_NORMAL_OFFSET_START)
                    .to_rl();
            let glow_col = if ship.boost_timer > 0.0 {
                GLOW_BOOST_COLOR
            } else {
                GLOW_NORMAL_COLOR
            };
            let glow_rad = GLOW_RADIUS_BASE * pulse;
            d.draw_cylinder_ex(
                glow_pos,
                (shadow_center - f * GLOW_BACK_OFFSET + frame.normal * GLOW_NORMAL_OFFSET_END)
                    .to_rl(),
                glow_rad,
                glow_rad,
                GLOW_SIDES,
                glow_col,
            );
        }
    }
}

pub fn draw_world_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    track: &BakedTrack,
    ships: &[Ship],
    cam_pos: Vec3,
    time_sec: f32,
    assets: &RenderAssets,
    biome: &BiomePalette,
) {
    let fog_col = rgb_color(biome.sky_horizon);
    let sun_dir = DEFAULT_SUN_DIR.normalized();
    let player_pos = ships.first().map(|s| s.world_pos).unwrap_or(Vec3::ZERO);
    let player_boosting = ships.first().map(|s| s.boost_timer > 0.0).unwrap_or(false);

    let ctx = TrackRenderContext {
        cam_pos,
        sun_dir,
        player_pos,
        player_boosting,
        fog_col,
        biome,
    };

    draw_scenery_props(d, &track.def.scenery_towers, &ctx);

    for seg in &track.segments {
        draw_track_segment(d, seg, &ctx, assets);
    }

    draw_ship_ground_shadow_and_glow(d, ships, track, time_sec);
}
