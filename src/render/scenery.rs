use crate::config::BiomePalette;
use crate::math::Vec3;
use crate::render::track::TrackRenderContext;
use crate::render::{
    apply_distance_fog, rgb_color, rgba_color, GROUND_FLOOR_Y, TRACK_DRAW_DISTANCE,
};
use crate::track::SceneryTower;
use raylib::prelude::*;

pub const HORIZON_SPLIT_OFFSET_Y: i32 = 50;
pub const SUN_OFFSET_X: i32 = 180;
pub const SUN_OFFSET_Y: i32 = 35;
pub const SUN_OUTLINE_RADIUS: f32 = 76.0;
pub const SUN_CORE_RADIUS: f32 = 72.0;
pub const SUN_PRIMARY_ALPHA: u8 = 210;
pub const SUN_SECONDARY_ALPHA: u8 = 235;

pub const TOWER_CULL_MARGIN: f32 = 30.0;
pub const TOWER_WIRE_PADDING: f32 = 0.35;
pub const SPIRE_HEIGHT_OFFSET: f32 = 14.0;
pub const SPIRE_HEIGHT: f32 = 28.0;
pub const SPIRE_WIDTH_SCALE: f32 = 0.62;
pub const SPIRE_WIRE_PADDING: f32 = 0.3;

pub const GRID_MIN_INDEX: i32 = -10;
pub const GRID_MAX_INDEX: i32 = 12;
pub const GRID_SPACING: f32 = 60.0;
pub const GRID_MIN_Z: f32 = -350.0;
pub const GRID_MAX_Z: f32 = 650.0;
pub const GRID_X_SHIFT: f32 = 100.0;
pub const GRID_FOG_DIST: f32 = 220.0;
pub const GRID_ALPHA: u8 = 165;

#[inline]
pub fn compute_horizon_ground_color(sky_top: [u8; 3]) -> Color {
    Color::new(
        sky_top[0].saturating_add(16),
        sky_top[1].saturating_add(18),
        sky_top[2].saturating_add(24),
        255,
    )
}

#[inline]
pub fn compute_tower_base_color(sky_top: [u8; 3]) -> Color {
    let base_r = (sky_top[0] as u16 + 45).min(220) as u8;
    let base_g = (sky_top[1] as u16 + 55).min(220) as u8;
    let base_b = (sky_top[2] as u16 + 75).min(235) as u8;
    Color::new(base_r, base_g, base_b, 255)
}

pub fn draw_sky_backdrop(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    biome: &BiomePalette,
) {
    let top = rgb_color(biome.sky_top);
    let hor = rgb_color(biome.sky_horizon);
    let ground = compute_horizon_ground_color(biome.sky_top);

    d.clear_background(top);
    d.draw_rectangle_gradient_v(0, 0, screen_w, screen_h / 2 + HORIZON_SPLIT_OFFSET_Y, top, hor);
    d.draw_rectangle_gradient_v(
        0,
        screen_h / 2 + HORIZON_SPLIT_OFFSET_Y,
        screen_w,
        screen_h / 2,
        hor,
        ground,
    );

    let sun_x = screen_w / 2 + SUN_OFFSET_X;
    let sun_y = screen_h / 2 - SUN_OFFSET_Y;
    let neon1 = rgba_color(biome.neon_primary, SUN_PRIMARY_ALPHA);
    let neon2 = rgba_color(biome.neon_secondary, SUN_SECONDARY_ALPHA);
    d.draw_circle(sun_x, sun_y, SUN_OUTLINE_RADIUS, Color::new(12, 14, 22, 255));
    d.draw_circle_gradient(sun_x, sun_y, SUN_CORE_RADIUS, neon1, neon2);
}

fn draw_scenery_tower(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    tower: &SceneryTower,
    idx: usize,
    ctx: &TrackRenderContext,
    base_color: Color,
    ink_col: Color,
) {
    let p = tower.pos;
    let dist = p.distance_to(ctx.cam_pos);
    if dist > TRACK_DRAW_DISTANCE + TOWER_CULL_MARGIN {
        return;
    }
    let b_col = apply_distance_fog(base_color, ctx.fog_col, dist);
    d.draw_cube(
        p.to_rl(),
        tower.size.x,
        tower.size.y,
        tower.size.z,
        b_col,
    );
    d.draw_cube_wires(
        p.to_rl(),
        tower.size.x + TOWER_WIRE_PADDING,
        tower.size.y + TOWER_WIRE_PADDING,
        tower.size.z + TOWER_WIRE_PADDING,
        ink_col,
    );

    if tower.has_spire {
        let neon1 = rgb_color(ctx.biome.neon_primary);
        let neon2 = rgb_color(ctx.biome.neon_secondary);
        let crown_pos = (p + Vec3::UP * (tower.size.y * 0.5 + SPIRE_HEIGHT_OFFSET)).to_rl();
        let crown_col =
            apply_distance_fog(if idx % 2 == 0 { neon2 } else { neon1 }, ctx.fog_col, dist);
        d.draw_cube(
            crown_pos,
            tower.size.x * SPIRE_WIDTH_SCALE,
            SPIRE_HEIGHT,
            tower.size.z * SPIRE_WIDTH_SCALE,
            crown_col,
        );
        d.draw_cube_wires(
            crown_pos,
            tower.size.x * SPIRE_WIDTH_SCALE + SPIRE_WIRE_PADDING,
            SPIRE_HEIGHT + SPIRE_WIRE_PADDING,
            tower.size.z * SPIRE_WIDTH_SCALE + SPIRE_WIRE_PADDING,
            ink_col,
        );
    }
}

pub(crate) fn draw_scenery_props(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    towers: &[SceneryTower],
    ctx: &TrackRenderContext,
) {
    let neon1 = rgb_color(ctx.biome.neon_primary);
    let grid_col = apply_distance_fog(
        Color::new(neon1.r / 2, neon1.g / 2, neon1.b / 2, GRID_ALPHA),
        ctx.fog_col,
        GRID_FOG_DIST,
    );

    for g in GRID_MIN_INDEX..=GRID_MAX_INDEX {
        let coord = g as f32 * GRID_SPACING;
        d.draw_line_3D(
            Vector3::new(coord, GROUND_FLOOR_Y, GRID_MIN_Z),
            Vector3::new(coord, GROUND_FLOOR_Y, GRID_MAX_Z),
            grid_col,
        );
        d.draw_line_3D(
            Vector3::new(GRID_MIN_Z, GROUND_FLOOR_Y, coord + GRID_X_SHIFT),
            Vector3::new(GRID_MAX_Z, GROUND_FLOOR_Y, coord + GRID_X_SHIFT),
            grid_col,
        );
    }

    let ink_col = Color::new(12, 14, 22, 255);
    let base_color = compute_tower_base_color(ctx.biome.sky_top);

    for (idx, tower) in towers.iter().enumerate() {
        draw_scenery_tower(d, tower, idx, ctx, base_color, ink_col);
    }
}
