use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Ship handling, boost, collision, and camera tuning loaded from `assets/config/physics.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicsConfig {
    pub max_speed: f32,
    pub limp_speed: f32,
    pub boost_speed: f32,
    pub acceleration: f32,
    pub boost_acceleration: f32,
    pub coast_deceleration: f32,
    pub dual_airbrake_decel: f32,
    pub single_airbrake_drag: f32,
    pub steer_lateral_speed: f32,
    pub airbrake_lateral_speed: f32,
    pub lateral_grip: f32,
    pub ice_grip_multiplier: f32,
    pub centrifugal_factor: f32,
    pub hover_target_height: f32,
    pub hover_spring_stiffness: f32,
    pub hover_spring_damping: f32,
    pub boost_energy_cost: f32,
    pub boost_duration_sec: f32,
    pub boost_pad_impulse: f32,
    pub recharge_strip_rate: f32,
    pub wall_restitution: f32,
    pub wall_speed_retention: f32,
    pub wall_damage_factor: f32,
    pub glide_gravity_nose_up: f32,
    pub glide_gravity_normal: f32,
    pub takeoff_vertical_boost: f32,
    pub void_respawn_height: f32,
    pub side_attack_impulse: f32,
    pub side_attack_duration: f32,
    pub ship_collision_length: f32,
    pub ship_collision_width: f32,
    pub camera_distance: f32,
    pub camera_height: f32,
    pub camera_lookahead: f32,
    pub camera_base_fov: f32,
    pub camera_boost_fov: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            max_speed: 82.0,
            limp_speed: 55.0,
            boost_speed: 116.0,
            acceleration: 42.0,
            boost_acceleration: 78.0,
            coast_deceleration: 22.0,
            dual_airbrake_decel: 48.0,
            single_airbrake_drag: 6.0,
            steer_lateral_speed: 24.0,
            airbrake_lateral_speed: 20.0,
            lateral_grip: 11.5,
            ice_grip_multiplier: 0.35,
            centrifugal_factor: 0.22,
            hover_target_height: 1.25,
            hover_spring_stiffness: 95.0,
            hover_spring_damping: 14.0,
            boost_energy_cost: 14.0,
            boost_duration_sec: 1.15,
            boost_pad_impulse: 68.0,
            recharge_strip_rate: 38.0,
            wall_restitution: 0.55,
            wall_speed_retention: 0.94,
            wall_damage_factor: 0.18,
            glide_gravity_nose_up: 18.0,
            glide_gravity_normal: 34.0,
            takeoff_vertical_boost: 4.5,
            void_respawn_height: -28.0,
            side_attack_impulse: 22.0,
            side_attack_duration: 0.45,
            ship_collision_length: 3.4,
            ship_collision_width: 2.2,
            camera_distance: 9.5,
            camera_height: 4.6,
            camera_lookahead: 24.0,
            camera_base_fov: 62.0,
            camera_boost_fov: 76.0,
        }
    }
}

fn load_json_or_write_default<T>(path: &str, make_default: fn() -> T) -> T
where
    T: Serialize + serde::de::DeserializeOwned,
{
    if Path::new(path).exists() {
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<T>(&text) {
                Ok(val) => val,
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to parse {}: {}. Using in-memory defaults without overwriting file.",
                        path,
                        err
                    );
                    make_default()
                }
            },
            Err(err) => {
                eprintln!(
                    "Warning: Failed to parse {}: {}. Using in-memory defaults without overwriting file.",
                    path,
                    err
                );
                make_default()
            }
        }
    } else {
        let val = make_default();
        if let Some(parent) = Path::new(path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&val) {
            let _ = fs::write(path, json);
        }
        val
    }
}

impl PhysicsConfig {
    pub const PATH: &'static str = "assets/config/physics.json";

    pub fn load() -> Self {
        load_json_or_write_default(Self::PATH, Self::default)
    }
}

/// Cel-shaded biome color palette loaded from `assets/config/biomes.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomePalette {
    pub name: String,
    pub sky_top: [u8; 3],
    pub sky_horizon: [u8; 3],
    pub neon_primary: [u8; 3],
    pub neon_secondary: [u8; 3],
}

impl BiomePalette {
    pub const PATH: &'static str = "assets/config/biomes.json";

    pub fn default_biomes() -> Vec<Self> {
        vec![Self {
            name: "NEO-TOKYO SUNSET (Cobalt & Golden Amber)".to_string(),
            sky_top: [38, 78, 195],
            sky_horizon: [255, 175, 85],
            neon_primary: [0, 240, 255],
            neon_secondary: [255, 55, 115],
        }]
    }

    pub fn load_all() -> Vec<Self> {
        let list = load_json_or_write_default(Self::PATH, Self::default_biomes);
        if list.is_empty() {
            Self::default_biomes()
        } else {
            list
        }
    }
}

/// Initial starting grid ship definition loaded from `assets/config/roster.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipRosterEntry {
    pub id: u8,
    pub name: String,
    pub primary_color: [u8; 3],
    pub accent_color: [u8; 3],
    pub start_s: f32,
    pub start_r: f32,
    pub is_ai: bool,
}

impl ShipRosterEntry {
    pub const PATH: &'static str = "assets/config/roster.json";

    pub fn default_roster() -> Vec<Self> {
        vec![
            Self {
                id: 0,
                name: "PLAYER 1".to_string(),
                primary_color: [235, 245, 255],
                accent_color: [0, 235, 255],
                start_s: 12.0,
                start_r: -3.5,
                is_ai: false,
            },
            Self {
                id: 1,
                name: "FEISAR-X".to_string(),
                primary_color: [255, 195, 35],
                accent_color: [35, 150, 255],
                start_s: 18.0,
                start_r: 3.5,
                is_ai: true,
            },
            Self {
                id: 2,
                name: "AG-SYS".to_string(),
                primary_color: [255, 55, 95],
                accent_color: [255, 240, 240],
                start_s: 24.0,
                start_r: -3.5,
                is_ai: true,
            },
            Self {
                id: 3,
                name: "QIREX-ZERO".to_string(),
                primary_color: [175, 65, 255],
                accent_color: [80, 255, 190],
                start_s: 30.0,
                start_r: 3.5,
                is_ai: true,
            },
        ]
    }

    pub fn load_all() -> Vec<Self> {
        let list = load_json_or_write_default(Self::PATH, Self::default_roster);
        if list.is_empty() {
            Self::default_roster()
        } else {
            list
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_config_default_roundtrip_json() {
        let original = PhysicsConfig::default();
        let json = serde_json::to_string_pretty(&original).expect("serialize default config");
        let deserialized: PhysicsConfig =
            serde_json::from_str(&json).expect("deserialize default config");
        assert_eq!(deserialized.max_speed, original.max_speed);
        assert_eq!(deserialized.boost_speed, original.boost_speed);
        assert_eq!(deserialized.wall_restitution, original.wall_restitution);
        assert_eq!(deserialized.camera_distance, original.camera_distance);
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_physics_config_partial_json_uses_serde_defaults() {
        let json = r#"{"max_speed": 99.5}"#;
        let cfg: PhysicsConfig =
            serde_json::from_str(json).expect("deserialize partial physics config");
        let default_cfg = PhysicsConfig::default();
        assert_eq!(cfg.max_speed, 99.5);
        assert_eq!(cfg.wall_restitution, default_cfg.wall_restitution);
        assert_eq!(cfg.boost_speed, default_cfg.boost_speed);
        assert_eq!(cfg.hover_target_height, default_cfg.hover_target_height);
    }

    #[test]
    fn test_malformed_json_is_rejected_without_panic() {
        let result = serde_json::from_str::<PhysicsConfig>("{ bad json ");
        assert!(result.is_err());
    }

    #[test]
    fn test_ship_roster_entry_roundtrip_json() {
        let roster = ShipRosterEntry::default_roster();
        let json = serde_json::to_string_pretty(&roster).expect("serialize default roster");
        let deserialized: Vec<ShipRosterEntry> =
            serde_json::from_str(&json).expect("deserialize default roster");
        assert_eq!(deserialized.len(), 4);
        assert_eq!(deserialized[0].name, "PLAYER 1");
        assert_eq!(deserialized, roster);
    }
}
