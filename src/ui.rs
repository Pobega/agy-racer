use crate::ghost::GhostManager;
use crate::math::Vec3;
use crate::net::NetSession;
use crate::physics::Ship;
use crate::track::{BakedTrack, SurfaceFrame, SurfaceType};
use raylib::prelude::*;

pub const SPEED_LINES_MIN_SPEED: f32 = 68.0;
pub const SPEED_LINES_SPEED_RANGE: f32 = 25.0;
pub const SPEED_LINES_BOOST_RAY_COUNT: usize = 36;
pub const SPEED_LINES_NORMAL_RAY_COUNT: usize = 20;
pub const SPEED_LINES_CENTER_Y_RATIO: f32 = 0.48;
pub const SPEED_LINES_OUTER_RADIUS_RATIO: f32 = 0.75;
pub const SPEED_LINES_MAX_NORMAL_INTENSITY: f32 = 0.65;
pub const SPEED_LINES_PHASE_SCALE: f32 = 0.35;
pub const SPEED_LINES_ASPECT_SQUISH: f32 = 0.68;
pub const SPEED_LINES_TIP_ANGLE_WIDTH: f32 = 0.018;
pub const MPS_TO_DISPLAY_KMH: f32 = 12.0;
pub const MAX_SPEEDOMETER_KMH: f32 = 1400.0;
pub const SPEEDOMETER_TICK_COUNT: i32 = 16;
pub const SPEEDOMETER_REDLINE_TICK: i32 = 12;
pub const LOW_SHIELD_FRACTION_THRESHOLD: f32 = 0.35;
pub const TURN_CALLOUT_LOOKAHEAD_METERS: f32 = 55.0;
pub const SHARP_TURN_DOT_THRESHOLD: f32 = 0.45;
pub const MEDIUM_TURN_DOT_THRESHOLD: f32 = 0.22;
pub const MINIMAP_RADIUS: f32 = 82.0;
pub const MINIMAP_PADDING: f32 = 14.0;
pub const MINIMAP_WORLD_SCALE: f32 = 0.22;
pub const MINIMAP_ORIGIN_X: f32 = 240.0;
pub const MINIMAP_ORIGIN_Z: f32 = 110.0;
pub const MINIMAP_SAMPLE_STRIDE: usize = 4;
pub const MINIMAP_PLAYER_DOT_RADIUS: f32 = 5.0;
pub const MINIMAP_RIVAL_DOT_RADIUS: f32 = 3.5;
pub const CYLINDER_HUD_CURVATURE: f32 = 0.85;
pub const HALFPIPE_HUD_CURVATURE: f32 = 0.2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapBounds {
    pub origin_x: f32,
    pub origin_z: f32,
    pub scale: f32,
}

impl Default for MinimapBounds {
    fn default() -> Self {
        Self {
            origin_x: MINIMAP_ORIGIN_X,
            origin_z: MINIMAP_ORIGIN_Z,
            scale: MINIMAP_WORLD_SCALE,
        }
    }
}

pub fn compute_minimap_bounds(track: &BakedTrack) -> MinimapBounds {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    let mut sample_count = 0;

    for seg in &track.segments {
        for s in &seg.samples {
            min_x = min_x.min(s.pos.x);
            max_x = max_x.max(s.pos.x);
            min_z = min_z.min(s.pos.z);
            max_z = max_z.max(s.pos.z);
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return MinimapBounds::default();
    }

    let span_x = (max_x - min_x).max(1.0);
    let span_z = (max_z - min_z).max(1.0);
    let max_span = span_x.max(span_z);
    let available_radius = (MINIMAP_RADIUS - MINIMAP_PADDING).max(10.0);
    let scale = (available_radius * 2.0) / max_span;

    MinimapBounds {
        origin_x: (min_x + max_x) * 0.5,
        origin_z: (min_z + max_z) * 0.5,
        scale,
    }
}

pub fn format_lap_time(seconds: f32) -> String {
    let total_ms = (seconds.max(0.0) * 1000.0) as u32;
    let mins = total_ms / 60_000;
    let secs = (total_ms % 60_000) / 1000;
    let hundredths = (total_ms % 1000) / 10;
    format!("{:02}:{:02}.{:02}", mins, secs, hundredths)
}

pub fn draw_hud_panel_box(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    border_color: Color,
) {
    d.draw_rectangle(x, y, width, height, Color::new(8, 12, 26, 205));
    d.draw_rectangle_lines(x, y, width, height, border_color);
}

pub fn compute_player_race_rank(player: &Ship, ships: &[Ship], track: &BakedTrack) -> usize {
    let mut rank = 1usize;
    let player_progress = player.race_progress(track);
    for rival in ships.iter().skip(1) {
        if rival.race_progress(track) > player_progress {
            rank += 1;
        }
    }
    rank
}

pub fn describe_active_track_section(
    player: &Ship,
    frame: &SurfaceFrame,
) -> (&'static str, Color) {
    if player.boost_timer > 0.0 {
        ("HYPER BOOST ENGAGED!", Color::new(0, 240, 255, 255))
    } else if player.is_airborne {
        ("AIRBORNE GLIDE [PITCH UP/DOWN]", Color::new(255, 220, 60, 255))
    } else if frame.curvature > CYLINDER_HUD_CURVATURE {
        ("360 CYLINDER TUBE LOCK", Color::new(255, 110, 180, 255))
    } else if frame.curvature > HALFPIPE_HUD_CURVATURE {
        ("HALF-PIPE CHANNEL", Color::new(110, 255, 190, 255))
    } else if frame.surface == SurfaceType::RechargeStrip {
        ("RECHARGING SHIELD ENERGY...", Color::new(40, 235, 130, 255))
    } else {
        (
            "[SPACE]: BOOST   [Q/E]: AIRBRAKES   [F5]: HOT-RELOAD",
            Color::new(255, 235, 90, 255),
        )
    }
}

pub fn detect_upcoming_turn_callout(
    player: &Ship,
    track: &BakedTrack,
) -> Option<(&'static str, Color)> {
    let seg = track.segment_or_main(player.segment_id);
    let s_now = seg.sample_at(player.s);
    let s_ahead = seg.sample_at(player.s + TURN_CALLOUT_LOOKAHEAD_METERS);
    let turn_val = s_now.right.dot(s_ahead.forward);
    if turn_val.abs() <= MEDIUM_TURN_DOT_THRESHOLD {
        return None;
    }

    let (turn_txt, turn_col) = if turn_val > SHARP_TURN_DOT_THRESHOLD {
        ("HARD RIGHT >>>", Color::new(255, 90, 90, 255))
    } else if turn_val > MEDIUM_TURN_DOT_THRESHOLD {
        ("RIGHT TURN >>", Color::new(255, 220, 60, 255))
    } else if turn_val < -SHARP_TURN_DOT_THRESHOLD {
        ("<<< HARD LEFT", Color::new(255, 90, 90, 255))
    } else {
        ("<< LEFT TURN", Color::new(255, 220, 60, 255))
    };
    Some((turn_txt, turn_col))
}

pub fn world_pos_to_minimap_point(
    pos: Vec3,
    center_x: f32,
    center_y: f32,
    bounds: &MinimapBounds,
) -> Vector2 {
    let x = center_x + (pos.x - bounds.origin_x) * bounds.scale;
    let y = center_y - (pos.z - bounds.origin_z) * bounds.scale;
    Vector2::new(x, y)
}

pub fn compute_speed_line_vertices(
    cx: f32,
    cy: f32,
    inner_r: f32,
    outer_r: f32,
    angle: f32,
) -> (Vector2, Vector2, Vector2) {
    let x0 = cx + angle.cos() * inner_r;
    let y0 = cy + angle.sin() * inner_r * SPEED_LINES_ASPECT_SQUISH;
    let x1 = cx + angle.cos() * outer_r;
    let y1 = cy + angle.sin() * outer_r * SPEED_LINES_ASPECT_SQUISH;
    let tip_angle = angle + SPEED_LINES_TIP_ANGLE_WIDTH;
    let x2 = cx + tip_angle.cos() * outer_r;
    let y2 = cy + tip_angle.sin() * outer_r * SPEED_LINES_ASPECT_SQUISH;
    (Vector2::new(x0, y0), Vector2::new(x1, y1), Vector2::new(x2, y2))
}

pub fn draw_boost_speed_lines(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    player: &Ship,
) {
    if player.v_s <= SPEED_LINES_MIN_SPEED && player.boost_timer <= 0.0 {
        return;
    }

    let cx = screen_w as f32 * 0.5;
    let cy = screen_h as f32 * SPEED_LINES_CENTER_Y_RATIO;
    let outer_r = (screen_w.max(screen_h) as f32) * SPEED_LINES_OUTER_RADIUS_RATIO;
    let intensity = if player.boost_timer > 0.0 {
        1.0
    } else {
        ((player.v_s - SPEED_LINES_MIN_SPEED) / SPEED_LINES_SPEED_RANGE)
            .clamp(0.0, SPEED_LINES_MAX_NORMAL_INTENSITY)
    };
    let ray_count = if player.boost_timer > 0.0 {
        SPEED_LINES_BOOST_RAY_COUNT
    } else {
        SPEED_LINES_NORMAL_RAY_COUNT
    };
    let phase = (player.s * SPEED_LINES_PHASE_SCALE) as i32;

    for r_idx in 0..ray_count {
        let hash = (((r_idx as i32) * 97 + phase * 31) & 255) as f32 / 255.0;
        let angle = (r_idx as f32 / ray_count as f32) * std::f32::consts::TAU + hash * 0.15;
        let inner_r = (screen_h as f32) * (0.42 + (1.0 - intensity) * 0.14 + (hash * 0.08));

        let (p0, p1, p2) = compute_speed_line_vertices(cx, cy, inner_r, outer_r, angle);

        let ray_col = if r_idx % 3 == 0 && player.boost_timer > 0.0 {
            Color::new(0, 240, 255, (165.0 * intensity) as u8)
        } else if r_idx % 2 == 0 {
            Color::new(255, 255, 255, (150.0 * intensity) as u8)
        } else {
            Color::new(12, 14, 22, (185.0 * intensity) as u8)
        };
        d.draw_triangle(p0, p1, p2, ray_col);
        d.draw_triangle(p0, p2, p1, ray_col);
    }
}

pub fn draw_lap_and_position_panel(
    d: &mut RaylibDrawHandle,
    player: &Ship,
    ships: &[Ship],
    track: &BakedTrack,
    best_ghost_time: Option<f32>,
    track_name: &str,
) {
    let rank = compute_player_race_rank(player, ships, track);
    let rank_str = match rank {
        1 => "1ST",
        2 => "2ND",
        3 => "3RD",
        _ => "4TH",
    };

    draw_hud_panel_box(d, 16, 14, 285, 108, Color::new(0, 225, 255, 220));

    d.draw_text(
        &format!("LAP  {}/{}", player.current_lap, player.total_laps),
        28,
        22,
        24,
        Color::new(0, 240, 255, 255),
    );
    d.draw_text(
        rank_str,
        215,
        20,
        28,
        if rank == 1 {
            Color::new(255, 230, 50, 255)
        } else {
            Color::new(255, 110, 180, 255)
        },
    );

    d.draw_text(
        &format!("TIME   {}", format_lap_time(player.lap_time_sec)),
        28,
        54,
        18,
        Color::WHITE,
    );

    let best_str = player
        .best_lap_sec
        .or(best_ghost_time)
        .map(format_lap_time)
        .unwrap_or_else(|| "--:--.--".to_string());
    d.draw_text(
        &format!("BEST   {}  [GHOST]", best_str),
        28,
        78,
        17,
        Color::new(110, 255, 190, 255),
    );
    d.draw_text(
        track_name,
        28,
        101,
        13,
        Color::new(170, 180, 210, 255),
    );
}

pub fn draw_shield_and_boost_meter(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    player: &Ship,
    track: &BakedTrack,
) {
    let bar_w = 270;
    let bar_h = 26;
    let bx = screen_w - bar_w - 20;
    let by = 18;

    draw_hud_panel_box(d, bx - 6, by - 6, bar_w + 12, 68, Color::new(255, 50, 170, 220));

    let energy_frac = (player.shield_energy / 100.0).clamp(0.0, 1.0);
    let fill_w = (bar_w as f32 * energy_frac) as i32;
    let bar_col = if player.boost_timer > 0.0 {
        Color::new(60, 255, 255, 255)
    } else if energy_frac > LOW_SHIELD_FRACTION_THRESHOLD {
        Color::new(40, 235, 130, 255)
    } else {
        Color::new(255, 65, 65, 255)
    };

    d.draw_rectangle(bx, by, bar_w, bar_h, Color::new(28, 32, 48, 255));
    d.draw_rectangle(bx, by, fill_w, bar_h, bar_col);
    d.draw_rectangle_lines(bx, by, bar_w, bar_h, Color::WHITE);

    d.draw_text(
        &format!("SHIELD / BOOST POWER: {:3.0}%", player.shield_energy),
        bx + 8,
        by + 5,
        16,
        Color::BLACK,
    );

    let sample = track.segment_or_main(player.segment_id).sample_at(player.s);
    let frame = sample.evaluate_cross_section(player.r);
    let (status_txt, status_col) = describe_active_track_section(player, &frame);
    d.draw_text(status_txt, bx + 4, by + 34, 13, status_col);
}

pub fn draw_digital_speedometer(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    player: &Ship,
) {
    let kmh = (player.v_s * MPS_TO_DISPLAY_KMH) as i32;
    let sx = screen_w - 235;
    let sy = screen_h - 92;
    draw_hud_panel_box(d, sx, sy, 215, 74, Color::new(0, 240, 255, 220));

    d.draw_text(
        &format!("{:04}", kmh),
        sx + 16,
        sy + 10,
        42,
        if player.boost_timer > 0.0 {
            Color::new(80, 255, 255, 255)
        } else {
            Color::new(255, 245, 255, 255)
        },
    );
    d.draw_text("KM/H", sx + 135, sy + 28, 20, Color::new(0, 225, 255, 255));

    let active_ticks = ((kmh as f32 / MAX_SPEEDOMETER_KMH) * SPEEDOMETER_TICK_COUNT as f32)
        .clamp(0.0, SPEEDOMETER_TICK_COUNT as f32) as i32;
    for t in 0..SPEEDOMETER_TICK_COUNT {
        let tc = if t < active_ticks {
            if t >= SPEEDOMETER_REDLINE_TICK {
                Color::new(255, 60, 170, 255)
            } else {
                Color::new(0, 240, 255, 255)
            }
        } else {
            Color::new(35, 42, 65, 255)
        };
        d.draw_rectangle(sx + 14 + t * 12, sy + 56, 9, 10, tc);
    }
}

pub fn draw_upcoming_turn_warning(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    player: &Ship,
    track: &BakedTrack,
) {
    if let Some((turn_txt, turn_col)) = detect_upcoming_turn_callout(player, track) {
        let tw = d.measure_text(turn_txt, 26);
        d.draw_text(turn_txt, (screen_w - tw) / 2, 54, 26, turn_col);
    }
}

pub fn draw_circuit_minimap(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    track: &BakedTrack,
    ships: &[Ship],
) {
    let bounds = compute_minimap_bounds(track);
    let map_cx = (screen_w - 140) as f32;
    let map_cy = 185.0f32;
    d.draw_circle(map_cx as i32, map_cy as i32, MINIMAP_RADIUS, Color::new(8, 12, 26, 185));
    d.draw_circle_lines(map_cx as i32, map_cy as i32, MINIMAP_RADIUS, Color::new(0, 225, 255, 190));

    for t_seg in &track.segments {
        let n = t_seg.samples.len();
        if n < 2 {
            continue;
        }
        for k in (0..n).step_by(MINIMAP_SAMPLE_STRIDE) {
            let kn = (k + MINIMAP_SAMPLE_STRIDE) % n;
            if !t_seg.closed_loop && kn < k {
                break;
            }
            let p0 = world_pos_to_minimap_point(t_seg.samples[k].pos, map_cx, map_cy, &bounds);
            let p1 = world_pos_to_minimap_point(t_seg.samples[kn].pos, map_cx, map_cy, &bounds);
            d.draw_line(
                p0.x as i32,
                p0.y as i32,
                p1.x as i32,
                p1.y as i32,
                if t_seg.id == 0 {
                    Color::new(0, 225, 255, 220)
                } else {
                    Color::new(255, 90, 200, 200)
                },
            );
        }
    }
    for (idx, ship) in ships.iter().enumerate() {
        let pt = world_pos_to_minimap_point(ship.world_pos, map_cx, map_cy, &bounds);
        let dot_col = if idx == 0 {
            Color::new(255, 255, 60, 255)
        } else {
            Color::new(255, 70, 120, 255)
        };
        d.draw_circle(
            pt.x as i32,
            pt.y as i32,
            if idx == 0 {
                MINIMAP_PLAYER_DOT_RADIUS
            } else {
                MINIMAP_RIVAL_DOT_RADIUS
            },
            dot_col,
        );
    }
}

pub fn draw_biome_and_controls_banners(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    biome: &str,
    net_status: &NetSession,
) {
    let biome_banner = format!("BIOME [Press B]: {}   |   [F5] Hot-Reload Config/Textures", biome);
    let sb_w = d.measure_text(&biome_banner, 14) + 24;
    let sb_x = (screen_w - sb_w) / 2;
    d.draw_rectangle(sb_x, 14, sb_w, 28, Color::new(8, 12, 26, 215));
    d.draw_rectangle_lines(sb_x, 14, sb_w, 28, Color::new(255, 220, 60, 220));
    d.draw_text(
        &biome_banner,
        sb_x + 12,
        21,
        14,
        Color::new(255, 235, 90, 255),
    );

    let net_label = if net_status.is_online {
        format!(
            "ONLINE P2P ROOM: [{}] (Peers: {})  |  [F2] Offline",
            net_status.room_code, net_status.peer_count
        )
    } else {
        format!(
            "ROOM CODE: [{}] (Offline/AI)  |  [F2] Host/Join P2P",
            net_status.room_code
        )
    };
    d.draw_rectangle(16, screen_h - 62, 535, 48, Color::new(8, 12, 26, 195));
    d.draw_text(
        &net_label,
        26,
        screen_h - 54,
        15,
        if net_status.is_online {
            Color::new(90, 255, 150, 255)
        } else {
            Color::new(190, 205, 235, 255)
        },
    );
    d.draw_text(
        "[W/Up] Thrust  [A/D] Steer  [Q/E] L/R Airbrake  [S/Shift] Pitch  [Space] Boost  [TAB] Track Editor",
        26,
        screen_h - 33,
        12,
        Color::new(150, 175, 210, 255),
    );
}

pub fn draw_race_hud(
    d: &mut RaylibDrawHandle,
    screen_w: i32,
    screen_h: i32,
    ships: &[Ship],
    track: &BakedTrack,
    ghost_mgr: &GhostManager,
    net: &NetSession,
    biome_name: &str,
) {
    let Some(player) = ships.first() else {
        return;
    };

    draw_boost_speed_lines(d, screen_w, screen_h, player);
    let best_ghost_time = ghost_mgr.best_ghost.as_ref().map(|g| g.lap_time_sec);
    draw_lap_and_position_panel(d, player, ships, track, best_ghost_time, &track.def.name);
    draw_shield_and_boost_meter(d, screen_w, player, track);
    draw_digital_speedometer(d, screen_w, screen_h, player);
    draw_upcoming_turn_warning(d, screen_w, player, track);
    draw_circuit_minimap(d, screen_w, track, ships);
    draw_biome_and_controls_banners(d, screen_w, screen_h, biome_name, net);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_lap_time_zero_and_fractions() {
        assert_eq!(format_lap_time(0.0), "00:00.00");
        assert_eq!(format_lap_time(-5.0), "00:00.00");
        assert_eq!(format_lap_time(61.25), "01:01.25");
        assert_eq!(format_lap_time(125.89), "02:05.89");
    }

    #[test]
    fn test_minimap_bounds_default_and_projection() {
        let mut def = BakedTrack::default_showcase_track();
        def.segments.clear();
        let empty_track = BakedTrack {
            def,
            segments: Vec::new(),
            primary_lap_length: 0.0,
        };
        let bounds = compute_minimap_bounds(&empty_track);
        assert_eq!(bounds, MinimapBounds::default());

        let pt = world_pos_to_minimap_point(
            Vec3::new(bounds.origin_x, 0.0, bounds.origin_z),
            100.0,
            100.0,
            &bounds,
        );
        assert!((pt.x - 100.0).abs() < 1e-4);
        assert!((pt.y - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_speed_line_vertices() {
        let (v0, v1, v2) = compute_speed_line_vertices(100.0, 200.0, 50.0, 150.0, 0.0);
        assert!((v0.x - 150.0).abs() < 1e-4);
        assert!((v0.y - 200.0).abs() < 1e-4);
        assert!((v1.x - 250.0).abs() < 1e-4);
        assert!((v1.y - 200.0).abs() < 1e-4);
        assert!(v2.y > v1.y);
    }
}
