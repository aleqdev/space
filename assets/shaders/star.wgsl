#import noisy_bevy::prelude
#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> color0: vec4<f32>;

@group(1) @binding(1)
var<uniform> color1: vec4<f32>;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var value =
simplex_noise_3d((world_position.xyz + globals.time / 200000.0) * 32000.0) +
simplex_noise_3d((world_position.xyz + globals.time / 160000.0) * 16000.0) +
simplex_noise_3d((world_position.xyz + globals.time / 120000.0) * 8000.0) +
simplex_noise_3d((world_position.xyz + globals.time / 60000.0) * 4000.0) +
        simplex_noise_3d((world_position.xyz + globals.time / 60000.0) * 2000.0) +
        simplex_noise_3d((world_position.xyz + globals.time / 10000.0) * 1000.0) +
        simplex_noise_3d((world_position.xyz + globals.time / 10000.0) * 500.0) +
        simplex_noise_3d((world_position.xyz + globals.time / 10000.0) * 250.0);

    value /= 7.0;

    return value * color0 + (1.0 - value) * color1;
}
