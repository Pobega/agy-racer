use crate::physics::Ship;
use crate::render::{rgba_color, scale_color_brightness};
use raylib::prelude::*;

pub const MAIN_OUTLINE_RADIUS: f32 = 0.085;
pub const WING_OUTLINE_RADIUS: f32 = 0.075;
pub const SPINE_OUTLINE_RADIUS: f32 = 0.07;
pub const GLINT_LINE_RADIUS: f32 = 0.055;
pub const OUTLINE_SIDES: i32 = 4;

pub const PLUME_BASE_RADIUS: f32 = 0.30;
pub const PLUME_TIP_RADIUS: f32 = 0.02;
pub const PLUME_SIDES: i32 = 6;

pub const CONTRAIL_MIN_SPEED: f32 = 25.0;
pub const CONTRAIL_MAX_GAP_SQ: f32 = 25.0;
pub const CONTRAIL_MAX_ALPHA: f32 = 150.0;

pub const MID_SHADE_FACTOR: f32 = 0.80;
pub const DARK_SHADE_FACTOR: f32 = 0.60;
pub const SOLID_SHIP_MIN_ALPHA: u8 = 200;

pub mod ship_dims {
    pub const NOSE_FWD: f32 = 2.85;
    pub const NOSE_UP: f32 = -0.04;

    pub const NOSE_CHIN_FWD: f32 = 1.45;
    pub const NOSE_CHIN_RIGHT: f32 = 0.42;
    pub const NOSE_CHIN_UP: f32 = -0.06;

    pub const CANOPY_FRONT_FWD: f32 = 0.85;
    pub const CANOPY_FRONT_UP: f32 = 0.34;

    pub const CANOPY_PEAK_FWD: f32 = -0.15;
    pub const CANOPY_PEAK_UP: f32 = 0.68;

    pub const SPINE_TAIL_FWD: f32 = -1.75;
    pub const SPINE_TAIL_UP: f32 = 0.42;

    pub const FUSELAGE_FWD: f32 = -0.4;
    pub const FUSELAGE_RIGHT: f32 = 0.68;
    pub const FUSELAGE_UP: f32 = 0.12;

    pub const WING_ROOT_FWD: f32 = -0.35;
    pub const WING_ROOT_RIGHT: f32 = 1.65;
    pub const WING_ROOT_UP: f32 = -0.10;

    pub const WING_TIP_FWD: f32 = -1.55;
    pub const WING_TIP_RIGHT: f32 = 1.95;
    pub const WING_TIP_UP: f32 = 0.05;

    pub const WINGLET_TOP_FWD: f32 = -1.65;
    pub const WINGLET_TOP_RIGHT: f32 = 1.90;
    pub const WINGLET_TOP_UP: f32 = 0.48;

    pub const TAIL_FWD: f32 = -1.75;
    pub const TAIL_RIGHT: f32 = 0.52;
    pub const TAIL_UP: f32 = 0.08;

    pub const BELLY_KEEL_FWD: f32 = -0.55;
    pub const BELLY_KEEL_UP: f32 = -0.32;

    pub const CANOPY_GLINT_A_FWD: f32 = 0.55;
    pub const CANOPY_GLINT_A_RIGHT: f32 = -0.18;
    pub const CANOPY_GLINT_A_UP: f32 = 0.49;

    pub const CANOPY_GLINT_B_FWD: f32 = 0.15;
    pub const CANOPY_GLINT_B_RIGHT: f32 = 0.22;
    pub const CANOPY_GLINT_B_UP: f32 = 0.62;

    pub const AIRBRAKE_ROOT_FWD: f32 = -0.85;
    pub const AIRBRAKE_INNER_RIGHT: f32 = 0.72;
    pub const AIRBRAKE_INNER_UP: f32 = 0.18;
    pub const AIRBRAKE_OUTER_RIGHT: f32 = 1.45;
    pub const AIRBRAKE_OUTER_UP: f32 = 0.10;
    pub const AIRBRAKE_TIP_FWD: f32 = -1.45;
    pub const AIRBRAKE_TIP_RIGHT: f32 = 1.15;

    pub const THRUSTER_BASE_FWD: f32 = -1.75;
    pub const THRUSTER_OFFSET_RIGHT: f32 = 0.45;
}

#[derive(Debug, Clone)]
pub struct ShipVertices {
    pub nose: Vector3,
    pub nose_chin_left: Vector3,
    pub nose_chin_right: Vector3,
    pub canopy_front: Vector3,
    pub canopy_peak: Vector3,
    pub spine_tail: Vector3,
    pub fuselage_left: Vector3,
    pub fuselage_right: Vector3,
    pub wing_root_left: Vector3,
    pub wing_root_right: Vector3,
    pub wing_tip_left: Vector3,
    pub wing_tip_right: Vector3,
    pub fin_top_left: Vector3,
    pub fin_top_right: Vector3,
    pub engine_nozzle_left: Vector3,
    pub engine_nozzle_right: Vector3,
    pub belly_keel: Vector3,
    pub canopy_glint_a: Vector3,
    pub canopy_glint_b: Vector3,
}

impl ShipVertices {
    pub fn from_ship(ship: &Ship) -> Self {
        Self {
            nose: ship.local_point(ship_dims::NOSE_FWD, 0.0, ship_dims::NOSE_UP),
            nose_chin_left: ship.local_point(
                ship_dims::NOSE_CHIN_FWD,
                -ship_dims::NOSE_CHIN_RIGHT,
                ship_dims::NOSE_CHIN_UP,
            ),
            nose_chin_right: ship.local_point(
                ship_dims::NOSE_CHIN_FWD,
                ship_dims::NOSE_CHIN_RIGHT,
                ship_dims::NOSE_CHIN_UP,
            ),
            canopy_front: ship.local_point(
                ship_dims::CANOPY_FRONT_FWD,
                0.0,
                ship_dims::CANOPY_FRONT_UP,
            ),
            canopy_peak: ship.local_point(
                ship_dims::CANOPY_PEAK_FWD,
                0.0,
                ship_dims::CANOPY_PEAK_UP,
            ),
            spine_tail: ship.local_point(ship_dims::SPINE_TAIL_FWD, 0.0, ship_dims::SPINE_TAIL_UP),
            fuselage_left: ship.local_point(
                ship_dims::FUSELAGE_FWD,
                -ship_dims::FUSELAGE_RIGHT,
                ship_dims::FUSELAGE_UP,
            ),
            fuselage_right: ship.local_point(
                ship_dims::FUSELAGE_FWD,
                ship_dims::FUSELAGE_RIGHT,
                ship_dims::FUSELAGE_UP,
            ),
            wing_root_left: ship.local_point(
                ship_dims::WING_ROOT_FWD,
                -ship_dims::WING_ROOT_RIGHT,
                ship_dims::WING_ROOT_UP,
            ),
            wing_root_right: ship.local_point(
                ship_dims::WING_ROOT_FWD,
                ship_dims::WING_ROOT_RIGHT,
                ship_dims::WING_ROOT_UP,
            ),
            wing_tip_left: ship.local_point(
                ship_dims::WING_TIP_FWD,
                -ship_dims::WING_TIP_RIGHT,
                ship_dims::WING_TIP_UP,
            ),
            wing_tip_right: ship.local_point(
                ship_dims::WING_TIP_FWD,
                ship_dims::WING_TIP_RIGHT,
                ship_dims::WING_TIP_UP,
            ),
            fin_top_left: ship.local_point(
                ship_dims::WINGLET_TOP_FWD,
                -ship_dims::WINGLET_TOP_RIGHT,
                ship_dims::WINGLET_TOP_UP,
            ),
            fin_top_right: ship.local_point(
                ship_dims::WINGLET_TOP_FWD,
                ship_dims::WINGLET_TOP_RIGHT,
                ship_dims::WINGLET_TOP_UP,
            ),
            engine_nozzle_left: ship.local_point(
                ship_dims::TAIL_FWD,
                -ship_dims::TAIL_RIGHT,
                ship_dims::TAIL_UP,
            ),
            engine_nozzle_right: ship.local_point(
                ship_dims::TAIL_FWD,
                ship_dims::TAIL_RIGHT,
                ship_dims::TAIL_UP,
            ),
            belly_keel: ship.local_point(ship_dims::BELLY_KEEL_FWD, 0.0, ship_dims::BELLY_KEEL_UP),
            canopy_glint_a: ship.local_point(
                ship_dims::CANOPY_GLINT_A_FWD,
                ship_dims::CANOPY_GLINT_A_RIGHT,
                ship_dims::CANOPY_GLINT_A_UP,
            ),
            canopy_glint_b: ship.local_point(
                ship_dims::CANOPY_GLINT_B_FWD,
                ship_dims::CANOPY_GLINT_B_RIGHT,
                ship_dims::CANOPY_GLINT_B_UP,
            ),
        }
    }
}

#[inline]
fn draw_double_triangle_3d(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    a: Vector3,
    b: Vector3,
    c: Vector3,
    col: Color,
) {
    d.draw_triangle3D(a, b, c, col);
    d.draw_triangle3D(a, c, b, col);
}

fn draw_nose_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    primary_col: Color,
    mid_col: Color,
    dark_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.nose,
        verts.nose_chin_left,
        verts.canopy_front,
        primary_col,
    );
    draw_double_triangle_3d(
        d,
        verts.nose,
        verts.canopy_front,
        verts.nose_chin_right,
        mid_col,
    );
    draw_double_triangle_3d(
        d,
        verts.nose_chin_left,
        verts.fuselage_left,
        verts.canopy_front,
        mid_col,
    );
    draw_double_triangle_3d(
        d,
        verts.nose_chin_right,
        verts.canopy_front,
        verts.fuselage_right,
        dark_col,
    );
}

fn draw_fuselage_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    mid_col: Color,
    dark_col: Color,
    accent_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.canopy_peak,
        verts.fuselage_left,
        verts.spine_tail,
        accent_col,
    );
    draw_double_triangle_3d(
        d,
        verts.canopy_peak,
        verts.spine_tail,
        verts.fuselage_right,
        accent_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_left,
        verts.engine_nozzle_left,
        verts.spine_tail,
        mid_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_right,
        verts.spine_tail,
        verts.engine_nozzle_right,
        dark_col,
    );
}

fn draw_wing_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    primary_col: Color,
    mid_col: Color,
    dark_col: Color,
    accent_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.nose_chin_left,
        verts.wing_root_left,
        verts.fuselage_left,
        primary_col,
    );
    draw_double_triangle_3d(
        d,
        verts.nose_chin_right,
        verts.fuselage_right,
        verts.wing_root_right,
        mid_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_left,
        verts.wing_root_left,
        verts.wing_tip_left,
        accent_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_right,
        verts.wing_tip_right,
        verts.wing_root_right,
        accent_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_left,
        verts.wing_tip_left,
        verts.engine_nozzle_left,
        dark_col,
    );
    draw_double_triangle_3d(
        d,
        verts.fuselage_right,
        verts.engine_nozzle_right,
        verts.wing_tip_right,
        dark_col,
    );
}

fn draw_winglet_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    primary_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.wing_root_left,
        verts.wing_tip_left,
        verts.fin_top_left,
        primary_col,
    );
    draw_double_triangle_3d(
        d,
        verts.wing_root_right,
        verts.fin_top_right,
        verts.wing_tip_right,
        primary_col,
    );
}

fn draw_underbelly_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    dark_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.nose,
        verts.belly_keel,
        verts.nose_chin_left,
        dark_col,
    );
    draw_double_triangle_3d(
        d,
        verts.nose,
        verts.nose_chin_right,
        verts.belly_keel,
        dark_col,
    );
    draw_double_triangle_3d(
        d,
        verts.belly_keel,
        verts.engine_nozzle_left,
        verts.engine_nozzle_right,
        dark_col,
    );
}

fn draw_ship_hull_faces(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    primary_col: Color,
    mid_col: Color,
    dark_col: Color,
    accent_col: Color,
) {
    draw_nose_faces(d, verts, primary_col, mid_col, dark_col);
    draw_fuselage_faces(d, verts, mid_col, dark_col, accent_col);
    draw_wing_faces(d, verts, primary_col, mid_col, dark_col, accent_col);
    draw_winglet_faces(d, verts, primary_col);
    draw_underbelly_faces(d, verts, dark_col);
}

fn draw_ship_cockpit_canopy(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    canopy_col: Color,
    _glint_col: Color,
) {
    draw_double_triangle_3d(
        d,
        verts.canopy_front,
        verts.fuselage_left,
        verts.canopy_peak,
        canopy_col,
    );
    draw_double_triangle_3d(
        d,
        verts.canopy_front,
        verts.canopy_peak,
        verts.fuselage_right,
        canopy_col,
    );
}

fn draw_ship_airbrake_flaps(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    ship: &Ship,
    accent_col: Color,
    ink_col: Color,
) {
    let flap_l_lift = Ship::airbrake_flap_lift(ship.flap_left);
    let flap_r_lift = Ship::airbrake_flap_lift(ship.flap_right);

    let flap_l_a = ship.local_point(
        ship_dims::AIRBRAKE_ROOT_FWD,
        -ship_dims::AIRBRAKE_INNER_RIGHT,
        ship_dims::AIRBRAKE_INNER_UP,
    );
    let flap_l_b = ship.local_point(
        ship_dims::AIRBRAKE_ROOT_FWD,
        -ship_dims::AIRBRAKE_OUTER_RIGHT,
        ship_dims::AIRBRAKE_OUTER_UP,
    );
    let flap_l_c = ship.local_point(
        ship_dims::AIRBRAKE_TIP_FWD,
        -ship_dims::AIRBRAKE_TIP_RIGHT,
        flap_l_lift,
    );

    let active_col = Color::new(255, 85, 140, accent_col.a);
    let l_col = if ship.flap_left > 0.1 {
        active_col
    } else {
        ink_col
    };
    draw_double_triangle_3d(d, flap_l_a, flap_l_b, flap_l_c, l_col);

    let flap_r_a = ship.local_point(
        ship_dims::AIRBRAKE_ROOT_FWD,
        ship_dims::AIRBRAKE_INNER_RIGHT,
        ship_dims::AIRBRAKE_INNER_UP,
    );
    let flap_r_b = ship.local_point(
        ship_dims::AIRBRAKE_ROOT_FWD,
        ship_dims::AIRBRAKE_OUTER_RIGHT,
        ship_dims::AIRBRAKE_OUTER_UP,
    );
    let flap_r_c = ship.local_point(
        ship_dims::AIRBRAKE_TIP_FWD,
        ship_dims::AIRBRAKE_TIP_RIGHT,
        flap_r_lift,
    );

    let r_col = if ship.flap_right > 0.1 {
        active_col
    } else {
        ink_col
    };
    draw_double_triangle_3d(d, flap_r_a, flap_r_c, flap_r_b, r_col);
}

fn draw_ship_ink_outlines(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    verts: &ShipVertices,
    ink_col: Color,
    glint_col: Color,
) {
    d.draw_cylinder_ex(
        verts.nose,
        verts.wing_root_left,
        MAIN_OUTLINE_RADIUS,
        MAIN_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.nose,
        verts.wing_root_right,
        MAIN_OUTLINE_RADIUS,
        MAIN_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_root_left,
        verts.wing_tip_left,
        MAIN_OUTLINE_RADIUS,
        MAIN_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_root_right,
        verts.wing_tip_right,
        MAIN_OUTLINE_RADIUS,
        MAIN_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_tip_left,
        verts.engine_nozzle_left,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_tip_right,
        verts.engine_nozzle_right,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_tip_left,
        verts.fin_top_left,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_tip_right,
        verts.fin_top_right,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_root_left,
        verts.fin_top_left,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.wing_root_right,
        verts.fin_top_right,
        WING_OUTLINE_RADIUS,
        WING_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.nose,
        verts.canopy_peak,
        SPINE_OUTLINE_RADIUS,
        SPINE_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );
    d.draw_cylinder_ex(
        verts.canopy_peak,
        verts.spine_tail,
        SPINE_OUTLINE_RADIUS,
        SPINE_OUTLINE_RADIUS,
        OUTLINE_SIDES,
        ink_col,
    );

    d.draw_cylinder_ex(
        verts.canopy_glint_a,
        verts.canopy_glint_b,
        GLINT_LINE_RADIUS,
        GLINT_LINE_RADIUS,
        OUTLINE_SIDES,
        glint_col,
    );
}

fn draw_ship_thruster_plumes(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    ship: &Ship,
    verts: &ShipVertices,
    is_boosting: bool,
    _accent_col: Color,
    alpha: u8,
) {
    let flame_len = ship.thruster_flame_length(is_boosting);
    let flame_col = if is_boosting {
        Color::new(70, 255, 255, alpha)
    } else {
        Color::new(255, 135, 35, alpha)
    };
    let flame_l_tip = ship.local_point(
        ship_dims::THRUSTER_BASE_FWD - flame_len,
        -ship_dims::THRUSTER_OFFSET_RIGHT,
        0.0,
    );
    let flame_r_tip = ship.local_point(
        ship_dims::THRUSTER_BASE_FWD - flame_len,
        ship_dims::THRUSTER_OFFSET_RIGHT,
        0.0,
    );

    d.draw_cylinder_ex(
        verts.engine_nozzle_left,
        flame_l_tip,
        PLUME_BASE_RADIUS,
        PLUME_TIP_RADIUS,
        PLUME_SIDES,
        flame_col,
    );
    d.draw_cylinder_ex(
        verts.engine_nozzle_right,
        flame_r_tip,
        PLUME_BASE_RADIUS,
        PLUME_TIP_RADIUS,
        PLUME_SIDES,
        flame_col,
    );
}

fn draw_ship_wingtip_contrails(
    d: &mut RaylibMode3D<RaylibDrawHandle>,
    ship: &Ship,
    is_boosting: bool,
    accent_col: Color,
) {
    let count = ship.trail_history.len();
    if count <= 2 || ship.v_s <= CONTRAIL_MIN_SPEED {
        return;
    }
    for k in 0..(count - 1) {
        let (l0, r0) = ship.trail_history[k];
        let (l1, r1) = ship.trail_history[k + 1];
        if (l1 - l0).length_sq() > CONTRAIL_MAX_GAP_SQ {
            continue;
        }
        let trail_alpha = ((k as f32 / count as f32) * CONTRAIL_MAX_ALPHA) as u8;
        let t_col = if is_boosting {
            Color::new(60, 250, 255, trail_alpha)
        } else {
            Color::new(accent_col.r, accent_col.g, accent_col.b, trail_alpha)
        };
        d.draw_line_3D(l0.to_rl(), l1.to_rl(), t_col);
        d.draw_line_3D(r0.to_rl(), r1.to_rl(), t_col);
    }
}

pub fn draw_cel_ship(d: &mut RaylibMode3D<RaylibDrawHandle>, ship: &Ship, alpha: u8) {
    let verts = ShipVertices::from_ship(ship);
    let is_boosting = ship.boost_timer > 0.0;

    let col_main = rgba_color(ship.primary_color, alpha);
    let col_mid = scale_color_brightness(col_main, MID_SHADE_FACTOR);
    let col_dark = scale_color_brightness(col_main, DARK_SHADE_FACTOR);
    let col_accent = rgba_color(ship.accent_color, alpha);
    let col_canopy = Color::new(30, 235, 255, alpha);
    let col_ink = Color::new(10, 12, 18, alpha);
    let col_glint = Color::WHITE;

    draw_ship_hull_faces(d, &verts, col_main, col_mid, col_dark, col_accent);
    draw_ship_cockpit_canopy(d, &verts, col_canopy, col_glint);
    draw_ship_airbrake_flaps(d, ship, col_accent, col_dark);

    if alpha > SOLID_SHIP_MIN_ALPHA {
        draw_ship_ink_outlines(d, &verts, col_ink, col_glint);
    }

    draw_ship_thruster_plumes(d, ship, &verts, is_boosting, col_accent, alpha);
    if alpha > SOLID_SHIP_MIN_ALPHA {
        draw_ship_wingtip_contrails(d, ship, is_boosting, col_accent);
    }
}
