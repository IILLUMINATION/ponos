struct Uniforms {
    resolution: vec2<f32>,
    thickness: f32,
    smoothing: f32,
    curve_color: vec4<f32>,
    point_count: u32,
};

@group(0) @binding(0) var<uniform> uni: Uniforms;
@group(0) @binding(1) var<storage, read> points: array<vec2<f32>>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    var pos: vec2<f32>;
    switch (vertex_index) {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return output;
}

fn linear_point(p0: vec2<f32>, p1: vec2<f32>, t: f32) -> vec2<f32> {
    return mix(p0, p1, t);
}

fn linear_distance(point: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>) -> f32 {
    let segment = p1 - p0;
    let segment_length = length(segment);
    
    if (segment_length < 0.0001) {
        return length(point - p0);
    }
    
    let segment_dir = segment / segment_length;
    let to_point = point - p0;
    let projection = dot(to_point, segment_dir);
    let t = clamp(projection / segment_length, 0.0, 1.0);
    
    let closest_point = mix(p0, p1, t);
    return length(point - closest_point);
}

fn quadratic_point(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, t: f32) -> vec2<f32> {
    let u = 1.0 - t;
    return u * u * p0 + 2.0 * u * t * p1 + t * t * p2;
}

fn quadratic_derivative(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, t: f32) -> vec2<f32> {
    return 2.0 * (1.0 - t) * (p1 - p0) + 2.0 * t * (p2 - p1);
}

fn quadratic_distance(point: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    var min_distance: f32 = 1000000.0;
    let initial_guesses = 8u;
    
    for (var i: u32 = 0u; i <= initial_guesses; i++) {
        var t = f32(i) / f32(initial_guesses);
        
        for (var iter: u32 = 0u; iter < 3u; iter++) {
            let curve_point = quadratic_point(p0, p1, p2, t);
            let derivative = quadratic_derivative(p0, p1, p2, t);
            let diff = curve_point - point;
            
            let f = dot(diff, derivative);
            let second_deriv = 2.0 * (p2 - 2.0 * p1 + p0);
            let df = dot(derivative, derivative) + dot(diff, second_deriv);
            
            if (abs(df) > 0.00000001) {
                t = clamp(t - f / df, 0.0, 1.0);
            }
        }
        
        let closest_point = quadratic_point(p0, p1, p2, t);
        let dist = length(closest_point - point);
        min_distance = min(min_distance, dist);
    }
    
    return min_distance;
}

fn cubic_point(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    let u = 1.0 - t;
    let uu = u * u;
    let uuu = uu * u;
    let tt = t * t;
    let ttt = tt * t;
    
    return uuu * p0 + 
           3.0 * uu * t * p1 + 
           3.0 * u * tt * p2 + 
           ttt * p3;
}

fn cubic_derivative(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    let u = 1.0 - t;
    return 3.0 * u * u * (p1 - p0) + 
           6.0 * u * t * (p2 - p1) + 
           3.0 * t * t * (p3 - p2);
}

fn cubic_second_derivative(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    return 6.0 * (1.0 - t) * (p2 - 2.0 * p1 + p0) + 
           6.0 * t * (p3 - 2.0 * p2 + p1);
}

fn cubic_distance(point: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> f32 {
    var min_distance: f32 = 1000000.0;
    let initial_guesses = 8u;
    
    for (var i: u32 = 0u; i <= initial_guesses; i++) {
        var t = f32(i) / f32(initial_guesses);
        
        for (var iter: u32 = 0u; iter < 3u; iter++) {
            let curve_point = cubic_point(p0, p1, p2, p3, t);
            let derivative = cubic_derivative(p0, p1, p2, p3, t);
            let diff = curve_point - point;
            
            let f = dot(diff, derivative);
            let second_deriv = cubic_second_derivative(p0, p1, p2, p3, t);
            let df = dot(derivative, derivative) + dot(diff, second_deriv);
            
            if (abs(df) > 0.00000001) {
                t = clamp(t - f / df, 0.0, 1.0);
            }
        }
        
        let closest_point = cubic_point(p0, p1, p2, p3, t);
        let dist = length(closest_point - point);
        min_distance = min(min_distance, dist);
    }
    
    return min_distance;
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let frag_coord = uv * uni.resolution;
    
    if (uni.point_count < 2u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    var min_dist: f32 = 1000000.0;
    
    for (var i: u32 = 0u; i < uni.point_count - 1u; i++) {
        let p0 = points[i];
        let p1 = points[i + 1u];
        
        let linear_dist = linear_distance(frag_coord, p0, p1);
        min_dist = min(min_dist, linear_dist);
        
        if (i + 3u < uni.point_count) {
            let p2 = points[i + 2u];
            let quadratic_dist = quadratic_distance(frag_coord, p0, p1, p2);
            min_dist = min(min_dist, quadratic_dist);
            
            if (i + 4u < uni.point_count) {
                let p3 = points[i + 3u];
                let cubic_dist = cubic_distance(frag_coord, p0, p1, p2, p3);
                min_dist = min(min_dist, cubic_dist);
            }
        }
    }
    
    let alpha = 1.0 - smoothstep(uni.thickness - uni.smoothing, uni.thickness + uni.smoothing, min_dist);
    return vec4<f32>(uni.curve_color.rgb, uni.curve_color.a * alpha);
}