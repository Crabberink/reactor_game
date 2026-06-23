#import bevy_pbr::forward_io::VertexOutput;

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var atlas: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var atlas_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	let color = textureSample(atlas, atlas_sampler, in.uv);

	return color;
}