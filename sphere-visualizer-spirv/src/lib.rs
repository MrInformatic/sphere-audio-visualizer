#![cfg_attr(target_arch = "spirv", no_std)]
#![warn(missing_docs)]

//! This crate exposes the logic implemnted in the `sphere-visualizer-core` to
//! to the gpu as a shader using rust-gpu
//! <https://github.com/EmbarkStudios/rust-gpu>

use sphere_visualizer_core::{
    metaballs::{Metaball, Metaballs, MetaballsArgs},
    raytracing::{
        light::{LightGroup, LightScene, PointLight},
        shape::{Rect, Scene, Sphere},
        BasicRaytracingArgsBundle, Raytracer,
    },
};
use spirv_std::glam::{vec4, Vec4, Vec4Swizzles};
use spirv_std::spirv;

/// This function contains the fragment shader implemntation for the metaballs
/// renderer.
#[spirv(fragment)]
pub fn metaballs_fs(
    #[spirv(frag_coord)] position: Vec4,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] args: &MetaballsArgs,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] metaballs: &[Metaball],
    output: &mut Vec4,
) {
    let metaballs = Metaballs::from_args(args.clone(), metaballs);

    *output = metaballs.sample(&position.xy()).extend(1.0);
}

/// This function contains the vertex shader implemntation for the metaballs
/// renderer.
#[spirv(vertex)]
pub fn metaballs_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position, invariant)] position: &mut Vec4,
) {
    let x = (vertex_index & 1) as f32 * 2.0 - 1.0;
    let y = (vertex_index & 2) as f32 - 1.0;

    *position = vec4(x, y, 0.0, 1.0);
}

/// This function contains the fragment shader implemntation for the raytracing
/// renderer.
#[spirv(fragment)]
pub fn raytracing_fs(
    #[spirv(frag_coord)] position: Vec4,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] args: &BasicRaytracingArgsBundle,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] spheres: &[Sphere],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] rects: &[Rect],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] point_lights: &[PointLight],
    output: &mut Vec4,
) {
    let scene = Scene::from_args(args.scene_args.clone(), spheres, rects);

    let light_scene = LightScene {
        point_lights: LightGroup(point_lights),
    };

    let raytracer = Raytracer::from_args(args.raytracer_args.clone(), scene, light_scene);

    *output = raytracer.sample(&position.xy()).extend(1.0);
}

/// This function contains the vertex shader implemntation for the raytracing
/// renderer.
#[spirv(vertex)]
pub fn raytracing_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position, invariant)] position: &mut Vec4,
) {
    let x = (vertex_index & 1) as f32 * 2.0 - 1.0;
    let y = (vertex_index & 2) as f32 - 1.0;

    *position = vec4(x, y, 0.0, 1.0);
}
