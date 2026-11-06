use raylib::prelude::Vector3;
use serde::{Deserialize, Serialize};

const NORMALIZE_EPSILON: f32 = 1e-6;
const PARALLEL_TRANSPORT_EPSILON: f32 = 1e-5;

/// 3D vector that serializes as `[x, y, z]` in JSON while providing `.x`, `.y`, `.z` fields in Rust.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f32; 3]", into = "[f32; 3]")]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vec3> for [f32; 3] {
    #[inline]
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);
    pub const FORWARD: Self = Self::new(0.0, 0.0, 1.0);
    pub const RIGHT: Self = Self::new(1.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn x(self) -> f32 {
        self.x
    }

    #[inline]
    pub fn y(self) -> f32 {
        self.y
    }

    #[inline]
    pub fn z(self) -> f32 {
        self.z
    }

    #[inline]
    pub fn to_rl(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn distance_sq_to(self, other: Self) -> f32 {
        (self - other).length_sq()
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > NORMALIZE_EPSILON {
            self * (1.0 / len)
        } else {
            Self::UP
        }
    }

    #[inline]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        self + (rhs - self) * t
    }

    #[inline]
    pub fn smooth_damp(self, target: Self, rate: f32, dt: f32) -> Self {
        let factor = (rate * dt).clamp(0.0, 1.0);
        self.lerp(target, factor)
    }

    /// Rodrigues' rotation formula around `axis` by `angle_rad`.
    #[inline]
    pub fn rotate_around_axis(self, axis: Self, angle_rad: f32) -> Self {
        let k = axis.normalized();
        let (sin_a, cos_a) = angle_rad.sin_cos();
        self * cos_a + k.cross(self) * sin_a + k * (k.dot(self) * (1.0 - cos_a))
    }

    /// Minimal-rotation frame transport from tangent `t0` to `t1` to avoid gimbal lock on loops.
    #[inline]
    pub fn parallel_transport(self, t0: Self, t1: Self) -> Self {
        let binormal = t0.cross(t1);
        let binormal_len = binormal.length();
        if binormal_len < PARALLEL_TRANSPORT_EPSILON {
            self
        } else {
            let axis = binormal * (1.0 / binormal_len);
            let angle = t0.dot(t1).clamp(-1.0, 1.0).acos();
            self.rotate_around_axis(axis, angle).normalized()
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[inline]
pub fn lerp_f32(current: f32, target: f32, t: f32) -> f32 {
    current + (target - current) * t
}

#[inline]
pub fn smooth_damp_f32(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let factor = (rate * dt).clamp(0.0, 1.0);
    lerp_f32(current, target, factor)
}

#[inline]
pub fn catmull_rom_pos(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    let a = p1 * 2.0;
    let b = p2 - p0;
    let c = p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3;
    let d = -p0 + p1 * 3.0 - p2 * 3.0 + p3;
    (a + b * t + c * t2 + d * t3) * 0.5
}

#[inline]
pub fn catmull_rom_tangent(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let b = p2 - p0;
    let c = (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * (2.0 * t);
    let d = (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * (3.0 * t2);
    ((b + c + d) * 0.5).normalized()
}

#[inline]
pub fn smooth_lerp(a: f32, b: f32, t: f32) -> f32 {
    let s = t * t * (3.0 - 2.0 * t);
    a + (b - a) * s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_vec3_basic_arithmetic_and_dot_cross() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));

        let mut c = a;
        c += b;
        assert_eq!(c, Vec3::new(5.0, 7.0, 9.0));

        let mut d = b;
        d -= a;
        assert_eq!(d, Vec3::new(3.0, 3.0, 3.0));

        assert_eq!(Vec3::RIGHT.dot(Vec3::UP), 0.0);
        assert_eq!(Vec3::RIGHT.dot(Vec3::RIGHT), 1.0);
        assert_eq!(Vec3::RIGHT.cross(Vec3::UP), Vec3::FORWARD);
        assert_eq!(Vec3::UP.cross(Vec3::FORWARD), Vec3::RIGHT);

        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);

        let p1 = Vec3::new(1.0, 1.0, 1.0);
        let p2 = Vec3::new(1.0, 4.0, 5.0);
        assert_eq!(p1.distance_to(p2), 5.0);

        let unit_y = Vec3::new(0.0, 5.0, 0.0).normalized();
        assert_eq!(unit_y, Vec3::UP);
        assert_eq!(Vec3::ZERO.normalized(), Vec3::UP);
    }

    #[test]
    fn test_rodrigues_rotation_90_degrees() {
        let rotated = Vec3::RIGHT.rotate_around_axis(Vec3::UP, FRAC_PI_2);
        let expected = -Vec3::FORWARD;
        assert!((rotated.x - expected.x).abs() < 1e-4);
        assert!((rotated.y - expected.y).abs() < 1e-4);
        assert!((rotated.z - expected.z).abs() < 1e-4);
        assert!(rotated.distance_to(expected) < 1e-4);
    }

    #[test]
    fn test_parallel_transport_preserves_orthogonality() {
        let t0 = Vec3::FORWARD;
        let t1 = Vec3::new(0.0, 0.7071, 0.7071).normalized();
        let n0 = Vec3::UP;

        let n1 = n0.parallel_transport(t0, t1);
        assert!((n1.length() - 1.0).abs() < 1e-4);
        assert!(n1.dot(t1).abs() < 1e-4);
    }

    #[test]
    fn test_catmull_rom_endpoints_and_midpoint() {
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let p1 = Vec3::new(1.0, 2.0, 3.0);
        let p2 = Vec3::new(4.0, 6.0, 8.0);
        let p3 = Vec3::new(10.0, 12.0, 14.0);

        let start = catmull_rom_pos(p0, p1, p2, p3, 0.0);
        assert!(start.distance_to(p1) < 1e-5);

        let end = catmull_rom_pos(p0, p1, p2, p3, 1.0);
        assert!(end.distance_to(p2) < 1e-5);

        let tan_start = catmull_rom_tangent(p0, p1, p2, p3, 0.0);
        assert!((tan_start.length() - 1.0).abs() < 1e-4);

        let tan_mid = catmull_rom_tangent(p0, p1, p2, p3, 0.5);
        assert!((tan_mid.length() - 1.0).abs() < 1e-4);

        let tan_end = catmull_rom_tangent(p0, p1, p2, p3, 1.0);
        assert!((tan_end.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_smooth_damp_f32_clamps_overshoot() {
        let result = smooth_damp_f32(0.0, 10.0, 50.0, 1.0);
        assert_eq!(result, 10.0);
    }
}
