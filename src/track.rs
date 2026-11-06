use crate::config::BiomePalette;
use crate::math::{catmull_rom_pos, catmull_rom_tangent, smooth_lerp, Vec3};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

pub const ACTIVE_TRACK_PATH: &str = "assets/tracks/circuit_02.json";

pub const CYLINDER_CURVATURE_THRESHOLD: f32 = 0.92;
pub const FLAT_CURVATURE_EPSILON: f32 = 1e-3;
pub const DEFAULT_TRACK_WIDTH: f32 = 24.0;
pub const DEFAULT_LAP_LENGTH: f32 = 1000.0;
pub const SAMPLES_PER_SPLINE_SPAN: usize = 40;
pub const PARALLEL_UP_SINGULARITY_DOT: f32 = 0.95;

/// Which side walls exist on a track section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallMode {
    Both,
    LeftOnly,
    RightOnly,
    None,
}

impl WallMode {
    pub fn next(self) -> Self {
        match self {
            WallMode::Both => WallMode::LeftOnly,
            WallMode::LeftOnly => WallMode::RightOnly,
            WallMode::RightOnly => WallMode::None,
            WallMode::None => WallMode::Both,
        }
    }

    pub fn has_left(self) -> bool {
        matches!(self, WallMode::Both | WallMode::LeftOnly)
    }

    pub fn has_right(self) -> bool {
        matches!(self, WallMode::Both | WallMode::RightOnly)
    }

    pub fn label(self) -> &'static str {
        match self {
            WallMode::Both => "Both Walls",
            WallMode::LeftOnly => "Left Wall",
            WallMode::RightOnly => "Right Wall",
            WallMode::None => "Wall-less Drop-off",
        }
    }
}

/// Surface effect on a control span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    Normal,
    BoostPad,
    RechargeStrip,
    LowGripIce,
}

impl SurfaceType {
    pub fn next(self) -> Self {
        match self {
            SurfaceType::Normal => SurfaceType::BoostPad,
            SurfaceType::BoostPad => SurfaceType::RechargeStrip,
            SurfaceType::RechargeStrip => SurfaceType::LowGripIce,
            SurfaceType::LowGripIce => SurfaceType::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SurfaceType::Normal => "Standard Mag-Asphalt",
            SurfaceType::BoostPad => "Boost Chevron Pad",
            SurfaceType::RechargeStrip => "Shield Recharge Pit",
            SurfaceType::LowGripIce => "Zero-Friction Mag-Ice",
        }
    }
}

/// A single control node on a track segment spline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPoint {
    pub pos: Vec3,
    pub roll_deg: f32,
    pub width: f32,
    /// Cross-section curvature in `[-1.0, 1.0]` (`0.0` = Flat, `0.5` = Half-Pipe, `1.0` = 360 Cylinder).
    pub curvature: f32,
    pub walls: WallMode,
    pub has_floor: bool,
    pub surface: SurfaceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSegmentDef {
    pub id: usize,
    pub name: String,
    pub closed_loop: bool,
    pub points: Vec<ControlPoint>,
    pub next_segments: Vec<usize>,
    pub parent_anchor: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneryTower {
    pub pos: Vec3,
    pub size: Vec3,
    pub has_spire: bool,
}

fn default_track_file_path() -> String {
    ACTIVE_TRACK_PATH.to_string()
}

/// Complete serializable Track Definition (`assets/tracks/*.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDef {
    pub name: String,
    pub author: String,
    pub sky_top: [u8; 3],
    pub sky_horizon: [u8; 3],
    pub neon_primary: [u8; 3],
    pub neon_secondary: [u8; 3],
    pub segments: Vec<TrackSegmentDef>,
    #[serde(default)]
    pub scenery_towers: Vec<SceneryTower>,
    #[serde(skip, default = "default_track_file_path")]
    pub file_path: String,
}

impl TrackDef {
    pub fn apply_biome(&mut self, b: &BiomePalette) {
        self.sky_top = b.sky_top;
        self.sky_horizon = b.sky_horizon;
        self.neon_primary = b.neon_primary;
        self.neon_secondary = b.neon_secondary;
    }
}

/// Dense arc-length sample along a baked spline segment.
#[derive(Debug, Clone, Copy)]
pub struct BakedSample {
    pub s: f32,
    pub cp_index: usize,
    pub pos: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub width: f32,
    pub curvature: f32,
    pub walls: WallMode,
    pub has_floor: bool,
    pub surface: SurfaceType,
}

/// Evaluated 3D surface point and orientation frame at a specific `(s, r)` coordinate.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFrame {
    pub surface_pos: Vec3,
    pub forward: Vec3,
    pub normal: Vec3,
    pub right: Vec3,
    pub width: f32,
    pub curvature: f32,
    pub walls: WallMode,
    pub has_floor: bool,
    pub surface: SurfaceType,
    pub is_cylinder: bool,
}

#[inline]
pub fn is_full_cylinder_curvature(curvature: f32) -> bool {
    curvature.abs() > CYLINDER_CURVATURE_THRESHOLD
}

#[inline]
pub fn is_flat_ribbon_curvature(curvature: f32) -> bool {
    curvature.abs() < FLAT_CURVATURE_EPSILON
}

fn evaluate_flat_cross_section(
    sample: &BakedSample,
    r: f32,
    surface: SurfaceType,
    has_floor: bool,
    wall_mode: WallMode,
) -> SurfaceFrame {
    SurfaceFrame {
        surface_pos: sample.pos + sample.right * r,
        forward: sample.forward,
        normal: sample.up,
        right: sample.right,
        width: sample.width,
        curvature: sample.curvature.clamp(-1.0, 1.0),
        walls: wall_mode,
        has_floor,
        surface,
        is_cylinder: false,
    }
}

fn evaluate_curved_cross_section(
    sample: &BakedSample,
    r: f32,
    surface: SurfaceType,
    has_floor: bool,
    wall_mode: WallMode,
) -> SurfaceFrame {
    let c = sample.curvature.clamp(-1.0, 1.0);
    let is_cylinder = is_full_cylinder_curvature(c);
    let total_angle = c * 2.0 * PI;
    let radius = sample.width / total_angle.abs();
    let phi = (r / (sample.width * 0.5)) * (total_angle * 0.5);
    let sign = c.signum();

    let x_local = radius * phi.sin();
    let y_local = sign * radius * (1.0 - phi.cos());
    let surface_pos = sample.pos + sample.right * x_local + sample.up * y_local;

    let (sin_p, cos_p) = phi.sin_cos();
    let normal = (sample.up * cos_p - sample.right * sin_p).normalized();
    let right = (sample.right * cos_p + sample.up * sin_p).normalized();

    SurfaceFrame {
        surface_pos,
        forward: sample.forward,
        normal,
        right,
        width: sample.width,
        curvature: c,
        walls: if is_cylinder { WallMode::None } else { wall_mode },
        has_floor,
        surface,
        is_cylinder,
    }
}

impl BakedSample {
    pub fn evaluate_cross_section(&self, r: f32) -> SurfaceFrame {
        let c = self.curvature.clamp(-1.0, 1.0);
        if is_flat_ribbon_curvature(c) {
            evaluate_flat_cross_section(self, r, self.surface, self.has_floor, self.walls)
        } else {
            evaluate_curved_cross_section(self, r, self.surface, self.has_floor, self.walls)
        }
    }
}

fn interpolate_baked_samples(
    a: &BakedSample,
    b: &BakedSample,
    t: f32,
    target_s: f32,
) -> BakedSample {
    let forward = a.forward.lerp(b.forward, t).normalized();
    let up = a.up.lerp(b.up, t).normalized();
    let right = forward.cross(up).normalized();
    let up = right.cross(forward).normalized();

    BakedSample {
        s: target_s,
        cp_index: a.cp_index,
        pos: a.pos.lerp(b.pos, t),
        forward,
        up,
        right,
        width: a.width + (b.width - a.width) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        walls: a.walls,
        has_floor: a.has_floor,
        surface: a.surface,
    }
}

pub struct BakedSegment {
    pub id: usize,
    pub total_length: f32,
    pub base_race_distance: f32,
    pub closed_loop: bool,
    pub next_segments: Vec<usize>,
    pub samples: Vec<BakedSample>,
}

impl BakedSegment {
    pub fn sample_at(&self, mut s: f32) -> BakedSample {
        if self.samples.is_empty() {
            return BakedSample {
                s: 0.0,
                cp_index: 0,
                pos: Vec3::ZERO,
                forward: Vec3::FORWARD,
                up: Vec3::UP,
                right: Vec3::RIGHT,
                width: DEFAULT_TRACK_WIDTH,
                curvature: 0.0,
                walls: WallMode::Both,
                has_floor: true,
                surface: SurfaceType::Normal,
            };
        }
        if self.closed_loop && self.total_length > 0.0 {
            s = s.rem_euclid(self.total_length);
        } else {
            s = s.clamp(0.0, self.total_length);
        }

        let idx = match self
            .samples
            .binary_search_by(|probe| probe.s.partial_cmp(&s).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };

        let next_idx = if idx + 1 < self.samples.len() {
            idx + 1
        } else if self.closed_loop {
            0
        } else {
            idx
        };

        let a = &self.samples[idx];
        let b = &self.samples[next_idx];
        let span = if next_idx == 0 && self.closed_loop {
            (self.total_length - a.s).max(1e-4)
        } else {
            (b.s - a.s).max(1e-4)
        };
        let t = ((s - a.s) / span).clamp(0.0, 1.0);

        interpolate_baked_samples(a, b, t, s)
    }
}

pub struct BakedTrack {
    pub def: TrackDef,
    pub segments: Vec<BakedSegment>,
    pub primary_lap_length: f32,
}

impl BakedTrack {
    pub fn from_def(def: TrackDef) -> Self {
        let mut segments: Vec<BakedSegment> =
            def.segments.iter().map(Self::bake_segment).collect();

        if let Some(first) = segments.first() {
            let main_len = first.total_length;
            for i in 1..segments.len() {
                if let Some((parent_id, cp_idx)) = def.segments[i].parent_anchor {
                    if let Some(parent) = segments.get(parent_id) {
                        let base_s = parent
                            .samples
                            .iter()
                            .find(|s| s.cp_index == cp_idx)
                            .map(|s| s.s)
                            .unwrap_or(0.0);
                        segments[i].base_race_distance = base_s;
                    }
                }
            }
            Self {
                def,
                segments,
                primary_lap_length: main_len,
            }
        } else {
            Self {
                def,
                segments,
                primary_lap_length: DEFAULT_LAP_LENGTH,
            }
        }
    }

    #[inline]
    pub fn segment_or_main(&self, seg_id: usize) -> &BakedSegment {
        self.segments
            .iter()
            .find(|seg| seg.id == seg_id)
            .or_else(|| self.segments.get(seg_id))
            .unwrap_or(&self.segments[0])
    }

    fn bake_segment(seg_def: &TrackSegmentDef) -> BakedSegment {
        let pts = &seg_def.points;
        let n = pts.len();
        if n < 2 {
            return BakedSegment {
                id: seg_def.id,
                total_length: 1.0,
                base_race_distance: 0.0,
                closed_loop: seg_def.closed_loop,
                next_segments: seg_def.next_segments.clone(),
                samples: Vec::new(),
            };
        }

        let span_count = if seg_def.closed_loop { n } else { n - 1 };
        let steps_per_span = SAMPLES_PER_SPLINE_SPAN;
        let mut samples = Vec::with_capacity(span_count * steps_per_span + 1);

        let p0_init = if seg_def.closed_loop {
            pts[(n + n - 1) % n].pos
        } else {
            pts[0].pos
        };
        let mut prev_tangent =
            catmull_rom_tangent(p0_init, pts[0].pos, pts[1 % n].pos, pts[2 % n].pos, 0.0);

        let ref_up = if prev_tangent.dot(Vec3::UP).abs() > PARALLEL_UP_SINGULARITY_DOT {
            Vec3::FORWARD
        } else {
            Vec3::UP
        };
        let init_right = prev_tangent.cross(ref_up).normalized();
        let mut transport_up = init_right.cross(prev_tangent).normalized();

        let mut cumulative_s = 0.0f32;
        let mut prev_pos = pts[0].pos;

        for span_idx in 0..span_count {
            let i0 = if seg_def.closed_loop {
                (span_idx + n - 1) % n
            } else {
                span_idx.saturating_sub(1)
            };
            let i1 = span_idx;
            let i2 = (span_idx + 1) % n;
            let i3 = if seg_def.closed_loop {
                (span_idx + 2) % n
            } else {
                (span_idx + 2).min(n - 1)
            };

            let cp1 = &pts[i1];
            let cp2 = &pts[i2];

            for step in 0..steps_per_span {
                let t = step as f32 / steps_per_span as f32;
                let pos = catmull_rom_pos(pts[i0].pos, cp1.pos, cp2.pos, pts[i3].pos, t);
                let tangent = catmull_rom_tangent(pts[i0].pos, cp1.pos, cp2.pos, pts[i3].pos, t);

                if !(span_idx == 0 && step == 0) {
                    cumulative_s += (pos - prev_pos).length();
                    transport_up = transport_up.parallel_transport(prev_tangent, tangent);
                }
                prev_pos = pos;
                prev_tangent = tangent;

                let base_right = tangent.cross(transport_up).normalized();
                transport_up = base_right.cross(tangent).normalized();

                let roll_rad = smooth_lerp(cp1.roll_deg, cp2.roll_deg, t).to_radians();
                let up = transport_up.rotate_around_axis(tangent, roll_rad).normalized();
                let right = tangent.cross(up).normalized();

                samples.push(BakedSample {
                    s: cumulative_s,
                    cp_index: i1,
                    pos,
                    forward: tangent,
                    up,
                    right,
                    width: smooth_lerp(cp1.width, cp2.width, t),
                    curvature: smooth_lerp(cp1.curvature, cp2.curvature, t),
                    walls: cp1.walls,
                    has_floor: cp1.has_floor,
                    surface: cp1.surface,
                });
            }
        }

        if seg_def.closed_loop && !samples.is_empty() {
            cumulative_s += (pts[0].pos - prev_pos).length();
            let last_step_t = (steps_per_span - 1) as f32 / steps_per_span as f32;
            let last_roll_deg = smooth_lerp(pts[n - 1].roll_deg, pts[0].roll_deg, last_step_t);
            distribute_closed_loop_twist(&mut samples, pts[0].roll_deg, last_roll_deg, cumulative_s);
        }

        BakedSegment {
            id: seg_def.id,
            total_length: cumulative_s.max(1.0),
            base_race_distance: 0.0,
            closed_loop: seg_def.closed_loop,
            next_segments: seg_def.next_segments.clone(),
            samples,
        }
    }

    pub fn project_world_pos(&self, world_pos: Vec3, preferred_seg: usize) -> (usize, f32, f32, f32) {
        if self.segments.is_empty() {
            return (preferred_seg, 0.0, 0.0, 0.0);
        }
        let mut best_seg_idx: usize = 0;
        let mut best_seg_id: usize = 0;
        let mut best_dist_sq = f32::MAX;
        let mut best_s = 0.0;

        for (seg_idx, seg) in self.segments.iter().enumerate() {
            for sample in &seg.samples {
                let d_sq = (world_pos - sample.pos).length_sq();
                if d_sq < best_dist_sq {
                    best_dist_sq = d_sq;
                    best_seg_idx = seg_idx;
                    best_seg_id = seg.id;
                    best_s = sample.s;
                }
            }
        }

        let sample = self.segments[best_seg_idx].sample_at(best_s);
        let rel = world_pos - sample.pos;
        let r = rel.dot(sample.right);
        let frame = sample.evaluate_cross_section(r.clamp(-sample.width * 0.5, sample.width * 0.5));
        let h = (world_pos - frame.surface_pos).dot(frame.normal);

        (best_seg_id, best_s, r, h)
    }

    pub fn default_showcase_track() -> TrackDef {
        serde_json::from_str(include_str!("../assets/tracks/circuit_02.json"))
            .expect("Embedded assets/tracks/circuit_02.json must be valid TrackDef JSON")
    }
}

fn distribute_closed_loop_twist(
    samples: &mut [BakedSample],
    first_roll_deg: f32,
    last_roll_deg: f32,
    total_length: f32,
) {
    if samples.len() < 2 || total_length <= 1.0 {
        return;
    }
    let first_tangent = samples[0].forward;
    let first_base_up = samples[0]
        .up
        .rotate_around_axis(first_tangent, -first_roll_deg.to_radians());

    let last_idx = samples.len() - 1;
    let last_tangent = samples[last_idx].forward;
    let last_base_up = samples[last_idx]
        .up
        .rotate_around_axis(last_tangent, -last_roll_deg.to_radians());

    let transport_up = last_base_up.parallel_transport(last_tangent, first_tangent);
    let sin_err = transport_up.cross(first_base_up).dot(first_tangent);
    let cos_err = transport_up.dot(first_base_up).clamp(-1.0, 1.0);
    let twist_error = sin_err.atan2(cos_err);

    if twist_error.abs() > 1e-4 {
        for sample in samples.iter_mut() {
            let corr = twist_error * (sample.s / total_length);
            sample.up = sample
                .up
                .rotate_around_axis(sample.forward, corr)
                .normalized();
            sample.right = sample.forward.cross(sample.up).normalized();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_track_bakes_cleanly() {
        let def = BakedTrack::default_showcase_track();
        let track = BakedTrack::from_def(def);
        assert_eq!(track.segments.len(), 2);
        assert!(track.primary_lap_length > 1000.0);

        let main_seg = &track.segments[0];
        let s_flat = main_seg.sample_at(10.0);
        assert!(!s_flat.evaluate_cross_section(5.0).is_cylinder);

        let mut s_cyl = s_flat;
        s_cyl.curvature = 1.0;
        let left_edge = s_cyl.evaluate_cross_section(-s_cyl.width * 0.5);
        let right_edge = s_cyl.evaluate_cross_section(s_cyl.width * 0.5);
        assert!(left_edge.is_cylinder);
        assert!((left_edge.surface_pos - right_edge.surface_pos).length() < 0.05);
    }

    #[test]
    fn test_cross_section_flat_halfpipe_and_cylinder() {
        let mut sample = BakedSample {
            s: 0.0,
            cp_index: 0,
            pos: Vec3::ZERO,
            forward: Vec3::FORWARD,
            up: Vec3::UP,
            right: Vec3::RIGHT,
            width: 20.0,
            curvature: 0.0,
            walls: WallMode::Both,
            has_floor: true,
            surface: SurfaceType::Normal,
        };

        let frame_flat = sample.evaluate_cross_section(0.0);
        assert!(!frame_flat.is_cylinder);

        sample.curvature = 0.5;
        let frame_half = sample.evaluate_cross_section(0.0);
        assert!(!frame_half.is_cylinder);
        let left_edge = sample.evaluate_cross_section(-sample.width * 0.5);
        let right_edge = sample.evaluate_cross_section(sample.width * 0.5);
        assert!((left_edge.surface_pos - sample.pos).dot(sample.up) > 0.0);
        assert!((right_edge.surface_pos - sample.pos).dot(sample.up) > 0.0);

        sample.curvature = 1.0;
        let frame_cyl = sample.evaluate_cross_section(0.0);
        assert!(frame_cyl.is_cylinder);
    }

    #[test]
    fn test_project_world_pos_with_non_contiguous_segment_ids() {
        let def = TrackDef {
            name: "Test Track".to_string(),
            author: "Tester".to_string(),
            sky_top: [0, 0, 0],
            sky_horizon: [0, 0, 0],
            neon_primary: [0, 255, 0],
            neon_secondary: [255, 0, 0],
            segments: vec![TrackSegmentDef {
                id: 7,
                name: "Segment Seven".to_string(),
                closed_loop: false,
                points: vec![
                    ControlPoint {
                        pos: Vec3::new(0.0, 0.0, 0.0),
                        roll_deg: 0.0,
                        width: 20.0,
                        curvature: 0.0,
                        walls: WallMode::Both,
                        has_floor: true,
                        surface: SurfaceType::Normal,
                    },
                    ControlPoint {
                        pos: Vec3::new(0.0, 0.0, 100.0),
                        roll_deg: 0.0,
                        width: 20.0,
                        curvature: 0.0,
                        walls: WallMode::Both,
                        has_floor: true,
                        surface: SurfaceType::Normal,
                    },
                ],
                next_segments: Vec::new(),
                parent_anchor: None,
            }],
            scenery_towers: Vec::new(),
            file_path: "test.json".to_string(),
        };
        let track = BakedTrack::from_def(def);
        let (best_seg_id, best_s, _r, _h) = track.project_world_pos(Vec3::new(0.0, 0.0, 50.0), 0);
        assert_eq!(best_seg_id, 7);
        assert!((best_s - 50.0).abs() < 5.0);
    }
}
