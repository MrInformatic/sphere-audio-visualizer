struct Sphere {
    position: vec2<f32>;
    radius: f32;
};

struct Args { 
    color: vec3<f32>;
    size: vec2<f32>;
    zoom: f32;
};

[[group(0), binding(0)]]
var<storage, read> args: Args;

struct Spheres {
    spheres: array<Sphere>;
};

[[group(0), binding(1)]]
var<storage, read> spheres: Spheres;

[[stage(vertex)]]
fn vertex([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = f32(vertex_index & 1u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) - 1.0;

    let position = vec4<f32>(x, y, 0.0, 1.0);

    return position;
}

[[stage(fragment)]]
fn fragment([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var value = 0.0;

    let position = (position.xy / args.size * 2.0 - 1.0) * args.zoom;

    let count = arrayLength(&spheres.spheres);
    for(var i: u32 = 0u; i < count; i = i + 1u) {
        let oc = position - spheres.spheres[i].position;
        let radius = spheres.spheres[i].radius;

        value = value + inverseSqrt(dot(oc, oc)) * radius * 0.05;
    }

    return select(vec4<f32>(args.color * value, 1.0), vec4<f32>(1.0, 1.0, 1.0, 1.0), value >= 0.75);
}