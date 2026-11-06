use raylib::prelude::*;
use std::fs;
use std::ops::Range;
use std::path::Path;

pub const ROAD_TEX_SIZE: i32 = 128;
pub const WALL_TEX_SIZE: i32 = 64;

pub const OUTER_BORDER_MIN_X: i32 = 4;
pub const OUTER_BORDER_MAX_X: i32 = 124;
pub const CURB_LEFT_RANGE: Range<i32> = 4..16;
pub const CURB_RIGHT_RANGE: Range<i32> = 112..124;
pub const CURB_LEFT_INK_RANGE: Range<i32> = 16..20;
pub const CURB_RIGHT_INK_RANGE: Range<i32> = 108..112;
pub const LANE_LEFT_INK_RANGE: Range<i32> = 38..40;
pub const LANE_RIGHT_INK_RANGE: Range<i32> = 88..90;
pub const CENTER_LANE_RANGE: Range<i32> = 40..88;
pub const CENTER_DASH_X_RANGE: Range<i32> = 61..67;
pub const BOOST_BORDER_WIDTH: i32 = 14;
pub const WALL_TOP_NEON_MAX_Y: i32 = 10;
pub const WALL_BOTTOM_HAZARD_MIN_Y: i32 = 50;

pub const INK_COLOR: Color = Color::new(10, 12, 18, 255);
pub const ROAD_BASE_COLOR: Color = Color::new(200, 205, 220, 255);
pub const CURB_GRAY_COLOR: Color = Color::new(135, 145, 168, 255);
pub const ROAD_CENTER_COLOR: Color = Color::new(218, 225, 240, 255);
pub const ROAD_OUTER_COLOR: Color = Color::new(182, 190, 210, 255);
pub const BOOST_BG_COLOR: Color = Color::new(18, 95, 210, 255);
pub const BOOST_BORDER_COLOR: Color = Color::new(255, 230, 50, 255);
pub const BOOST_ARROW_COLOR: Color = Color::new(40, 250, 255, 255);
pub const RECHARGE_BG_COLOR: Color = Color::new(22, 105, 72, 255);
pub const RECHARGE_GRID_COLOR: Color = Color::new(55, 250, 145, 255);
pub const ICE_BG_COLOR: Color = Color::new(150, 220, 250, 255);
pub const WALL_BG_COLOR: Color = Color::new(82, 94, 118, 255);
pub const WALL_TOP_NEON_COLOR: Color = Color::new(0, 240, 255, 255);
pub const WALL_STRIPE_YELLOW: Color = Color::new(245, 195, 30, 255);
pub const WALL_STRIPE_DARK: Color = Color::new(20, 24, 34, 255);

#[inline]
fn is_road_ink_outline_pixel(x: i32, y: i32) -> bool {
    x < OUTER_BORDER_MIN_X
        || x >= OUTER_BORDER_MAX_X
        || CURB_LEFT_INK_RANGE.contains(&x)
        || CURB_RIGHT_INK_RANGE.contains(&x)
        || LANE_LEFT_INK_RANGE.contains(&x)
        || LANE_RIGHT_INK_RANGE.contains(&x)
        || (y % 64 < 3)
}

#[inline]
fn is_curb_band_pixel(x: i32) -> bool {
    CURB_LEFT_RANGE.contains(&x) || CURB_RIGHT_RANGE.contains(&x)
}

#[inline]
fn is_curb_white_stripe(x: i32, y: i32) -> bool {
    ((y + x / 2) / 16) % 2 == 0
}

#[inline]
fn is_center_dash_pixel(x: i32, y: i32) -> bool {
    CENTER_DASH_X_RANGE.contains(&x) && (y % 32 < 18)
}

#[inline]
fn chevron_metric(x: i32, y: i32) -> (i32, i32) {
    let dy = y % 64;
    let dx = (x - 64).abs();
    let chev_dist = (dy - dx / 2).abs();
    (dx, chev_dist)
}

#[inline]
fn is_center_chevron_fill(x: i32, y: i32) -> bool {
    let (dx, dist) = chevron_metric(x, y);
    dist <= 2 && (8..24).contains(&dx)
}

#[inline]
fn is_center_chevron_ink(x: i32, y: i32) -> bool {
    let (dx, dist) = chevron_metric(x, y);
    dist == 3 && (7..25).contains(&dx)
}

#[inline]
fn is_center_lane_pixel(x: i32) -> bool {
    CENTER_LANE_RANGE.contains(&x)
}

#[inline]
fn is_boost_border_band(x: i32) -> bool {
    x < BOOST_BORDER_WIDTH || x >= ROAD_TEX_SIZE - BOOST_BORDER_WIDTH
}

#[inline]
fn is_boost_chevron_arrow(x: i32, y: i32) -> bool {
    let dx = (x - 64).abs();
    dx < 48 && ((y + dx) / 12) % 2 == 0
}

#[inline]
fn is_recharge_grid_line(x: i32, y: i32) -> bool {
    x < BOOST_BORDER_WIDTH
        || x >= ROAD_TEX_SIZE - BOOST_BORDER_WIDTH
        || (x % 16 < 3)
        || (y % 16 < 3)
}

#[inline]
fn is_ice_crystal_sparkle(x: i32, y: i32) -> bool {
    ((x * 19 + y * 37) % 29) == 0
}

#[inline]
fn is_wall_hazard_top_neon_strip(y: i32) -> bool {
    y < WALL_TOP_NEON_MAX_Y
}

#[inline]
fn is_wall_hazard_bottom_strip(y: i32) -> bool {
    y > WALL_BOTTOM_HAZARD_MIN_Y
}

#[inline]
fn is_hazard_stripe_yellow(x: i32, y: i32) -> bool {
    ((x + y) / 10) % 2 == 0
}

pub struct RenderAssets {
    pub tex_road_cel: Texture2D,
    pub tex_road_boost: Texture2D,
    pub tex_road_recharge: Texture2D,
    pub tex_road_ice: Texture2D,
    pub tex_wall_hazard: Texture2D,
}

impl RenderAssets {
    pub fn load_or_init(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let _ = fs::create_dir_all("assets/textures");

        Self {
            tex_road_cel: Self::load_or_save_png(
                rl,
                thread,
                "assets/textures/road_cel.png",
                Self::default_road_cel_image,
            ),
            tex_road_boost: Self::load_or_save_png(
                rl,
                thread,
                "assets/textures/road_boost.png",
                Self::default_road_boost_image,
            ),
            tex_road_recharge: Self::load_or_save_png(
                rl,
                thread,
                "assets/textures/road_recharge.png",
                Self::default_road_recharge_image,
            ),
            tex_road_ice: Self::load_or_save_png(
                rl,
                thread,
                "assets/textures/road_ice.png",
                Self::default_road_ice_image,
            ),
            tex_wall_hazard: Self::load_or_save_png(
                rl,
                thread,
                "assets/textures/wall_hazard.png",
                Self::default_wall_hazard_image,
            ),
        }
    }

    #[inline]
    pub fn load_or_export(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        Self::load_or_init(rl, thread)
    }

    pub fn reload(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        *self = Self::load_or_init(rl, thread);
    }

    fn load_or_save_png(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &str,
        fallback_gen: fn() -> Image,
    ) -> Texture2D {
        if !Path::new(path).exists() {
            let img = fallback_gen();
            img.export_image(path);
        }
        rl.load_texture(thread, path).unwrap_or_else(|_| {
            rl.load_texture_from_image(thread, &fallback_gen())
                .expect("Failed to upload procedural fallback texture to GPU")
        })
    }

    fn default_road_cel_image() -> Image {
        let mut img = Image::gen_image_color(ROAD_TEX_SIZE, ROAD_TEX_SIZE, ROAD_BASE_COLOR);
        for y in 0..ROAD_TEX_SIZE {
            for x in 0..ROAD_TEX_SIZE {
                let col = if is_road_ink_outline_pixel(x, y) || is_center_chevron_ink(x, y) {
                    INK_COLOR
                } else if is_curb_band_pixel(x) {
                    if is_curb_white_stripe(x, y) {
                        Color::WHITE
                    } else {
                        CURB_GRAY_COLOR
                    }
                } else if is_center_dash_pixel(x, y) || is_center_chevron_fill(x, y) {
                    Color::WHITE
                } else if is_center_lane_pixel(x) {
                    ROAD_CENTER_COLOR
                } else {
                    ROAD_OUTER_COLOR
                };
                img.draw_pixel(x, y, col);
            }
        }
        img
    }

    fn default_road_boost_image() -> Image {
        let mut img = Image::gen_image_color(ROAD_TEX_SIZE, ROAD_TEX_SIZE, BOOST_BG_COLOR);
        for y in 0..ROAD_TEX_SIZE {
            for x in 0..ROAD_TEX_SIZE {
                if is_boost_border_band(x) {
                    img.draw_pixel(x, y, BOOST_BORDER_COLOR);
                } else if is_boost_chevron_arrow(x, y) {
                    img.draw_pixel(x, y, BOOST_ARROW_COLOR);
                }
            }
        }
        img
    }

    fn default_road_recharge_image() -> Image {
        let mut img = Image::gen_image_color(ROAD_TEX_SIZE, ROAD_TEX_SIZE, RECHARGE_BG_COLOR);
        for y in 0..ROAD_TEX_SIZE {
            for x in 0..ROAD_TEX_SIZE {
                if is_recharge_grid_line(x, y) {
                    img.draw_pixel(x, y, RECHARGE_GRID_COLOR);
                }
            }
        }
        img
    }

    fn default_road_ice_image() -> Image {
        let mut img = Image::gen_image_color(ROAD_TEX_SIZE, ROAD_TEX_SIZE, ICE_BG_COLOR);
        for y in 0..ROAD_TEX_SIZE {
            for x in 0..ROAD_TEX_SIZE {
                if is_ice_crystal_sparkle(x, y) {
                    img.draw_pixel(x, y, Color::WHITE);
                }
            }
        }
        img
    }

    fn default_wall_hazard_image() -> Image {
        let mut img = Image::gen_image_color(WALL_TEX_SIZE, WALL_TEX_SIZE, WALL_BG_COLOR);
        for y in 0..WALL_TEX_SIZE {
            for x in 0..WALL_TEX_SIZE {
                if is_wall_hazard_top_neon_strip(y) {
                    img.draw_pixel(x, y, WALL_TOP_NEON_COLOR);
                } else if is_wall_hazard_bottom_strip(y) {
                    let c = if is_hazard_stripe_yellow(x, y) {
                        WALL_STRIPE_YELLOW
                    } else {
                        WALL_STRIPE_DARK
                    };
                    img.draw_pixel(x, y, c);
                }
            }
        }
        img
    }
}
