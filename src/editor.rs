use crate::math::Vec3;
use crate::track::{BakedSample, BakedTrack, ControlPoint, TrackDef, ACTIVE_TRACK_PATH};
use raylib::prelude::*;
use std::fs;

pub const MOUSE_LOOK_SENSITIVITY: f32 = 0.0045;
pub const CAMERA_FAST_SPEED: f32 = 160.0;
pub const CAMERA_NORMAL_SPEED: f32 = 65.0;
pub const NODE_FAST_MOVE_SPEED: f32 = 65.0;
pub const NODE_NORMAL_MOVE_SPEED: f32 = 24.0;
pub const ROLL_EDIT_RATE_DEG: f32 = 60.0;
pub const WIDTH_EDIT_RATE: f32 = 12.0;
pub const MIN_TRACK_WIDTH: f32 = 8.0;
pub const MAX_TRACK_WIDTH: f32 = 48.0;
pub const CURVATURE_EDIT_RATE: f32 = 0.8;
pub const STATUS_MESSAGE_DURATION_SEC: f32 = 4.0;
pub const DEFAULT_EDITOR_CAM_POS: Vec3 = Vec3::new(-40.0, 35.0, -35.0);
pub const DEFAULT_EDITOR_YAW: f32 = 0.75;
pub const DEFAULT_EDITOR_PITCH: f32 = -0.35;
pub const MAX_CAMERA_PITCH_RAD: f32 = 1.45;
pub const FOCUS_BACK_DISTANCE: f32 = 45.0;
pub const FOCUS_HEIGHT_OFFSET: f32 = 12.0;
pub const EDITOR_CAMERA_FOV: f32 = 65.0;
pub const NEW_NODE_LIFT_Y: f32 = 2.0;
pub const MIN_SEGMENT_POINTS: usize = 4;
pub const AXIS_GIZMO_LENGTH: f32 = 12.0;
pub const RING_PREVIEW_STEPS: usize = 16;
pub const RING_NORMAL_OFFSET: f32 = 0.3;
pub const SELECTED_NODE_RADIUS: f32 = 2.2;
pub const UNSELECTED_NODE_RADIUS: f32 = 1.2;
pub const NODE_SPHERE_RINGS: i32 = 6;

pub fn cycle_curvature_preset(curvature: f32) -> f32 {
    if curvature.abs() < 0.1 {
        0.5
    } else if (curvature - 0.5).abs() < 0.15 {
        1.0
    } else if (curvature - 1.0).abs() < 0.15 {
        -0.5
    } else {
        0.0
    }
}

pub fn describe_curvature(curvature: f32) -> &'static str {
    if curvature.abs() < 0.1 {
        "Flat Ribbon"
    } else if curvature > 0.85 {
        "360 Inside Cylinder"
    } else if curvature > 0.2 {
        "Half-Pipe Channel"
    } else {
        "Outside Pipe / Ridge"
    }
}

/// Interactive In-Game 3D Spline Track Editor (Toggled via `Tab`).
pub struct TrackEditor {
    pub active: bool,
    pub selected_seg: usize,
    pub selected_cp: usize,
    pub cam_pos: Vec3,
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    pub status_msg: String,
    pub status_timer: f32,
}

impl TrackEditor {
    pub fn new() -> Self {
        Self {
            active: false,
            selected_seg: 0,
            selected_cp: 0,
            cam_pos: DEFAULT_EDITOR_CAM_POS,
            cam_yaw: DEFAULT_EDITOR_YAW,
            cam_pitch: DEFAULT_EDITOR_PITCH,
            status_msg: "EDITOR READY - Press TAB to toggle back to racing".to_string(),
            status_timer: 0.0,
        }
    }

    pub fn camera_forward(&self) -> Vec3 {
        Vec3::new(
            self.cam_yaw.cos() * self.cam_pitch.cos(),
            self.cam_pitch.sin(),
            self.cam_yaw.sin() * self.cam_pitch.cos(),
        )
        .normalized()
    }

    pub fn focus_on_selected(&mut self, def: &TrackDef) {
        if let Some(seg) = def.segments.get(self.selected_seg) {
            if let Some(cp) = seg.points.get(self.selected_cp) {
                let forward = self.camera_forward();
                self.cam_pos = cp.pos - forward * FOCUS_BACK_DISTANCE + Vec3::UP * FOCUS_HEIGHT_OFFSET;
            }
        }
    }

    pub fn update_free_fly_camera(&mut self, rl: &RaylibHandle, dt: f32) {
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            let delta = rl.get_mouse_delta();
            self.cam_yaw += delta.x * MOUSE_LOOK_SENSITIVITY;
            self.cam_pitch = (self.cam_pitch - delta.y * MOUSE_LOOK_SENSITIVITY)
                .clamp(-MAX_CAMERA_PITCH_RAD, MAX_CAMERA_PITCH_RAD);
        }

        let cam_fwd = self.camera_forward();
        let cam_right = cam_fwd.cross(Vec3::UP).normalized();

        let fly_speed = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            CAMERA_FAST_SPEED
        } else {
            CAMERA_NORMAL_SPEED
        };

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            if rl.is_key_down(KeyboardKey::KEY_W) {
                self.cam_pos += cam_fwd * (fly_speed * dt);
            }
            if rl.is_key_down(KeyboardKey::KEY_S) {
                self.cam_pos -= cam_fwd * (fly_speed * dt);
            }
            if rl.is_key_down(KeyboardKey::KEY_D) {
                self.cam_pos += cam_right * (fly_speed * dt);
            }
            if rl.is_key_down(KeyboardKey::KEY_A) {
                self.cam_pos -= cam_right * (fly_speed * dt);
            }
            if rl.is_key_down(KeyboardKey::KEY_E) || rl.is_key_down(KeyboardKey::KEY_SPACE) {
                self.cam_pos += Vec3::UP * (fly_speed * dt);
            }
            if rl.is_key_down(KeyboardKey::KEY_Q) {
                self.cam_pos -= Vec3::UP * (fly_speed * dt);
            }
        }
    }

    pub fn update_node_and_segment_selection(&mut self, rl: &RaylibHandle, track_def: &TrackDef) {
        if track_def.segments.is_empty() {
            return;
        }
        self.selected_seg = self.selected_seg.min(track_def.segments.len() - 1);
        let pts_len = track_def.segments[self.selected_seg].points.len();
        if pts_len == 0 {
            return;
        }
        self.selected_cp = self.selected_cp.min(pts_len - 1);

        if rl.is_key_pressed(KeyboardKey::KEY_PERIOD) || rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            self.selected_cp = (self.selected_cp + 1) % pts_len;
            self.focus_on_selected(track_def);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_COMMA) || rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            self.selected_cp = (self.selected_cp + pts_len - 1) % pts_len;
            self.focus_on_selected(track_def);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_V) {
            self.selected_seg = (self.selected_seg + 1) % track_def.segments.len();
            self.selected_cp = 0;
            self.focus_on_selected(track_def);
            self.set_status(format!(
                "Switched to Segment #{}: {}",
                self.selected_seg, track_def.segments[self.selected_seg].name
            ));
        }
    }

    pub fn translate_selected_point(rl: &RaylibHandle, cp: &mut ControlPoint, dt: f32) -> bool {
        let move_speed = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            NODE_FAST_MOVE_SPEED
        } else {
            NODE_NORMAL_MOVE_SPEED
        };
        let step = move_speed * dt;
        let mut modified = false;

        if rl.is_key_down(KeyboardKey::KEY_UP) {
            cp.pos.z += step;
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            cp.pos.z -= step;
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            cp.pos.x += step;
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            cp.pos.x -= step;
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_R) {
            cp.pos.y += step;
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_F) {
            cp.pos.y -= step;
            modified = true;
        }

        modified
    }

    pub fn adjust_selected_point_attributes(rl: &RaylibHandle, cp: &mut ControlPoint, dt: f32) -> bool {
        let mut modified = false;

        if !rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            if rl.is_key_down(KeyboardKey::KEY_Q) {
                cp.roll_deg -= ROLL_EDIT_RATE_DEG * dt;
                modified = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_E) {
                cp.roll_deg += ROLL_EDIT_RATE_DEG * dt;
                modified = true;
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_LEFT_BRACKET) {
            cp.width = (cp.width - WIDTH_EDIT_RATE * dt).max(MIN_TRACK_WIDTH);
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT_BRACKET) {
            cp.width = (cp.width + WIDTH_EDIT_RATE * dt).min(MAX_TRACK_WIDTH);
            modified = true;
        }

        if rl.is_key_down(KeyboardKey::KEY_MINUS) {
            cp.curvature = (cp.curvature - CURVATURE_EDIT_RATE * dt).clamp(-1.0, 1.0);
            modified = true;
        }
        if rl.is_key_down(KeyboardKey::KEY_EQUAL) {
            cp.curvature = (cp.curvature + CURVATURE_EDIT_RATE * dt).clamp(-1.0, 1.0);
            modified = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            cp.curvature = cycle_curvature_preset(cp.curvature);
            modified = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            cp.walls = cp.walls.next();
            modified = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_G) {
            cp.has_floor = !cp.has_floor;
            modified = true;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            cp.surface = cp.surface.next();
            modified = true;
        }

        modified
    }

    pub fn handle_insert_or_delete_point(&mut self, rl: &RaylibHandle, track_def: &mut TrackDef) -> bool {
        if track_def.segments.is_empty() {
            return false;
        }
        let seg = &mut track_def.segments[self.selected_seg];
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            let n = seg.points.len();
            let cur = seg.points[self.selected_cp].clone();
            let next_pos = seg.points[(self.selected_cp + 1) % n].pos;
            let new_pt = ControlPoint {
                pos: cur.pos.lerp(next_pos, 0.5) + Vec3::UP * NEW_NODE_LIFT_Y,
                ..cur
            };
            let insert_idx = self.selected_cp + 1;
            seg.points.insert(insert_idx, new_pt);
            self.selected_cp = insert_idx;
            self.set_status(format!("Inserted new Control Point #{}", self.selected_cp));
            return true;
        }

        if (rl.is_key_pressed(KeyboardKey::KEY_DELETE) || rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE))
            && seg.points.len() > MIN_SEGMENT_POINTS
        {
            seg.points.remove(self.selected_cp);
            let new_len = seg.points.len();
            self.selected_cp = self.selected_cp.min(new_len - 1);
            self.set_status("Removed Control Point".to_string());
            return true;
        }

        false
    }

    pub fn save_track_to_disk(&mut self, track_def: &TrackDef) {
        let target_path = if track_def.file_path.is_empty() {
            ACTIVE_TRACK_PATH
        } else {
            &track_def.file_path
        };
        let _ = fs::create_dir_all("assets/tracks");
        if let Ok(json) = serde_json::to_string_pretty(track_def) {
            if fs::write(target_path, json).is_ok() {
                self.set_status(format!("Saved Track JSON to {}", target_path));
            }
        }
    }

    /// Update editor fly-cam and control-point manipulation; returns `true` if the track was modified.
    pub fn update(&mut self, rl: &RaylibHandle, def: &mut TrackDef, dt: f32) -> bool {
        self.status_timer = (self.status_timer - dt).max(0.0);
        let mut modified = false;

        self.update_free_fly_camera(rl, dt);
        self.update_node_and_segment_selection(rl, def);

        if def.segments.is_empty() || def.segments[self.selected_seg].points.is_empty() {
            return false;
        }

        let cp = &mut def.segments[self.selected_seg].points[self.selected_cp];
        if Self::translate_selected_point(rl, cp, dt) {
            modified = true;
        }
        if Self::adjust_selected_point_attributes(rl, cp, dt) {
            modified = true;
        }

        if self.handle_insert_or_delete_point(rl, def) {
            modified = true;
        }

        let ctrl_s = (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
            && rl.is_key_pressed(KeyboardKey::KEY_S);
        if rl.is_key_pressed(KeyboardKey::KEY_K) || ctrl_s {
            self.save_track_to_disk(def);
        }

        modified
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_msg = msg;
        self.status_timer = STATUS_MESSAGE_DURATION_SEC;
    }

    pub fn camera3d(&self) -> Camera3D {
        let forward = self.camera_forward();
        Camera3D::perspective(
            self.cam_pos.to_rl(),
            (self.cam_pos + forward * 10.0).to_rl(),
            Vector3::new(0.0, 1.0, 0.0),
            EDITOR_CAMERA_FOV,
        )
    }

    pub fn draw_control_point_axes(d: &mut RaylibMode3D<RaylibDrawHandle>, pos: Vec3) {
        d.draw_line_3D(
            pos.to_rl(),
            (pos + Vec3::RIGHT * AXIS_GIZMO_LENGTH).to_rl(),
            Color::new(255, 60, 60, 255),
        );
        d.draw_line_3D(
            pos.to_rl(),
            (pos + Vec3::UP * AXIS_GIZMO_LENGTH).to_rl(),
            Color::new(60, 255, 90, 255),
        );
        d.draw_line_3D(
            pos.to_rl(),
            (pos + Vec3::FORWARD * AXIS_GIZMO_LENGTH).to_rl(),
            Color::new(70, 140, 255, 255),
        );
    }

    pub fn draw_cross_section_preview_ring(
        d: &mut RaylibMode3D<RaylibDrawHandle>,
        sample: &BakedSample,
    ) {
        for k in 0..RING_PREVIEW_STEPS {
            let r0 = ((k as f32 / RING_PREVIEW_STEPS as f32) - 0.5) * sample.width;
            let r1 = (((k + 1) as f32 / RING_PREVIEW_STEPS as f32) - 0.5) * sample.width;
            let f0 = sample.evaluate_cross_section(r0);
            let f1 = sample.evaluate_cross_section(r1);
            d.draw_line_3D(
                (f0.surface_pos + f0.normal * RING_NORMAL_OFFSET).to_rl(),
                (f1.surface_pos + f1.normal * RING_NORMAL_OFFSET).to_rl(),
                Color::new(255, 255, 80, 255),
            );
        }
    }

    /// Draw 3D spline node gizmos, bank-angle arrows, and cross-section rings.
    pub fn draw_3d_gizmos(&self, d: &mut RaylibMode3D<RaylibDrawHandle>, track: &BakedTrack) {
        for (seg_idx, seg_def) in track.def.segments.iter().enumerate() {
            for (cp_idx, cp) in seg_def.points.iter().enumerate() {
                let is_selected = seg_idx == self.selected_seg && cp_idx == self.selected_cp;
                let col = if is_selected {
                    Color::new(255, 255, 60, 255)
                } else if seg_idx == 0 {
                    Color::new(0, 220, 255, 200)
                } else {
                    Color::new(255, 90, 200, 200)
                };

                let radius = if is_selected {
                    SELECTED_NODE_RADIUS
                } else {
                    UNSELECTED_NODE_RADIUS
                };
                d.draw_sphere_wires(
                    cp.pos.to_rl(),
                    radius,
                    NODE_SPHERE_RINGS,
                    NODE_SPHERE_RINGS,
                    col,
                );

                if is_selected {
                    Self::draw_control_point_axes(d, cp.pos);

                    if let Some(baked_seg) = track.segments.get(seg_idx) {
                        if let Some(sample) = baked_seg.samples.iter().find(|s| s.cp_index == cp_idx) {
                            Self::draw_cross_section_preview_ring(d, sample);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_gizmos(&self, d: &mut RaylibMode3D<RaylibDrawHandle>, track: &BakedTrack) {
        self.draw_3d_gizmos(d, track);
    }

    /// Draw the 2D Track Editor overlay panel.
    pub fn draw_hud(&self, d: &mut RaylibDrawHandle, def: &TrackDef) {
        if def.segments.is_empty() {
            return;
        }
        let seg_idx = self.selected_seg.min(def.segments.len() - 1);
        if def.segments[seg_idx].points.is_empty() {
            return;
        }
        let cp_idx = self.selected_cp.min(def.segments[seg_idx].points.len() - 1);

        d.draw_rectangle(14, 14, 455, 255, Color::new(10, 14, 28, 225));
        d.draw_rectangle_lines(14, 14, 455, 255, Color::new(255, 220, 40, 255));

        d.draw_text(
            "3D SPLINE TRACK EDITOR [TAB: RETURN TO RACE]",
            26,
            24,
            16,
            Color::new(255, 230, 60, 255),
        );

        let seg = &def.segments[seg_idx];
        let cp = &seg.points[cp_idx];

        d.draw_text(
            &format!(
                "Segment: #{} ({})  |  Node: #{} / {}",
                seg_idx,
                seg.name,
                cp_idx,
                seg.points.len()
            ),
            26,
            50,
            16,
            Color::WHITE,
        );
        d.draw_text(
            &format!(
                "Pos: [{:.1}, {:.1}, {:.1}]   Roll: {:.1} deg   Width: {:.1}m",
                cp.pos.x,
                cp.pos.y,
                cp.pos.z,
                cp.roll_deg,
                cp.width
            ),
            26,
            72,
            15,
            Color::new(140, 235, 255, 255),
        );

        let curve_desc = describe_curvature(cp.curvature);
        d.draw_text(
            &format!(
                "Curvature: {:.2} ({})  |  Floor: {}",
                cp.curvature,
                curve_desc,
                if cp.has_floor { "SOLID" } else { "JUMP CHASM" }
            ),
            26,
            94,
            15,
            Color::new(255, 160, 240, 255),
        );
        d.draw_text(
            &format!(
                "Walls: {}  |  Surface: {}",
                cp.walls.label(),
                cp.surface.label()
            ),
            26,
            116,
            15,
            Color::new(120, 255, 160, 255),
        );

        let help_col = Color::new(190, 200, 220, 255);
        d.draw_text(
            "[,] / [.] : Prev/Next Node    [V] : Switch Split Segment",
            26,
            146,
            14,
            help_col,
        );
        d.draw_text(
            "[Arrows + R/F] : Move Node    [Q/E] : Bank Roll   [[ / ]] : Width",
            26,
            165,
            14,
            help_col,
        );
        d.draw_text(
            "[C] or [-/=] : Morph Flat / Half-Pipe / 360 Cylinder",
            26,
            184,
            14,
            help_col,
        );
        d.draw_text(
            "[T] : Surface Zone   [M] : Walls   [G] : Toggle Jump Chasm",
            26,
            203,
            14,
            help_col,
        );
        d.draw_text(
            "[N] : Insert Node  [Del] : Delete  [K / Ctrl+S] : Save JSON",
            26,
            222,
            14,
            help_col,
        );
        d.draw_text(
            "[Hold Right-Mouse + WASD] : Free-Fly Camera",
            26,
            241,
            14,
            Color::new(255, 210, 90, 255),
        );

        if self.status_timer > 0.0 {
            d.draw_text(
                &self.status_msg,
                26,
                278,
                18,
                Color::new(80, 255, 140, 255),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_forward_unit_vector() {
        let mut editor = TrackEditor::new();
        editor.cam_yaw = 0.0;
        editor.cam_pitch = 0.0;
        let fwd = editor.camera_forward();
        assert!((fwd.length() - 1.0).abs() < 1e-4);
        assert!((fwd.x - 1.0).abs() < 1e-4);
        assert!(fwd.y.abs() < 1e-4);
        assert!(fwd.z.abs() < 1e-4);
    }

    #[test]
    fn test_focus_on_selected_empty_segments_no_panic() {
        let mut editor = TrackEditor::new();
        let mut def = BakedTrack::default_showcase_track();
        def.segments.clear();
        editor.focus_on_selected(&def);
    }

    #[test]
    fn test_cycle_and_describe_curvature() {
        assert_eq!(describe_curvature(0.0), "Flat Ribbon");
        assert_eq!(describe_curvature(0.5), "Half-Pipe Channel");
        assert_eq!(describe_curvature(1.0), "360 Inside Cylinder");
        assert_eq!(describe_curvature(-0.5), "Outside Pipe / Ridge");

        assert!((cycle_curvature_preset(0.0) - 0.5).abs() < 1e-4);
        assert!((cycle_curvature_preset(0.5) - 1.0).abs() < 1e-4);
        assert!((cycle_curvature_preset(1.0) - (-0.5)).abs() < 1e-4);
        assert!((cycle_curvature_preset(-0.5) - 0.0).abs() < 1e-4);
    }
}
