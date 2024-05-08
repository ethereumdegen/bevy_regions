 
//see bindings in terrain_material.rs 
 
 
 #import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
      pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
  
struct StandardMaterial {
    time: f32,
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
};

 



struct RegionColorArray {
    vectors: array<vec4<f32>, 32>, //32 max regions 
}


struct ToolPreviewUniforms { 
    tool_coordinates: vec2<f32>,
    tool_radius: f32,
    tool_color: vec3<f32>    
};

//https://github.com/DGriffin91/bevy_mod_standard_material/blob/main/assets/shaders/pbr.wgsl


@group(1) @binding(1)
var base_color_texture1: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler1: sampler;
 

@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;

@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;

 

@group(2) @binding(21)
var<uniform> tool_preview_uniforms: ToolPreviewUniforms;
 
  

@group(2) @binding(22)
var regions_map_texture: texture_2d<u32>;

//@group(2) @binding(23)
//var regions_map_sampler: sampler;

 

@fragment
fn fragment(
    mesh: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    
     
   // let region_texture_value = textureLoad(regions_map_texture, vec2<i32>( mesh.uv  ), 0).r;
    
    // read from color array
    let prelighting_color = vec4<f32>(f32( 22 ) / 255.0, 0.0, 0.0, 1.0);
    
   
  // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(mesh, is_front);
 
    //hack the material (StandardMaterialUniform)  so the color is from the terrain splat 
    pbr_input.material.base_color =  prelighting_color;


    var pbr_out: FragmentOutput;
 
    
    // apply lighting
    pbr_out.color = apply_pbr_lighting(pbr_input);
    // we can optionally modify the lit color before post-processing is applied
    // out.color = out.color;
    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    pbr_out.color = main_pass_post_lighting_processing(pbr_input, pbr_out.color);



    // -----

   // let shadowFactor = calculate_shadow_factor(frag_lightSpacePos);


   
    let vertex_world_psn = mesh.world_position.xz; // Assuming the vertex position is in world space

    let tool_coordinates = tool_preview_uniforms.tool_coordinates;
    let tool_radius = tool_preview_uniforms.tool_radius;
    let color_from_tool = tool_preview_uniforms.tool_color;

    let distance = length(vertex_world_psn - tool_coordinates);

    let within_tool_radius = f32(distance <= tool_radius);

    let alpha_value = 0.5;

    let final_color = mix(
        vec4(pbr_out.color.rgb,  alpha_value),
        vec4(pbr_out.color.rgb * color_from_tool, alpha_value),
        within_tool_radius
    );
          

   
    
    return final_color;
    
}
 