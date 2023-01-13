fn shlick(direction: vec3<f32>, normal: vec3<f32>, n1: f32, n2: f32) -> f32 {
    let dot = dot(direction, normal);
    let r = (n1 - n2) / (n1 + n2);
    let r2 = r*r;
    return r2 + (1.0 - r2) * pow(1.0 + dot, 5.0);
}

struct AABB {
    min: vec3<f32>;
    max: vec3<f32>;
};

struct SceneArgs {
    rects_bounding_box: AABB;
    spheres_bounding_box: AABB;
};

struct Camera {
    transform: mat4x4<f32>;
    screen_size: vec2<f32>;
    tan_fov: f32;
    t_min: f32;
    t_max: f32;
};

struct Background {
    color: vec3<f32>;
};

struct RaytracerArgs {
    camera: Camera;
    background: Background;
    bounces: u32;
};

struct Args {
    raytracer_args: RaytracerArgs;
    scene_args: SceneArgs;
};

[[group(0), binding(0)]]
var<storage, read> args: Args;

struct Sphere {
    position: vec3<f32>;
    _pad0: f32;
    color: vec3<f32>;
    _pad1: f32;
    radius: f32;
    n: f32;
};

struct Spheres {
    spheres: array<Sphere>;
};

[[group(0), binding(1)]]
var<storage, read> spheres: Spheres;

struct Rect {
    transform: mat4x4<f32>;
    color: vec3<f32>;
};

struct Rects {
    rects: array<Rect>;
};

[[group(0), binding(2)]]
var<storage, read> rects: Rects;

struct PointLight {
    position: vec3<f32>;
    color: vec3<f32>;
};

struct PointLights {
    point_lights: array<PointLight>;
};

[[group(0), binding(3)]]
var<storage, read> point_lights: PointLights;

struct Ray {
    origin: vec3<f32>;
    t_min: f32;
    direction: vec3<f32>;
    t_max: f32;
};

fn valid_t(ray: Ray, t: f32) -> bool {
    return ray.t_min < t && ray.t_max > t;
}

fn point_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}

fn transform_ray(ray: Ray, transform: mat4x4<f32>) -> Ray {
    var result: Ray;

    result.origin = (transform * vec4<f32>(ray.origin, 1.0)).xyz;
    result.direction = (transform * vec4<f32>(ray.direction, 0.0)).xyz;
    result.t_min = ray.t_min;
    result.t_max = ray.t_max;
    
    return result;
}

struct SphereIntersection {
    a: f32;
    b: f32;
    discriminant: f32;
};

fn sphere_intersect(ray: Ray, sphere: Sphere, intersection: ptr<function, f32>) -> bool {
    let oc = ray.origin - sphere.position;
    let radius = sphere.radius;
    let direction = ray.direction;

    let a = dot(direction, direction);
    let b = 2.0 * dot(oc, direction);
    let c = dot(oc,oc) - radius*radius;
    let discriminant = b*b - 4.0*a*c;
    
    if(discriminant >= 0.0) {
        let t = (-b - sqrt(discriminant)) / (2.0*a);

        *intersection = t;

        return valid_t(ray, t);
    }

    return false;
}

struct SpheresIntersection {
    nearest_intersection_result: f32;
    nearest_intersected_sphere: u32;
};

fn intersect_spheres(ray: Ray, spheres_intersection: ptr<function, SpheresIntersection>) -> bool {
    let sphere_count = arrayLength(&spheres.spheres);

    var nearest_intersection_result: f32 = ray.t_max;
    var nearest_intersected_sphere: u32 = sphere_count;

    for(var i: u32 = 0u; i < sphere_count; i = i + 1u) {
        var t: f32;

        if(sphere_intersect(ray, spheres.spheres[i], &t)) {
            if(nearest_intersection_result > t) {
                nearest_intersection_result = t;
                nearest_intersected_sphere = i;
            }
        }
    }

    (*spheres_intersection).nearest_intersection_result = nearest_intersection_result;
    (*spheres_intersection).nearest_intersected_sphere = nearest_intersected_sphere;

    return nearest_intersected_sphere != sphere_count;
}

struct ShadingResult {
    reflection_ray: Ray;
    reflective_color: vec3<f32>;
    emissive_color: vec3<f32>;
    reflection: bool;
};

fn shadow(ray: Ray) -> bool {
    var spheres_intersection: SpheresIntersection;

    return intersect_spheres(ray, &spheres_intersection);
}

fn lambert_point_light(point_light: PointLight, position: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let light_dir = point_light.position - position;

    var ray: Ray;

    ray.direction = light_dir;
    ray.origin = position;
    ray.t_max = 1.0;
    ray.t_min = 0.001;

    return select(max(dot(normalize(light_dir), normal), 0.0) / dot(light_dir, light_dir), 0.0, shadow(ray)) * point_light.color;
}

fn lambert(position: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let point_light_count = arrayLength(&point_lights.point_lights);

    var result = vec3<f32>(0.0);

    for(var i: u32 = 0u; i < point_light_count; i = i + 1u) {
        result = result + lambert_point_light(point_lights.point_lights[i], position, normal);
    }

    return result;
}

fn sphere_sdf(sphere: Sphere, position: vec3<f32>) -> f32 {
    return distance(sphere.position, position) - sphere.radius;
}

fn sdf(position: vec3<f32>) -> f32 {
    let spheres_count = arrayLength(&spheres.spheres);

    var min_distance = 1000.0;

    for(var i: u32 = 0u; i < spheres_count; i = i + 1u) {
        min_distance = min(min_distance, sphere_sdf(spheres.spheres[i], position));
    }

    return min_distance;
}

fn ambient_occlusion(position: vec3<f32>, normal: vec3<f32>) -> f32 {
    var occlusion = 1.0;
    for(var i: u32 = 1u; i < 6u; i = i + 1u) {
        let sample = f32(i);
        occlusion = occlusion - ((sample * 0.35 - sdf(position + normal * (sample * 0.35))) / pow(2.0, sample));
    }
    return occlusion;
}

fn shade_sphere(sphere: Sphere, ray: Ray, t: f32) -> ShadingResult {
    var shading_result: ShadingResult;

    var reflection_ray: Ray;

    let position = point_at(ray, t);
    let normal = normalize(position - sphere.position);

    reflection_ray.origin = position;
    reflection_ray.direction = reflect(ray.direction, normal);
    reflection_ray.t_min = 0.001;
    reflection_ray.t_max = 1000.0;

    let shlick = shlick(ray.direction, normal, 1.0, sphere.n);

    //shading_result.emissive_color = vec3<f32>(0.0);
    shading_result.emissive_color = (1.0 - shlick) * sphere.color * (ambient_occlusion(position, normal) + lambert(position, normal));
    shading_result.reflection = true;
    shading_result.reflection_ray = reflection_ray;
    shading_result.reflective_color = vec3<f32>(shlick);
    //shading_result.reflective_color = vec3<f32>(0.5);

    return shading_result;
}

fn rect_intersect(ray: Ray, rect: Rect, intersection: ptr<function, f32>) -> bool {
    let ray = transform_ray(ray, rect.transform);
    
    let dot = ray.direction.y * 1.0;

    let t = ((-ray.origin.y) * 1.0) / dot;
    let position = point_at(ray, t);

    *intersection = t;

    let axis_valid = abs(position.xz);

    return valid_t(ray, t) && axis_valid.x < 0.5 && axis_valid.y < 0.5;
}

struct RectsIntersection {
    nearest_intersection_result: f32;
    nearest_intersected_rect: u32;
};

fn intersect_rects(ray: Ray, rects_intersection: ptr<function, RectsIntersection>) -> bool {
    let rect_count = arrayLength(&rects.rects);

    var nearest_intersection_result: f32 = ray.t_max; 
    var nearest_intersected_rect: u32 = rect_count;

    for(var i: u32 = 0u; i < rect_count; i = i + 1u) {
        var t: f32;

        if(rect_intersect(ray, rects.rects[i], &t)) {
            if(nearest_intersection_result > t) {
                nearest_intersection_result = t;
                nearest_intersected_rect = i;
            }
        }
    }

    (*rects_intersection).nearest_intersection_result = nearest_intersection_result;
    (*rects_intersection).nearest_intersected_rect = nearest_intersected_rect;

    return nearest_intersected_rect != rect_count;
}

fn rect_shade(rect: Rect) -> ShadingResult {
    var shading_result: ShadingResult;

    shading_result.emissive_color = rect.color;
    shading_result.reflection = false;

    return shading_result;
}

let RAY_BOUNCES: u32 = 5u;

fn radiance(ray: Ray) -> vec3<f32> {
    var ray = ray;
    var reflective_color = vec3<f32>(1.0);
    var radiance = vec3<f32>(0.0);

    for(var i: u32 = 0u; i < RAY_BOUNCES; i = i + 1u) {
        var spheres_intersection: SpheresIntersection;

        let is_sphere_intersected = intersect_spheres(ray, &spheres_intersection);

        var rects_intersection: RectsIntersection;

        let is_rect_intersected = intersect_rects(ray, &rects_intersection);

        var shading_result: ShadingResult;

        shading_result.emissive_color = args.raytracer_args.background.color;
        shading_result.reflection = false;

        if(is_sphere_intersected && spheres_intersection.nearest_intersection_result < rects_intersection.nearest_intersection_result) {
            shading_result = shade_sphere(spheres.spheres[spheres_intersection.nearest_intersected_sphere], ray, spheres_intersection.nearest_intersection_result);
        }

        if(is_rect_intersected && rects_intersection.nearest_intersection_result < spheres_intersection.nearest_intersection_result) {
            shading_result = rect_shade(rects.rects[rects_intersection.nearest_intersected_rect]);
        }
        
        if(shading_result.reflection) { 
            ray = shading_result.reflection_ray;
            radiance = radiance + reflective_color * shading_result.emissive_color;
            reflective_color = reflective_color * shading_result.reflective_color;
        } else {
            radiance = radiance + reflective_color * shading_result.emissive_color;
            break;
        }
    }

    return radiance;
}

fn prime_ray(camera: Camera, sample: vec2<f32>) -> Ray {
    var ray: Ray;

    let sensor = (sample / camera.screen_size * 2.0 - vec2<f32>(1.0))
            * camera.tan_fov
            * vec2<f32>(1.0, -(camera.screen_size.y / camera.screen_size.x));

    ray.origin = vec3<f32>(0.0);
    ray.direction = normalize(vec3<f32>(sensor, 1.0));
    ray.t_min = camera.t_min;
    ray.t_max = camera.t_max;

    var ray = transform_ray(ray, camera.transform);

    ray.direction = normalize(ray.direction);

    return ray;
}

fn tonemapFilmic(x: vec3<f32>) -> vec3<f32> {
    let X: vec3<f32> = max(vec3<f32>(0.0), x - 0.004);
    let result: vec3<f32> = (X * (6.2 * X + 0.5)) / (X * (6.2 * X + 1.7) + 0.06);
    return pow(result, vec3<f32>(2.2));
}

[[stage(vertex)]]
fn vertex([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = f32(vertex_index & 1u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) - 1.0;

    let position = vec4<f32>(x, y, 0.0, 1.0);

    return position;
}

[[stage(fragment)]]
fn fragment([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let prime_ray = prime_ray(args.raytracer_args.camera, position.xy);

    return vec4<f32>(tonemapFilmic(radiance(prime_ray)), 1.0);
}