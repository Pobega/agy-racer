use crate::math::Vec3;
use crate::physics::Ship;
use crate::render::draw_cel_ship;
use crate::track::BakedTrack;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const GHOST_SAMPLE_INTERVAL_SEC: f32 = 0.10;
pub const MIN_VALID_LAP_SAMPLES: usize = 10;
pub const INITIAL_GHOST_CAPACITY: usize = 600;
pub const MIN_PLAYBACK_LAP_DURATION_SEC: f32 = 1.0;
pub const SAMPLE_TIME_SPAN_EPSILON: f32 = 1e-4;
pub const GHOST_SHIP_ID: u8 = 99;
pub const GHOST_MARKER_RINGS: i32 = 4;
pub const GHOST_HOLOGRAM_ALPHA: u8 = 130;
pub const GHOST_MARKER_HEIGHT: f32 = 1.8;
pub const GHOST_MARKER_RADIUS: f32 = 0.35;
pub const GHOST_PRIMARY_COLOR: [u8; 3] = [60, 240, 255];
pub const GHOST_ACCENT_COLOR: [u8; 3] = [255, 255, 255];

/// Periodic keyframe recording ship state for ghost replay.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GhostKeyframe {
    pub time_sec: f32,
    pub segment_id: u8,
    pub s: f32,
    pub r: f32,
    pub h: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

/// Interpolated ghost sample at a specific lap time.
#[derive(Debug, Clone, Copy)]
pub struct GhostSample {
    pub segment_id: usize,
    pub s: f32,
    pub r: f32,
    pub h: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostReplay {
    pub track_name: String,
    pub lap_time_sec: f32,
    pub frames: Vec<GhostKeyframe>,
}

pub struct GhostManager {
    pub current_recording: Vec<GhostKeyframe>,
    pub best_ghost: Option<GhostReplay>,
    pub sample_accum: f32,
    pub last_known_lap: i32,
}

impl GhostManager {
    pub fn new(track_name: &str) -> Self {
        let mut mgr = Self {
            current_recording: Vec::with_capacity(INITIAL_GHOST_CAPACITY),
            best_ghost: None,
            sample_accum: 0.0,
            last_known_lap: 1,
        };
        mgr.load_best_ghost(track_name);
        mgr
    }

    pub fn ghost_file_path(track_name: &str) -> String {
        let slug: String = track_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
            .collect();
        format!("assets/tracks/{}_ghost.json", slug)
    }

    pub fn load_best_ghost(&mut self, track_name: &str) {
        let path = Self::ghost_file_path(track_name);
        if Path::new(&path).exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(replay) = serde_json::from_str::<GhostReplay>(&data) {
                    self.best_ghost = Some(replay);
                }
            }
        }
    }

    pub fn save_best_ghost(&self) {
        if let Some(ref ghost) = self.best_ghost {
            let _ = fs::create_dir_all("assets/tracks");
            let path = Self::ghost_file_path(&ghost.track_name);
            if let Ok(json) = serde_json::to_string(ghost) {
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn should_promote_completed_lap(
        &self,
        completed_lap_sec: Option<f32>,
        sample_count: usize,
    ) -> Option<f32> {
        let lap_sec = completed_lap_sec?;
        if sample_count <= MIN_VALID_LAP_SAMPLES {
            return None;
        }
        let beats_existing = self
            .best_ghost
            .as_ref()
            .map_or(true, |existing| lap_sec < existing.lap_time_sec);
        if beats_existing {
            Some(lap_sec)
        } else {
            None
        }
    }

    /// Record keyframes at fixed intervals and promote to best_ghost on a new personal best lap.
    pub fn update_recording(&mut self, player: &Ship, track_name: &str, dt: f32) {
        if player.current_lap > self.last_known_lap {
            self.last_known_lap = player.current_lap;
            let candidate_lap = player.last_completed_lap_sec.or_else(|| {
                if self.best_ghost.is_none() {
                    player.best_lap_sec
                } else {
                    None
                }
            });

            if let Some(new_pb_sec) =
                self.should_promote_completed_lap(candidate_lap, self.current_recording.len())
            {
                self.best_ghost = Some(GhostReplay {
                    track_name: track_name.to_string(),
                    lap_time_sec: new_pb_sec,
                    frames: self.current_recording.clone(),
                });
                self.save_best_ghost();
            }
            self.current_recording.clear();
            self.sample_accum = 0.0;
        }

        self.sample_accum += dt;
        if self.sample_accum >= GHOST_SAMPLE_INTERVAL_SEC || self.current_recording.is_empty() {
            self.sample_accum = 0.0;
            self.current_recording.push(GhostKeyframe {
                time_sec: player.lap_time_sec,
                segment_id: player.segment_id as u8,
                s: player.s,
                r: player.r,
                h: player.h,
                yaw: player.yaw_offset,
                pitch: player.pitch_offset,
                roll: player.visual_roll,
            });
        }
    }

    /// Compute interpolated ghost coordinates along the track at the specified lap timestamp.
    pub fn interpolate_sample_at_time(
        &self,
        track: &BakedTrack,
        lap_time: f32,
    ) -> Option<GhostSample> {
        let ghost = self.best_ghost.as_ref()?;
        if ghost.frames.len() < 2 {
            return None;
        }

        let t = lap_time.rem_euclid(ghost.lap_time_sec.max(MIN_PLAYBACK_LAP_DURATION_SEC));
        let idx = match ghost
            .frames
            .binary_search_by(|f| f.time_sec.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let next_idx = (idx + 1).min(ghost.frames.len() - 1);
        let a = &ghost.frames[idx];
        let b = &ghost.frames[next_idx];

        let span = (b.time_sec - a.time_sec).max(SAMPLE_TIME_SPAN_EPSILON);
        let frac = ((t - a.time_sec) / span).clamp(0.0, 1.0);

        let seg_id = a.segment_id as usize;
        let seg = track.segment_or_main(seg_id);

        let mut ds = b.s - a.s;
        if a.segment_id == b.segment_id && ds < -seg.total_length * 0.5 {
            ds += seg.total_length;
        }
        let s = if a.segment_id == b.segment_id {
            (a.s + ds * frac).rem_euclid(seg.total_length)
        } else {
            a.s
        };
        let r = a.r + (b.r - a.r) * frac;
        let h = a.h + (b.h - a.h) * frac;
        let yaw = a.yaw + (b.yaw - a.yaw) * frac;
        let pitch = a.pitch + (b.pitch - a.pitch) * frac;
        let roll = a.roll + (b.roll - a.roll) * frac;

        Some(GhostSample {
            segment_id: seg_id,
            s,
            r,
            h,
            yaw,
            pitch,
            roll,
        })
    }

    /// Interpolate and draw the translucent cyan ghost ship at the player's current lap timestamp.
    pub fn draw_ghost(
        &self,
        d: &mut RaylibMode3D<RaylibDrawHandle>,
        track: &BakedTrack,
        lap_time_sec: f32,
    ) {
        let Some(sample) = self.interpolate_sample_at_time(track, lap_time_sec) else {
            return;
        };

        let mut ghost_ship = Ship::new(
            GHOST_SHIP_ID,
            "PB GHOST",
            GHOST_PRIMARY_COLOR,
            GHOST_ACCENT_COLOR,
            sample.s,
            sample.r,
            true,
        );
        ghost_ship.segment_id = sample.segment_id;
        ghost_ship.h = sample.h;
        ghost_ship.yaw_offset = sample.yaw;
        ghost_ship.pitch_offset = sample.pitch;
        ghost_ship.visual_roll = sample.roll;
        ghost_ship.update_world_transform(track);

        draw_cel_ship(d, &ghost_ship, GHOST_HOLOGRAM_ALPHA);
        let label_pos = (ghost_ship.world_pos + Vec3::UP * GHOST_MARKER_HEIGHT).to_rl();
        d.draw_sphere_wires(
            label_pos,
            GHOST_MARKER_RADIUS,
            GHOST_MARKER_RINGS,
            GHOST_MARKER_RINGS,
            Color::new(80, 255, 255, 140),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slower_lap_does_not_overwrite_faster_best_ghost() {
        let mgr = GhostManager {
            current_recording: Vec::new(),
            best_ghost: Some(GhostReplay {
                track_name: "test_circuit".to_string(),
                lap_time_sec: 30.0,
                frames: Vec::new(),
            }),
            sample_accum: 0.0,
            last_known_lap: 1,
        };

        assert_eq!(mgr.should_promote_completed_lap(Some(45.0), 20), None);
        assert_eq!(mgr.best_ghost.as_ref().unwrap().lap_time_sec, 30.0);

        assert_eq!(mgr.should_promote_completed_lap(Some(28.5), 20), Some(28.5));

        assert_eq!(mgr.should_promote_completed_lap(Some(30.0), 20), None);

        assert_eq!(
            mgr.should_promote_completed_lap(Some(25.0), MIN_VALID_LAP_SAMPLES),
            None
        );
        assert_eq!(mgr.should_promote_completed_lap(Some(25.0), 0), None);

        assert_eq!(mgr.should_promote_completed_lap(None, 20), None);

        let empty_mgr = GhostManager {
            current_recording: Vec::new(),
            best_ghost: None,
            sample_accum: 0.0,
            last_known_lap: 1,
        };
        assert_eq!(
            empty_mgr.should_promote_completed_lap(Some(45.0), 20),
            Some(45.0)
        );
    }
}
