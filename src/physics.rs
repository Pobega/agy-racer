use crate::config::PhysicsConfig;
use crate::math::{smooth_damp_f32, Vec3};
use crate::track::{BakedTrack, SurfaceFrame, SurfaceType};

const AI_LOOKAHEAD_DISTANCE: f32 = 35.0;
const AI_WEAVE_TIME_FREQ: f32 = 0.9;
const AI_WEAVE_ID_PHASE: f32 = 2.1;
const AI_WEAVE_WIDTH_FACTOR: f32 = 0.25;
const AI_STEER_GAIN: f32 = 0.14;
const AI_SHARP_TURN_DOT: f32 = 0.18;
const AI_AIRBRAKE_MAX: f32 = 0.8;
const AI_GAP_PITCH: f32 = 0.75;
const AI_BOOST_MIN_SHIELD: f32 = 55.0;
const AI_BOOST_STRAIGHT_DOT: f32 = 0.08;
const AI_THROTTLE: f32 = 0.90;

const BOOST_ACTIVATION_COOLDOWN_THRESHOLD: f32 = 0.15;
const BOOST_INSTANT_SPEED_KICK: f32 = 26.0;
const BOOST_MIN_SHIELD_FLOOR: f32 = 1.0;
const BOOST_PAD_MIN_TIMER: f32 = 0.65;
const BOOST_PAD_MAX_SPEED_BONUS: f32 = 6.0;
pub const BOOST_SAFETY_SHIELD_MARGIN: f32 = 1.0;

pub const DEFAULT_HOVER_HEIGHT: f32 = 1.25;
pub const DEFAULT_RACE_LAPS: u32 = 3;
pub const MAX_SHIELD_ENERGY: f32 = 100.0;
pub const THROTTLE_DEADZONE: f32 = 0.01;
pub const AIRBRAKE_DEADZONE: f32 = 0.1;

const GROUNDED_MAX_HEIGHT: f32 = 3.2;
const EDGE_RUNOFF_MARGIN: f32 = 0.8;
const MIN_HOVER_HEIGHT: f32 = 0.40;
const TAKEOFF_PITCH_LIFT_FACTOR: f32 = 18.0;
const AIRBORNE_STEER_ACCEL: f32 = 18.0;
const AIRBORNE_PITCH_TARGET_FACTOR: f32 = 0.48;
const AIRBORNE_PITCH_RATE: f32 = 6.0;
const GLIDE_NOSE_UP_THRESHOLD: f32 = 0.05;
const GROUNDED_PITCH_TARGET_FACTOR: f32 = 0.28;
const GROUNDED_PITCH_RATE: f32 = 8.0;
const PITCH_HOVER_FORCE: f32 = 10.0;
const OVERSPEED_DECEL_RATE: f32 = 4.0;
const SHIELD_LIMP_THRESHOLD: f32 = 1.5;

pub const MIN_VALID_LAP_TIME_SEC: f32 = 5.0;

const STEER_YAW_FACTOR: f32 = 0.22;
const AIRBRAKE_YAW_FACTOR: f32 = 0.42;
const MAX_YAW_OFFSET_RAD: f32 = 0.60;
const YAW_DAMP_RATE: f32 = 14.0;

const STEER_VISUAL_ROLL_FACTOR: f32 = 0.42;
const AIRBRAKE_VISUAL_ROLL_FACTOR: f32 = 0.48;
const MAX_VISUAL_ROLL_RAD: f32 = 0.78;
const ROLL_DAMP_RATE: f32 = 12.0;
const FLAP_DAMP_RATE: f32 = 16.0;

const MIN_CENTRIFUGAL_SPEED_RATIO: f32 = 0.25;
const MAX_CENTRIFUGAL_SPEED_RATIO: f32 = 1.35;
const CENTRIFUGAL_LOOKAHEAD_METERS: f32 = 6.0;
const AIRBRAKE_GRIP_LOSS_FACTOR: f32 = 0.45;
const MIN_AIRBRAKE_GRIP_MULT: f32 = 0.4;

const BRANCH_CAPTURE_WIDTH_RATIO: f32 = 0.65;
const BRANCH_R_CLAMP: f32 = 6.0;

const TRAIL_MAX_POINTS: usize = 22;
const TRAIL_OFFSET_FORWARD: f32 = 1.55;
const TRAIL_OFFSET_RIGHT: f32 = 1.95;
const TRAIL_OFFSET_UP: f32 = 0.05;

const RESPAWN_HEIGHT: f32 = 1.6;
const RESPAWN_SPEED: f32 = 35.0;
const RESPAWN_DAMAGE: f32 = 20.0;
const RESPAWN_MIN_SHIELD: f32 = 10.0;

const WALL_MIN_BOUNCE_SPEED: f32 = 6.0;
const WALL_MARGIN: f32 = 0.9;
const MAX_LATERAL_GRIP_RESPONSE: f32 = 18.0;

const SHIP_COLLISION_HEIGHT: f32 = 2.0;
const COLLISION_RAM_BONUS_TIMER: f32 = 0.2;
const COLLISION_RAM_BONUS_IMPULSE: f32 = 18.0;
const COLLISION_BASE_IMPULSE: f32 = 10.0;
const COLLISION_REL_VR_FACTOR: f32 = 0.6;
const COLLISION_MAX_IMPULSE: f32 = 35.0;
const COLLISION_BASE_SHIELD_DAMAGE: f32 = 2.5;
const COLLISION_RAM_SHIELD_DAMAGE_FACTOR: f32 = 0.4;

/// Input commands for a single simulation tick.
#[derive(Debug, Clone, Copy, Default)]
pub struct ShipInput {
    pub throttle: f32,
    pub steer: f32,
    pub airbrake_left: f32,
    pub airbrake_right: f32,
    pub pitch: f32,
    pub boost_pressed: bool,
    pub side_attack_dir: i8,
}

/// Complete physical state of an antigravity craft.
#[derive(Debug, Clone)]
pub struct Ship {
    pub id: u8,
    pub name: String,
    pub primary_color: [u8; 3],
    pub accent_color: [u8; 3],
    pub is_ai: bool,
    pub ai_target_r: f32,

    pub segment_id: usize,
    pub s: f32,
    pub r: f32,
    pub h: f32,

    pub v_s: f32,
    pub v_r: f32,
    pub v_h: f32,
    pub is_airborne: bool,

    pub yaw_offset: f32,
    pub pitch_offset: f32,
    pub visual_roll: f32,
    pub flap_left: f32,
    pub flap_right: f32,
    pub trail_history: Vec<(Vec3, Vec3)>,

    pub shield_energy: f32,
    pub boost_timer: f32,
    pub side_attack_timer: f32,
    pub last_safe_pos: (usize, f32),

    pub world_pos: Vec3,
    pub world_forward: Vec3,
    pub world_up: Vec3,
    pub world_right: Vec3,

    pub current_lap: i32,
    pub total_laps: i32,
    pub lap_time_sec: f32,
    pub best_lap_sec: Option<f32>,
    pub last_completed_lap_sec: Option<f32>,
    pub passed_halfway: bool,
}

impl Ship {
    pub fn new(
        id: u8,
        name: &str,
        primary_color: [u8; 3],
        accent_color: [u8; 3],
        start_s: f32,
        start_r: f32,
        is_ai: bool,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            primary_color,
            accent_color,
            is_ai,
            ai_target_r: start_r,
            segment_id: 0,
            s: start_s,
            r: start_r,
            h: DEFAULT_HOVER_HEIGHT,
            v_s: 0.0,
            v_r: 0.0,
            v_h: 0.0,
            is_airborne: false,
            yaw_offset: 0.0,
            pitch_offset: 0.0,
            visual_roll: 0.0,
            flap_left: 0.0,
            flap_right: 0.0,
            trail_history: Vec::with_capacity(TRAIL_MAX_POINTS),
            shield_energy: MAX_SHIELD_ENERGY,
            boost_timer: 0.0,
            side_attack_timer: 0.0,
            last_safe_pos: (0, start_s.max(0.0)),
            world_pos: Vec3::new(start_r, DEFAULT_HOVER_HEIGHT, start_s),
            world_forward: Vec3::FORWARD,
            world_up: Vec3::UP,
            world_right: Vec3::RIGHT,
            current_lap: 1,
            total_laps: DEFAULT_RACE_LAPS as i32,
            lap_time_sec: 0.0,
            best_lap_sec: None,
            last_completed_lap_sec: None,
            passed_halfway: false,
        }
    }

    #[inline]
    pub fn local_point(&self, forward: f32, right: f32, up: f32) -> raylib::prelude::Vector3 {
        (self.world_pos + self.world_forward * forward + self.world_right * right + self.world_up * up).to_rl()
    }

    #[inline]
    pub fn airbrake_flap_lift(flap_open: f32) -> f32 {
        const BASE_FLAP_HEIGHT: f32 = 0.12;
        const OPEN_FLAP_LIFT: f32 = 0.55;
        BASE_FLAP_HEIGHT + flap_open * OPEN_FLAP_LIFT
    }

    #[inline]
    pub fn thruster_flame_length(&self, is_boosting: bool) -> f32 {
        const BOOST_FLAME_LENGTH: f32 = 4.2;
        const BASE_FLAME_LENGTH: f32 = 1.1;
        const SPEED_FLAME_SCALE: f32 = 1.9 / 90.0;
        if is_boosting {
            BOOST_FLAME_LENGTH
        } else {
            BASE_FLAME_LENGTH + self.v_s * SPEED_FLAME_SCALE
        }
    }

    pub fn race_progress(&self, track: &BakedTrack) -> f32 {
        let seg_base = track.segment_or_main(self.segment_id).base_race_distance;
        (self.current_lap as f32 - 1.0) * track.primary_lap_length + seg_base + self.s
    }

    pub fn compute_ai_input(&mut self, track: &BakedTrack, time_sec: f32) -> ShipInput {
        let seg = track.segment_or_main(self.segment_id);
        let sample_now = seg.sample_at(self.s);
        let sample_ahead = seg.sample_at(self.s + AI_LOOKAHEAD_DISTANCE);

        self.ai_target_r = compute_ai_lane_weave_offset(self.id, time_sec, sample_now.width);

        let steer = ((self.ai_target_r - self.r) * AI_STEER_GAIN).clamp(-1.0, 1.0);
        let turn_dot = sample_now.forward.dot(sample_ahead.right);
        let (airbrake_left, airbrake_right) = compute_ai_cornering_airbrakes(turn_dot);
        let pitch = if !sample_now.has_floor || !sample_ahead.has_floor {
            AI_GAP_PITCH
        } else {
            0.0
        };
        let boost_pressed = should_ai_activate_boost(
            self.shield_energy,
            self.boost_timer,
            sample_now.surface,
            turn_dot,
        );

        ShipInput {
            throttle: AI_THROTTLE,
            steer,
            airbrake_left,
            airbrake_right,
            pitch,
            boost_pressed,
            side_attack_dir: 0,
        }
    }

    #[inline]
    pub fn is_over_open_air(&self, frame: &SurfaceFrame) -> bool {
        let half_w = frame.width * 0.5;
        let off_lateral_edge = !frame.is_cylinder && self.r.abs() > half_w + EDGE_RUNOFF_MARGIN;
        !frame.has_floor || off_lateral_edge
    }

    #[inline]
    pub fn can_activate_manual_boost(&self, cfg: &PhysicsConfig) -> bool {
        self.boost_timer <= BOOST_ACTIVATION_COOLDOWN_THRESHOLD
            && self.shield_energy > cfg.boost_energy_cost + BOOST_SAFETY_SHIELD_MARGIN
    }

    #[inline]
    pub fn is_in_shield_limp_mode(&self) -> bool {
        self.shield_energy <= SHIELD_LIMP_THRESHOLD
    }

    pub fn apply_boost_and_side_attack(&mut self, input: &ShipInput, cfg: &PhysicsConfig, _dt: f32) {
        if input.boost_pressed && self.can_activate_manual_boost(cfg) {
            self.shield_energy = (self.shield_energy - cfg.boost_energy_cost).max(BOOST_MIN_SHIELD_FLOOR);
            self.boost_timer = cfg.boost_duration_sec;
            self.v_s = (self.v_s + BOOST_INSTANT_SPEED_KICK).min(cfg.boost_speed);
        }

        if input.side_attack_dir != 0 && self.side_attack_timer <= 0.0 {
            self.side_attack_timer = cfg.side_attack_duration;
            self.v_r += (input.side_attack_dir as f32) * cfg.side_attack_impulse;
        }
    }

    pub fn apply_surface_zone_effects(
        &mut self,
        surface: SurfaceType,
        cfg: &PhysicsConfig,
        dt: f32,
    ) -> f32 {
        let mut grip_mult = 1.0f32;
        match surface {
            SurfaceType::Normal => {}
            SurfaceType::BoostPad => {
                self.boost_timer = self.boost_timer.max(BOOST_PAD_MIN_TIMER);
                self.v_s = (self.v_s + cfg.boost_pad_impulse * dt).min(cfg.boost_speed + BOOST_PAD_MAX_SPEED_BONUS);
            }
            SurfaceType::RechargeStrip => {
                self.shield_energy =
                    (self.shield_energy + cfg.recharge_strip_rate * dt).min(MAX_SHIELD_ENERGY);
            }
            SurfaceType::LowGripIce => {
                grip_mult = cfg.ice_grip_multiplier;
            }
        }
        grip_mult
    }

    #[inline]
    pub fn compute_spline_turn_rate(sample_now: & crate::track::BakedSample, sample_ahead: & crate::track::BakedSample) -> f32 {
        sample_now.right.dot(sample_ahead.forward - sample_now.forward) / CENTRIFUGAL_LOOKAHEAD_METERS
    }

    #[inline]
    pub fn compute_centrifugal_slide(&self, spline_turn_rate: f32, cfg: &PhysicsConfig) -> f32 {
        -spline_turn_rate * self.v_s * self.v_s * cfg.centrifugal_factor
    }

    #[inline]
    pub fn compute_hover_spring_acceleration(&self, pitch_input: f32, cfg: &PhysicsConfig) -> f32 {
        let h_err = cfg.hover_target_height - self.h;
        h_err * cfg.hover_spring_stiffness
            - self.v_h * cfg.hover_spring_damping
            + pitch_input * PITCH_HOVER_FORCE
    }

    pub fn step_grounded_flight(
        &mut self,
        input: &ShipInput,
        spline_turn_rate: f32,
        friction_mult: f32,
        cfg: &PhysicsConfig,
        dt: f32,
    ) {
        self.is_airborne = false;
        self.last_safe_pos = (self.segment_id, self.s);

        let ab_diff = input.airbrake_right - input.airbrake_left;
        let ab_both = input.airbrake_left.min(input.airbrake_right);

        let target_yaw = (input.steer * STEER_YAW_FACTOR + ab_diff * AIRBRAKE_YAW_FACTOR)
            .clamp(-MAX_YAW_OFFSET_RAD, MAX_YAW_OFFSET_RAD);
        self.yaw_offset = smooth_damp_f32(self.yaw_offset, target_yaw, YAW_DAMP_RATE, dt);

        let base_max_speed = if self.is_in_shield_limp_mode() {
            cfg.limp_speed
        } else {
            cfg.max_speed
        };
        let max_speed = if self.boost_timer > 0.0 {
            cfg.boost_speed
        } else {
            base_max_speed
        };

        let pitch_target = input.pitch * GROUNDED_PITCH_TARGET_FACTOR;
        self.pitch_offset = smooth_damp_f32(self.pitch_offset, pitch_target, GROUNDED_PITCH_RATE, dt);

        let accel = if self.boost_timer > 0.0 {
            cfg.boost_acceleration
        } else {
            cfg.acceleration
        };
        if input.throttle > THROTTLE_DEADZONE {
            self.v_s += input.throttle * accel * dt;
        } else {
            self.v_s -= cfg.coast_deceleration * dt;
        }

        if ab_both > AIRBRAKE_DEADZONE {
            self.v_s -= ab_both * cfg.dual_airbrake_decel * dt;
        }
        let single_ab = (input.airbrake_left - input.airbrake_right).abs();
        self.v_s -= single_ab * cfg.single_airbrake_drag * dt;

        if self.v_s > max_speed {
            self.v_s -= (self.v_s - max_speed) * OVERSPEED_DECEL_RATE * dt;
        }
        self.v_s = self.v_s.max(0.0);

        let speed_ratio = (self.v_s / cfg.max_speed.max(1.0))
            .clamp(MIN_CENTRIFUGAL_SPEED_RATIO, MAX_CENTRIFUGAL_SPEED_RATIO);
        let desired_lat_speed = (input.steer * cfg.steer_lateral_speed
            + ab_diff * cfg.airbrake_lateral_speed)
            * speed_ratio;
        let centrifugal_push = self.compute_centrifugal_slide(spline_turn_rate, cfg);

        let grip_response = (cfg.lateral_grip
            * (1.0 - single_ab * AIRBRAKE_GRIP_LOSS_FACTOR).max(MIN_AIRBRAKE_GRIP_MULT)
            * friction_mult)
            .min(MAX_LATERAL_GRIP_RESPONSE);
        self.v_r += ((desired_lat_speed - self.v_r) * grip_response + centrifugal_push) * dt;

        let spring_force = self.compute_hover_spring_acceleration(input.pitch, cfg);
        self.v_h += spring_force * dt;
        self.h = (self.h + self.v_h * dt).max(MIN_HOVER_HEIGHT);
    }

    pub fn step_airborne_flight(
        &mut self,
        input: &ShipInput,
        track: &BakedTrack,
        frame: &SurfaceFrame,
        cfg: &PhysicsConfig,
        dt: f32,
    ) {
        if !self.is_airborne {
            self.is_airborne = true;
            self.v_h += (self.pitch_offset.max(0.0) * TAKEOFF_PITCH_LIFT_FACTOR) + cfg.takeoff_vertical_boost;
        }

        let pitch_target = input.pitch * AIRBORNE_PITCH_TARGET_FACTOR;
        self.pitch_offset = smooth_damp_f32(self.pitch_offset, pitch_target, AIRBORNE_PITCH_RATE, dt);

        let glide_gravity = if self.pitch_offset > GLIDE_NOSE_UP_THRESHOLD {
            cfg.glide_gravity_nose_up
        } else {
            cfg.glide_gravity_normal
        };
        self.v_h -= glide_gravity * dt;
        self.h += self.v_h * dt;

        self.v_r += input.steer * AIRBORNE_STEER_ACCEL * dt;
        self.advance_spline_coords(track, frame, cfg, dt);

        if self.h < cfg.void_respawn_height {
            self.respawn_on_track();
        } else if frame.has_floor
            && !self.is_over_open_air(frame)
            && self.h <= cfg.hover_target_height
            && self.v_h <= 0.0
        {
            self.is_airborne = false;
            self.h = cfg.hover_target_height;
            self.v_h = 0.0;
        }
    }

    pub fn update_visual_tilt_and_trails(&mut self, input: &ShipInput, dt: f32) {
        let target_roll = (-input.steer * STEER_VISUAL_ROLL_FACTOR
            - (input.airbrake_right - input.airbrake_left) * AIRBRAKE_VISUAL_ROLL_FACTOR)
            .clamp(-MAX_VISUAL_ROLL_RAD, MAX_VISUAL_ROLL_RAD);
        self.visual_roll = smooth_damp_f32(self.visual_roll, target_roll, ROLL_DAMP_RATE, dt);

        self.flap_left = smooth_damp_f32(self.flap_left, input.airbrake_left, FLAP_DAMP_RATE, dt);
        self.flap_right = smooth_damp_f32(self.flap_right, input.airbrake_right, FLAP_DAMP_RATE, dt);

        let wl = self.world_pos - self.world_forward * TRAIL_OFFSET_FORWARD - self.world_right * TRAIL_OFFSET_RIGHT
            + self.world_up * TRAIL_OFFSET_UP;
        let wr = self.world_pos - self.world_forward * TRAIL_OFFSET_FORWARD + self.world_right * TRAIL_OFFSET_RIGHT
            + self.world_up * TRAIL_OFFSET_UP;
        self.trail_history.push((wl, wr));
        if self.trail_history.len() > TRAIL_MAX_POINTS {
            self.trail_history.remove(0);
        }
    }

    /// Step ship physics using tunable parameters from `assets/config/physics.json`.
    pub fn step_physics(
        &mut self,
        input: ShipInput,
        track: &BakedTrack,
        cfg: &PhysicsConfig,
        dt: f32,
    ) {
        self.last_completed_lap_sec = None;
        self.lap_time_sec += dt;
        self.boost_timer = (self.boost_timer - dt).max(0.0);
        self.side_attack_timer = (self.side_attack_timer - dt).max(0.0);

        if track.segments.is_empty() {
            return;
        }

        let seg = track.segment_or_main(self.segment_id);
        let sample = seg.sample_at(self.s);
        let sample_ahead = seg.sample_at(self.s + CENTRIFUGAL_LOOKAHEAD_METERS);
        let frame = sample.evaluate_cross_section(self.r);
        let spline_turn_rate = Self::compute_spline_turn_rate(&sample, &sample_ahead);

        self.apply_boost_and_side_attack(&input, cfg, dt);

        if !self.is_over_open_air(&frame) && self.h <= GROUNDED_MAX_HEIGHT {
            let friction_mult = self.apply_surface_zone_effects(frame.surface, cfg, dt);
            self.step_grounded_flight(&input, spline_turn_rate, friction_mult, cfg, dt);
            self.advance_spline_coords(track, &frame, cfg, dt);
        } else {
            self.step_airborne_flight(&input, track, &frame, cfg, dt);
        }

        self.update_visual_tilt_and_trails(&input, dt);
        self.update_world_transform(track);
    }

    pub fn has_crossed_halfway(prev_s: f32, new_s: f32, total_len: f32) -> bool {
        let halfway = total_len * 0.5;
        prev_s < halfway && new_s >= halfway
    }

    pub fn try_switch_to_branch_segment(&mut self, prev_s: f32, track: &BakedTrack) {
        let seg = track.segment_or_main(self.segment_id);
        let cur_cp = seg.sample_at(self.s).cp_index;
        let prev_cp = seg.sample_at(prev_s).cp_index;
        if cur_cp == prev_cp {
            return;
        }

        for seg_def in &track.def.segments {
            if seg_def.id != self.segment_id
                && seg_def.parent_anchor == Some((self.segment_id, cur_cp))
            {
                if let Some(first_pt) = seg_def.points.first() {
                    let branch_offset = (first_pt.pos - seg.sample_at(self.s).pos)
                        .dot(seg.sample_at(self.s).right);
                    if (self.r - branch_offset).abs() < first_pt.width * BRANCH_CAPTURE_WIDTH_RATIO {
                        self.segment_id = seg_def.id;
                        self.s = 0.0;
                        self.r = (self.r - branch_offset).clamp(-BRANCH_R_CLAMP, BRANCH_R_CLAMP);
                        break;
                    }
                }
            }
        }
    }

    pub fn advance_or_wrap_segment_end(&mut self, track: &BakedTrack) {
        let seg = track.segment_or_main(self.segment_id);
        if self.s < seg.total_length {
            return;
        }

        if seg.closed_loop {
            self.s = self.s.rem_euclid(seg.total_length);
            if self.segment_id == 0 && self.passed_halfway && self.lap_time_sec > MIN_VALID_LAP_TIME_SEC {
                self.passed_halfway = false;
                self.last_completed_lap_sec = Some(self.lap_time_sec);
                if self.best_lap_sec.map_or(true, |best| self.lap_time_sec < best) {
                    self.best_lap_sec = Some(self.lap_time_sec);
                }
                self.lap_time_sec = 0.0;
                self.current_lap += 1;
            }
        } else if let Some(&next_seg_id) = seg.next_segments.first() {
            let (_, new_s, new_r, _) = track.project_world_pos(self.world_pos, next_seg_id);
            self.segment_id = next_seg_id;
            self.s = new_s;
            self.r = new_r;
        }
    }

    pub fn resolve_lateral_wall_bounds(&mut self, frame: &SurfaceFrame, cfg: &PhysicsConfig) {
        let half_w = frame.width * 0.5 - WALL_MARGIN;
        if frame.is_cylinder {
            if self.r > frame.width * 0.5 {
                self.r -= frame.width;
            } else if self.r < -frame.width * 0.5 {
                self.r += frame.width;
            }
        } else if !self.is_airborne {
            if self.r < -half_w && frame.walls.has_left() {
                let impact = (-self.v_r).max(0.0);
                self.r = -half_w;
                self.v_r = (impact * cfg.wall_restitution).max(WALL_MIN_BOUNCE_SPEED);
                self.v_s *= cfg.wall_speed_retention;
                self.shield_energy = (self.shield_energy - impact * cfg.wall_damage_factor).max(0.0);
            } else if self.r > half_w && frame.walls.has_right() {
                let impact = self.v_r.max(0.0);
                self.r = half_w;
                self.v_r = -(impact * cfg.wall_restitution).max(WALL_MIN_BOUNCE_SPEED);
                self.v_s *= cfg.wall_speed_retention;
                self.shield_energy = (self.shield_energy - impact * cfg.wall_damage_factor).max(0.0);
            }
        }
    }

    fn advance_spline_coords(
        &mut self,
        track: &BakedTrack,
        frame: &SurfaceFrame,
        cfg: &PhysicsConfig,
        dt: f32,
    ) {
        let seg = track.segment_or_main(self.segment_id);
        let prev_s = self.s;
        let total_len = seg.total_length;

        self.s += self.v_s * dt;
        self.r += self.v_r * dt;

        if self.segment_id == 0 && Self::has_crossed_halfway(prev_s, self.s, total_len) {
            self.passed_halfway = true;
        }

        self.try_switch_to_branch_segment(prev_s, track);
        self.advance_or_wrap_segment_end(track);
        self.resolve_lateral_wall_bounds(frame, cfg);
    }

    pub fn respawn_on_track(&mut self) {
        self.segment_id = self.last_safe_pos.0;
        self.s = self.last_safe_pos.1;
        self.r = 0.0;
        self.h = RESPAWN_HEIGHT;
        self.v_s = RESPAWN_SPEED;
        self.v_r = 0.0;
        self.v_h = 0.0;
        self.is_airborne = false;
        self.shield_energy = (self.shield_energy - RESPAWN_DAMAGE).max(RESPAWN_MIN_SHIELD);
    }

    pub fn update_world_transform(&mut self, track: &BakedTrack) {
        if track.segments.is_empty() {
            return;
        }
        let seg = track.segment_or_main(self.segment_id);
        let sample = seg.sample_at(self.s);
        let frame = sample.evaluate_cross_section(self.r);

        self.world_pos = frame.surface_pos + frame.normal * self.h;

        let yawed_fwd = frame
            .forward
            .rotate_around_axis(frame.normal, -self.yaw_offset)
            .normalized();
        let yawed_right = yawed_fwd.cross(frame.normal).normalized();

        let pitched_fwd = yawed_fwd
            .rotate_around_axis(yawed_right, self.pitch_offset)
            .normalized();
        let pitched_up = yawed_right.cross(pitched_fwd).normalized();

        let rolled_up = pitched_up
            .rotate_around_axis(pitched_fwd, self.visual_roll)
            .normalized();
        let rolled_right = pitched_fwd.cross(rolled_up).normalized();

        self.world_forward = pitched_fwd;
        self.world_up = rolled_up;
        self.world_right = rolled_right;
    }

    pub fn resolve_ship_collisions(ships: &mut [Ship], track: &BakedTrack, cfg: &PhysicsConfig) {
        let n = ships.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if ships[i].segment_id != ships[j].segment_id {
                    continue;
                }
                let seg_len = track.segment_or_main(ships[i].segment_id).total_length;

                if ships_are_overlapping(&ships[i], &ships[j], seg_len, cfg) {
                    let (left, right) = ships.split_at_mut(j);
                    let ship_a = &mut left[i];
                    let ship_b = &mut right[0];
                    apply_pair_bump_impulse(ship_a, ship_b, cfg);
                }
            }
        }
    }
}

fn compute_ai_lane_weave_offset(ship_id: u8, time_sec: f32, track_width: f32) -> f32 {
    (time_sec * AI_WEAVE_TIME_FREQ + ship_id as f32 * AI_WEAVE_ID_PHASE).sin()
        * (track_width * AI_WEAVE_WIDTH_FACTOR)
}

fn compute_ai_cornering_airbrakes(turn_dot: f32) -> (f32, f32) {
    let airbrake_left = if turn_dot < -AI_SHARP_TURN_DOT {
        (-turn_dot).clamp(0.0, AI_AIRBRAKE_MAX)
    } else {
        0.0
    };
    let airbrake_right = if turn_dot > AI_SHARP_TURN_DOT {
        turn_dot.clamp(0.0, AI_AIRBRAKE_MAX)
    } else {
        0.0
    };
    (airbrake_left, airbrake_right)
}

fn should_ai_activate_boost(
    shield_energy: f32,
    boost_timer: f32,
    surface: SurfaceType,
    turn_dot: f32,
) -> bool {
    shield_energy > AI_BOOST_MIN_SHIELD
        && boost_timer <= 0.0
        && surface == SurfaceType::Normal
        && turn_dot.abs() < AI_BOOST_STRAIGHT_DOT
}

pub fn ships_are_overlapping(a: &Ship, b: &Ship, seg_len: f32, cfg: &PhysicsConfig) -> bool {
    let mut ds = b.s - a.s;
    if ds > seg_len * 0.5 {
        ds -= seg_len;
    } else if ds < -seg_len * 0.5 {
        ds += seg_len;
    }
    ds.abs() < cfg.ship_collision_length
        && (b.r - a.r).abs() < cfg.ship_collision_width
        && (b.h - a.h).abs() < SHIP_COLLISION_HEIGHT
}

pub fn compute_pair_collision_damage(ram_bonus_a: f32, ram_bonus_b: f32) -> (f32, f32) {
    let damage_a = COLLISION_BASE_SHIELD_DAMAGE + ram_bonus_b * COLLISION_RAM_SHIELD_DAMAGE_FACTOR;
    let damage_b = COLLISION_BASE_SHIELD_DAMAGE + ram_bonus_a * COLLISION_RAM_SHIELD_DAMAGE_FACTOR;
    (damage_a, damage_b)
}

pub fn apply_pair_bump_impulse(a: &mut Ship, b: &mut Ship, cfg: &PhysicsConfig) {
    let dr = b.r - a.r;
    let push_dir = if dr >= 0.0 { 1.0 } else { -1.0 };
    let overlap_r = (cfg.ship_collision_width - dr.abs()).max(0.0);
    a.r -= push_dir * overlap_r * 0.5;
    b.r += push_dir * overlap_r * 0.5;

    let ram_bonus_a = if a.side_attack_timer > COLLISION_RAM_BONUS_TIMER {
        COLLISION_RAM_BONUS_IMPULSE
    } else {
        0.0
    };
    let ram_bonus_b = if b.side_attack_timer > COLLISION_RAM_BONUS_TIMER {
        COLLISION_RAM_BONUS_IMPULSE
    } else {
        0.0
    };

    let rel_vr = (a.v_r - b.v_r).abs();
    let impulse = (rel_vr * COLLISION_REL_VR_FACTOR + COLLISION_BASE_IMPULSE).min(COLLISION_MAX_IMPULSE);

    a.v_r = -push_dir * (impulse + ram_bonus_b);
    b.v_r = push_dir * (impulse + ram_bonus_a);

    let (damage_a, damage_b) = compute_pair_collision_damage(ram_bonus_a, ram_bonus_b);
    a.shield_energy = (a.shield_energy - damage_a).max(0.0);
    b.shield_energy = (b.shield_energy - damage_b).max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_crossed_halfway() {
        assert!(Ship::has_crossed_halfway(40.0, 60.0, 100.0));
        assert!(!Ship::has_crossed_halfway(10.0, 30.0, 100.0));
        assert!(!Ship::has_crossed_halfway(60.0, 70.0, 100.0));
    }

    #[test]
    fn test_compute_pair_collision_damage() {
        let (dmg_a, dmg_b) = compute_pair_collision_damage(COLLISION_RAM_BONUS_IMPULSE, 0.0);
        assert_eq!(dmg_a, COLLISION_BASE_SHIELD_DAMAGE);
        assert_eq!(
            dmg_b,
            COLLISION_BASE_SHIELD_DAMAGE + COLLISION_RAM_BONUS_IMPULSE * COLLISION_RAM_SHIELD_DAMAGE_FACTOR
        );
    }

    #[test]
    fn test_hover_spring_pushes_up_when_below_target_height() {
        let cfg = PhysicsConfig::default();
        let mut ship = Ship::new(0, "Test", [255, 0, 0], [0, 255, 0], 0.0, 0.0, false);
        ship.h = 0.5;
        ship.v_h = 0.0;
        let accel = ship.compute_hover_spring_acceleration(0.0, &cfg);
        assert!(accel > 0.0, "Hover spring must push upward when below target height");

        ship.h = 2.0;
        let accel_down = ship.compute_hover_spring_acceleration(0.0, &cfg);
        assert!(accel_down < 0.0, "Hover spring must push downward when above target height");
    }

    #[test]
    fn test_manual_boost_consumes_shield_and_increases_speed() {
        let cfg = PhysicsConfig::default();
        let mut ship = Ship::new(0, "Test", [255, 0, 0], [0, 255, 0], 0.0, 0.0, false);
        ship.shield_energy = 100.0;
        ship.v_s = 40.0;
        ship.boost_timer = 0.0;

        let input = ShipInput {
            boost_pressed: true,
            ..Default::default()
        };
        ship.apply_boost_and_side_attack(&input, &cfg, 0.016);

        assert!(ship.boost_timer > 0.0, "Boost timer should be activated");
        assert_eq!(ship.shield_energy, 100.0 - cfg.boost_energy_cost);
        assert!(ship.v_s > 40.0, "Speed should increase immediately on boost kick");
    }

    #[test]
    fn test_ship_collision_detection_and_separation() {
        let cfg = PhysicsConfig::default();
        let mut ship_a = Ship::new(0, "ShipA", [255, 0, 0], [0, 255, 0], 10.0, 0.0, false);
        let mut ship_b = Ship::new(1, "ShipB", [0, 255, 0], [255, 0, 0], 10.5, 0.5, false);

        assert!(ships_are_overlapping(&ship_a, &ship_b, 1000.0, &cfg));

        let prev_shield_a = ship_a.shield_energy;
        let prev_shield_b = ship_b.shield_energy;

        apply_pair_bump_impulse(&mut ship_a, &mut ship_b, &cfg);

        assert!(ship_a.r < 0.0);
        assert!(ship_b.r > 0.5);
        assert!(ship_a.v_r < 0.0);
        assert!(ship_b.v_r > 0.0);
        assert!(ship_a.shield_energy < prev_shield_a);
        assert!(ship_b.shield_energy < prev_shield_b);

        ship_b.s = 100.0;
        assert!(!ships_are_overlapping(&ship_a, &ship_b, 1000.0, &cfg));
    }

    #[test]
    fn test_straight_halfpipe_does_not_pull_ship_left() {
        let cfg = PhysicsConfig::default();
        let mut def = BakedTrack::default_showcase_track();
        def.segments[0].closed_loop = false;
        def.segments[0].points = vec![
            crate::track::ControlPoint {
                pos: Vec3::new(0.0, 0.0, 0.0),
                roll_deg: 0.0,
                width: 30.0,
                curvature: 0.5,
                walls: crate::track::WallMode::Both,
                has_floor: true,
                surface: SurfaceType::Normal,
            },
            crate::track::ControlPoint {
                pos: Vec3::new(0.0, 0.0, 200.0),
                roll_deg: 0.0,
                width: 30.0,
                curvature: 0.5,
                walls: crate::track::WallMode::Both,
                has_floor: true,
                surface: SurfaceType::Normal,
            },
        ];
        let track = BakedTrack::from_def(def);
        let mut ship = Ship::new(0, "Player", [255, 255, 255], [0, 255, 255], 10.0, 0.0, false);
        let input = ShipInput {
            throttle: 1.0,
            ..Default::default()
        };
        for _ in 0..60 {
            ship.step_physics(input, &track, &cfg, 1.0 / 60.0);
        }
        assert!(ship.r.abs() < 1e-3, "Straight half-pipe must not apply lateral centrifugal drift, got r={}", ship.r);
    }

    #[test]
    fn test_showcase_track_start_straight_does_not_drift() {
        let cfg = PhysicsConfig::default();
        let def = BakedTrack::default_showcase_track();
        let track = BakedTrack::from_def(def);
        let mut ship = Ship::new(0, "Player", [235, 245, 255], [0, 235, 255], 12.0, -3.5, false);
        ship.update_world_transform(&track);

        let input = ShipInput {
            throttle: 1.0,
            ..Default::default()
        };
        for _ in 0..90 {
            ship.step_physics(input, &track, &cfg, 1.0 / 60.0);
        }
        assert!(
            !ship.is_airborne,
            "Ship should remain grounded on start straight"
        );
        assert!(
            ship.r > -13.0 && ship.r < 13.0,
            "Ship should stay well inside track walls on start straight, got r={}",
            ship.r
        );
    }
}
