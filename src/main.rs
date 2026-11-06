mod config;
mod editor;
mod ghost;
mod math;
mod net;
mod physics;
mod render;
mod track;
mod ui;

use config::{BiomePalette, PhysicsConfig, ShipRosterEntry};
use editor::TrackEditor;
use ghost::GhostManager;
use math::Vec3;
use net::NetSession;
use physics::{Ship, ShipInput};
use raylib::prelude::*;
use render::RenderAssets;
use std::fs;
use std::path::Path;
use track::{BakedTrack, SurfaceFrame, TrackDef, ACTIVE_TRACK_PATH};

pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;
pub const TARGET_FPS: u32 = 60;
pub const PHYSICS_TICK_DT: f32 = 1.0 / 60.0;
pub const FIXED_DT: f32 = PHYSICS_TICK_DT;
pub const MAX_PHYSICS_ACCUM_SEC: f32 = 0.20;
pub const GAMEPAD_STEER_DEADZONE: f32 = 0.12;
pub const GAMEPAD_TRIGGER_DEADZONE: f32 = 0.15;
pub const DEFAULT_ROOM_CODE: &str = "AGY4";

pub const DEFAULT_CAM_POS: Vec3 = Vec3::new(0.0, 5.0, -10.0);
pub const DEFAULT_CAM_TARGET: Vec3 = Vec3::new(0.0, 2.0, 20.0);
pub const DEFAULT_CAM_FOV: f32 = 65.0;
pub const CAM_YAW_LEAD_FACTOR: f32 = 0.9;
pub const CAM_TARGET_SHIP_WEIGHT: f32 = 0.35;
pub const CAM_TARGET_LOOKAHEAD_WEIGHT: f32 = 0.65;
pub const CAM_TARGET_LATERAL_FACTOR: f32 = 0.5;
pub const CAM_TARGET_NORMAL_OFFSET: f32 = 1.4;
pub const CAM_FOV_REF_SPEED: f32 = 90.0;
pub const CAM_FOV_SPEED_BONUS: f32 = 8.0;
pub const CAM_POS_SMOOTH_RATE: f32 = 16.0;
pub const CAM_UP_SMOOTH_RATE: f32 = 10.0;
pub const CAM_FOV_SMOOTH_RATE: f32 = 8.0;
pub const MIN_FRAME_DT: f32 = 0.001;
pub const MAX_FRAME_DT: f32 = 0.10;
pub const OPAQUE_SHIP_ALPHA: u8 = 255;

/// Smooth chase camera that follows the player ship along track contours.
pub struct ChaseCamera {
    pub pos: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
}

impl ChaseCamera {
    pub fn new() -> Self {
        Self {
            pos: DEFAULT_CAM_POS,
            target: DEFAULT_CAM_TARGET,
            up: Vec3::UP,
            fov: DEFAULT_CAM_FOV,
        }
    }

    pub fn compute_ideal_position(
        player: &Ship,
        frame: &SurfaceFrame,
        cfg: &PhysicsConfig,
    ) -> Vec3 {
        player.world_pos - frame.forward * cfg.camera_distance
            + frame.normal * cfg.camera_height
            - frame.right * (player.yaw_offset * CAM_YAW_LEAD_FACTOR)
    }

    pub fn compute_lookahead_target(
        player: &Ship,
        frame: &SurfaceFrame,
        track: &BakedTrack,
        cfg: &PhysicsConfig,
    ) -> Vec3 {
        let seg = track.segment_or_main(player.segment_id);
        let lookahead_sample = seg.sample_at(player.s + cfg.camera_lookahead);
        player.world_pos * CAM_TARGET_SHIP_WEIGHT
            + (lookahead_sample.pos + frame.right * (player.r * CAM_TARGET_LATERAL_FACTOR))
                * CAM_TARGET_LOOKAHEAD_WEIGHT
            + frame.normal * CAM_TARGET_NORMAL_OFFSET
    }

    pub fn compute_target_fov(player: &Ship, cfg: &PhysicsConfig) -> f32 {
        if player.boost_timer > 0.0 {
            cfg.camera_boost_fov
        } else {
            cfg.camera_base_fov + (player.v_s / CAM_FOV_REF_SPEED) * CAM_FOV_SPEED_BONUS
        }
    }

    pub fn update(
        &mut self,
        player: &Ship,
        track: &BakedTrack,
        cfg: &PhysicsConfig,
        dt: f32,
    ) {
        let seg = track.segment_or_main(player.segment_id);
        let sample = seg.sample_at(player.s);
        let frame = sample.evaluate_cross_section(player.r);

        let desired_pos = Self::compute_ideal_position(player, &frame, cfg);
        let desired_target = Self::compute_lookahead_target(player, &frame, track, cfg);
        let desired_up = frame.normal;

        let blend = (CAM_POS_SMOOTH_RATE * dt).clamp(0.0, 1.0);
        self.pos = self.pos.lerp(desired_pos, blend);
        self.target = self.target.lerp(desired_target, blend);
        self.up = self.up.lerp(desired_up, (CAM_UP_SMOOTH_RATE * dt).clamp(0.0, 1.0)).normalized();

        let target_fov = Self::compute_target_fov(player, cfg);
        self.fov += (target_fov - self.fov) * (CAM_FOV_SMOOTH_RATE * dt);
    }

    pub fn to_raylib(&self) -> Camera3D {
        Camera3D::perspective(
            self.pos.to_rl(),
            self.target.to_rl(),
            self.up.to_rl(),
            self.fov,
        )
    }
}

fn load_or_create_default_track() -> TrackDef {
    let default_path = ACTIVE_TRACK_PATH;
    if Path::new(default_path).exists() {
        if let Ok(json) = fs::read_to_string(default_path) {
            if let Ok(mut def) = serde_json::from_str::<TrackDef>(&json) {
                def.file_path = default_path.to_string();
                return def;
            }
        }
    }
    let mut def = BakedTrack::default_showcase_track();
    def.file_path = default_path.to_string();
    let _ = fs::create_dir_all("assets/tracks");
    if let Ok(json) = serde_json::to_string_pretty(&def) {
        let _ = fs::write(default_path, json);
    }
    def
}

fn read_player_input(rl: &RaylibHandle) -> ShipInput {
    let mut throttle: f32 = 0.0;
    let mut steer: f32 = 0.0;
    let mut airbrake_left: f32 = 0.0;
    let mut airbrake_right: f32 = 0.0;
    let mut pitch: f32 = 0.0;
    let mut boost_pressed = false;
    let mut side_attack_dir: i8 = 0;

    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        throttle = 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        steer -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        steer += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_Q) {
        airbrake_left = 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_E) {
        airbrake_right = 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_K) {
        pitch += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_I) {
        pitch -= 1.0;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        boost_pressed = true;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_Z) {
        side_attack_dir = -1;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_C) {
        side_attack_dir = 1;
    }

    if rl.is_gamepad_available(0) {
        let gp_x = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let gp_y = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        if gp_x.abs() > GAMEPAD_STEER_DEADZONE {
            steer = gp_x.clamp(-1.0, 1.0);
        }
        if gp_y.abs() > GAMEPAD_TRIGGER_DEADZONE {
            pitch = gp_y.clamp(-1.0, 1.0);
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
            steer = -1.0;
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT) {
            steer = 1.0;
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
            || rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT)
        {
            throttle = 1.0;
        }
        if rl.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT)
            || rl.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP)
        {
            boost_pressed = true;
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1)
            || rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_2)
        {
            airbrake_left = 1.0;
        }
        if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
            || rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2)
        {
            airbrake_right = 1.0;
        }
    }

    ShipInput {
        throttle,
        steer,
        airbrake_left,
        airbrake_right,
        pitch,
        boost_pressed,
        side_attack_dir,
    }
}

fn spawn_race_roster(track: &BakedTrack) -> Vec<Ship> {
    let entries = ShipRosterEntry::load_all();
    let mut ships: Vec<Ship> = entries
        .iter()
        .map(|entry| {
            Ship::new(
                entry.id,
                &entry.name,
                entry.primary_color,
                entry.accent_color,
                entry.start_s,
                entry.start_r,
                entry.is_ai,
            )
        })
        .collect();
    for s in &mut ships {
        s.update_world_transform(track);
    }
    ships
}

pub fn reload_all_assets_and_configs(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    phys_cfg: &mut PhysicsConfig,
    biomes: &mut Vec<BiomePalette>,
    active_biome_idx: &mut usize,
    assets: &mut RenderAssets,
) {
    *phys_cfg = PhysicsConfig::load();
    *biomes = BiomePalette::load_all();
    if !biomes.is_empty() {
        *active_biome_idx %= biomes.len();
    }
    *assets = RenderAssets::load_or_export(rl, thread);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("AGY-RACER // Cel-Shaded Antigravity Spline Racer")
        .resizable()
        .msaa_4x()
        .build();

    rl.set_target_fps(TARGET_FPS);

    let mut phys_cfg = PhysicsConfig::load();
    let mut biomes = BiomePalette::load_all();
    let mut biome_idx = 0usize;

    let mut track_def = load_or_create_default_track();
    track_def.apply_biome(&biomes[biome_idx]);
    let mut track = BakedTrack::from_def(track_def.clone());
    let mut ships = spawn_race_roster(&track);
    let mut ghost_mgr = GhostManager::new(&track.def.name);
    let mut editor = TrackEditor::new();
    let mut net = NetSession::new(DEFAULT_ROOM_CODE);
    let mut assets = RenderAssets::load_or_export(&mut rl, &thread);

    let mut chase_cam = ChaseCamera::new();
    chase_cam.fov = phys_cfg.camera_base_fov;

    let mut phys_accum = 0.0f32;
    let mut elapsed_sec = 0.0f32;

    while !rl.window_should_close() {
        let frame_dt = rl.get_frame_time().clamp(MIN_FRAME_DT, MAX_FRAME_DT);
        elapsed_sec += frame_dt;

        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            editor.active = !editor.active;
            if editor.active {
                if let Some(p) = ships.first() {
                    let cur_cp = track.segment_or_main(p.segment_id).sample_at(p.s).cp_index;
                    editor.selected_seg = p.segment_id;
                    editor.selected_cp = cur_cp;
                }
                editor.focus_on_selected(&track_def);
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            biome_idx = (biome_idx + 1) % biomes.len();
            track_def.apply_biome(&biomes[biome_idx]);
            track.def = track_def.clone();
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F5) {
            reload_all_assets_and_configs(
                &mut rl,
                &thread,
                &mut phys_cfg,
                &mut biomes,
                &mut biome_idx,
                &mut assets,
            );
            track_def.apply_biome(&biomes[biome_idx]);
            track.def = track_def.clone();
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            net.toggle_online();
        }

        if !editor.active && rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            ships = spawn_race_roster(&track);
        }

        if editor.active {
            if editor.update(&rl, &mut track_def, frame_dt) {
                track = BakedTrack::from_def(track_def.clone());
                for s in &mut ships {
                    s.update_world_transform(&track);
                }
            }
        } else {
            phys_accum = (phys_accum + frame_dt).min(MAX_PHYSICS_ACCUM_SEC);
            let mut player_input = read_player_input(&rl);

            while phys_accum >= PHYSICS_TICK_DT {
                phys_accum -= PHYSICS_TICK_DT;

                for i in 0..ships.len() {
                    let input = if i == 0 {
                        player_input
                    } else {
                        ships[i].compute_ai_input(&track, elapsed_sec)
                    };
                    ships[i].step_physics(input, &track, &phys_cfg, PHYSICS_TICK_DT);
                }

                Ship::resolve_ship_collisions(&mut ships, &track, &phys_cfg);
                ghost_mgr.update_recording(&ships[0], &track.def.name, PHYSICS_TICK_DT);
                net.tick_sync(&mut ships, &track, PHYSICS_TICK_DT);

                player_input.boost_pressed = false;
                player_input.side_attack_dir = 0;
            }

            chase_cam.update(&ships[0], &track, &phys_cfg, frame_dt);
        }

        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        render::draw_sky_backdrop(&mut d, screen_w, screen_h, &biomes[biome_idx]);

        let camera = if editor.active {
            editor.camera3d()
        } else {
            chase_cam.to_raylib()
        };

        let active_cam_pos = if editor.active {
            editor.cam_pos
        } else {
            chase_cam.pos
        };

        {
            let mut d3 = d.begin_mode3D(camera);

            render::draw_world_3d(
                &mut d3,
                &track,
                &ships,
                active_cam_pos,
                elapsed_sec,
                &assets,
                &biomes[biome_idx],
            );

            ghost_mgr.draw_ghost(&mut d3, &track, ships[0].lap_time_sec);
            for ship in &ships {
                render::draw_cel_ship(&mut d3, ship, OPAQUE_SHIP_ALPHA);
            }

            if editor.active {
                editor.draw_gizmos(&mut d3, &track);
            }
        }

        if editor.active {
            editor.draw_hud(&mut d, &track_def);
        } else {
            ui::draw_race_hud(
                &mut d,
                screen_w,
                screen_h,
                &ships,
                &track,
                &ghost_mgr,
                &net,
                &biomes[biome_idx].name,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chase_camera_initialization() {
        let cam = ChaseCamera::new();
        assert_eq!(cam.fov, DEFAULT_CAM_FOV);
        assert!((cam.pos.y - DEFAULT_CAM_POS.y).abs() < 1e-4);
    }

    #[test]
    fn test_chase_camera_boost_fov() {
        let mut player = Ship::new(
            0,
            "TEST",
            [255, 255, 255],
            [0, 200, 255],
            0.0,
            0.0,
            false,
        );
        let cfg = PhysicsConfig::load();
        player.boost_timer = 0.5;
        let boost_fov = ChaseCamera::compute_target_fov(&player, &cfg);
        assert_eq!(boost_fov, cfg.camera_boost_fov);

        player.boost_timer = 0.0;
        let normal_fov = ChaseCamera::compute_target_fov(&player, &cfg);
        assert_eq!(normal_fov, cfg.camera_base_fov);
    }
}
