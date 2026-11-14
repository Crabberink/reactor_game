import bevy_pbr::render::forward_io::VertexOutput;

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var atlas_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(atlas, atlas_sampler, in.uv);

    return color;
}