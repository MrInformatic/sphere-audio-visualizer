//! contains basic functions which are reimplemented in inline assembly for
//! spirv compile target to optimize computation.

#[cfg(target_arch = "spirv")]
use core::arch::asm;
#[cfg(target_arch = "spirv")]
use glam::Vec4;
use glam::{Mat4, Vec2, Vec3A};
#[cfg(target_arch = "spirv")]
use num_traits::Float;

/// calculates Shlick's approximation <https://en.wikipedia.org/wiki/Schlick%27s_approximation>
/// of the Fresnel equation <https://en.wikipedia.org/wiki/Fresnel_equations>
pub fn shlick(direction: &Vec3A, normal: &Vec3A, n1: f32, n2: f32) -> f32 {
    let dot = dot(direction, normal);
    let r = (n1 - n2) / (n1 + n2);
    let r2 = r * r;
    r2 + (1.0 - r2) * (1.0 + dot).powf(5.0)
}

/// Applies filmic tonemaping
pub fn tonemap_filmic(x: &Vec3A) -> Vec3A {
    let x2 = Vec3A::splat(0.0).max(*x - 0.004);
    let result = (x2 * (6.2 * x2 + 0.5)) / (x2 * (6.2 * x2 + 1.7) + 0.06);
    return result.powf(2.2);
}

/// normalizes a vector
#[cfg(target_arch = "spirv")]
#[inline]
pub fn normalize(value: &Vec3A) -> Vec3A {
    unsafe {
        let mut result = Vec3A::ZERO;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%value = OpLoad _ {value}",
            "%result = OpExtInst typeof*{result} %1 69 %value",
            "OpStore {result} %result",
            value = in(reg) value,
            result = in(reg) &mut result,
        }
        result
    }
}

/// normalizes a vector
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn normalize(value: &Vec3A) -> Vec3A {
    value.normalize()
}

/// reflects `direction` at `normal`
#[cfg(target_arch = "spirv")]
#[inline]
pub fn reflect(direction: &Vec3A, normal: &Vec3A) -> Vec3A {
    unsafe {
        let mut result = Vec3A::ZERO;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%direction = OpLoad _ {direction}",
            "%normal = OpLoad _ {normal}",
            "%result = OpExtInst typeof*{result} %1 71 %direction %normal",
            "OpStore {result} %result",
            direction = in(reg) direction,
            normal = in(reg) normal,
            result = in(reg) &mut result,
        }
        result
    }
}

/// reflects `direction` at `normal`
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn reflect(direction: &Vec3A, normal: &Vec3A) -> Vec3A {
    *direction + (*normal * (dot(direction, normal) * -2.0))
}

/// calculates the distance between a and b
#[cfg(target_arch = "spirv")]
#[inline]
pub fn distance(a: &Vec3A, b: &Vec3A) -> f32 {
    unsafe {
        let mut result = 0.0;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%a = OpLoad _ {a}",
            "%b = OpLoad _ {b}",
            "%result = OpExtInst typeof*{result} %1 67 %a %b",
            "OpStore {result} %result",
            a = in(reg) a,
            b = in(reg) b,
            result = in(reg) &mut result,
        }
        result
    }
}

/// calculates the distance between a and b
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn distance(a: &Vec3A, b: &Vec3A) -> f32 {
    a.distance(*b)
}

/// Calcualtes the inverse square root
#[cfg(target_arch = "spirv")]
#[inline]
pub fn inverse_sqrt(value: f32) -> f32 {
    unsafe {
        let mut result: f32 = 0.0;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%result = OpExtInst typeof*{result} %1 32 {value}",
            "OpStore {result} %result",
            value = in(reg) value,
            result = in(reg) &mut result,
        }
        result
    }
}

/// Calcualtes the inverse square root
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn inverse_sqrt(value: f32) -> f32 {
    1.0 / value.sqrt()
}

/// Calcualtes the dot product between a and b
#[cfg(target_arch = "spirv")]
#[inline]
pub fn dot(a: &Vec3A, b: &Vec3A) -> f32 {
    unsafe {
        let mut result: f32 = 0.0;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%a = OpLoad _ {a}",
            "%b = OpLoad _ {b}",
            "%result = OpDot typeof*{result} %a %b",
            "OpStore {result} %result",
            a = in(reg) a,
            b = in(reg) b,
            result = in(reg) &mut result,
        }
        result
    }
}

/// Calcualtes the dot product between a and b
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn dot(a: &Vec3A, b: &Vec3A) -> f32 {
    a.dot(*b)
}

/// Calcualtes the dot product between a and b
#[cfg(target_arch = "spirv")]
#[inline]
pub fn dot2(a: &Vec2, b: &Vec2) -> f32 {
    unsafe {
        let mut result: f32 = 0.0;
        asm! {
            "%1 = OpExtInstImport \"GLSL.std.450\"",
            "%a = OpLoad _ {a}",
            "%b = OpLoad _ {b}",
            "%result = OpDot typeof*{result} %a %b",
            "OpStore {result} %result",
            a = in(reg) a,
            b = in(reg) b,
            result = in(reg) &mut result,
        }
        result
    }
}

/// Calcualtes the dot product between a and b
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn dot2(a: &Vec2, b: &Vec2) -> f32 {
    a.dot(*b)
}

/// Transforms a vector `point` with matrix `transform`
#[cfg(target_arch = "spirv")]
#[inline]
pub fn transform_vector4(transform: &Mat4, point: &Vec4) -> Vec4 {
    unsafe {
        let mut result = Vec4::ZERO;
        asm! {
            "%float = OpTypeFloat 32",
            "%v4float = OpTypeVector %float 4",
            "%mat4v4float = OpTypeMatrix %v4float 4",
            "%transform_x_axis = OpLoad %v4float {transform_x_axis}",
            "%transform_y_axis = OpLoad %v4float {transform_y_axis}",
            "%transform_z_axis = OpLoad %v4float {transform_z_axis}",
            "%transform_w_axis = OpLoad %v4float {transform_w_axis}",
            "%point = OpLoad %v4float {point}",
            "%transform = OpCompositeConstruct %mat4v4float %transform_x_axis %transform_y_axis %transform_z_axis %transform_w_axis",
            "%result = OpMatrixTimesVector %v4float %transform %point",
            "OpStore {result} %result",
            transform_x_axis = in(reg) &transform.x_axis,
            transform_y_axis = in(reg) &transform.y_axis,
            transform_z_axis = in(reg) &transform.z_axis,
            transform_w_axis = in(reg) &transform.w_axis,
            point = in(reg) point,
            result = in(reg) &mut result,
        }
        result
    }
}

/// Transforms a vector `point` with matrix `transform`
#[cfg(target_arch = "spirv")]
#[inline]
pub fn transform_point3a(transform: &Mat4, point: &Vec3A) -> Vec3A {
    transform_vector4(transform, &point.extend(1.0))
        .truncate()
        .into()
}

/// Transforms a vector `point` with matrix `transform`
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn transform_point3a(transform: &Mat4, point: &Vec3A) -> Vec3A {
    transform.transform_point3a(*point)
}

/// Transforms a vector `point` with matrix `transform`
#[cfg(target_arch = "spirv")]
#[inline]
pub fn transform_vector3a(transform: &Mat4, point: &Vec3A) -> Vec3A {
    transform_vector4(transform, &point.extend(0.0))
        .truncate()
        .into()
}

/// Transforms a vector `point` with matrix `transform`
#[cfg(not(target_arch = "spirv"))]
#[inline]
pub fn transform_vector3a(transform: &Mat4, point: &Vec3A) -> Vec3A {
    transform.transform_point3a(*point)
}
