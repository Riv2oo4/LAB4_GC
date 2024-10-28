use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use fastnoise_lite::FastNoiseLite;



pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  let mut position = Vec4::new(
      vertex.position.x,
      vertex.position.y,
      vertex.position.z,
      1.0,
  );

  let wobble = (uniforms.time as f32 * 0.02).sin() * 0.05;
  position.x += wobble * vertex.position.y;
  position.y += wobble * vertex.position.z;

  let transformed = uniforms.projection_matrix
      * uniforms.view_matrix
      * uniforms.model_matrix
      * position;

  let w = transformed.w;
  let transformed_position = Vec4::new(
      transformed.x / w,
      transformed.y / w,
      transformed.z / w,
      1.0,
  );

  let screen_position = uniforms.viewport_matrix * transformed_position;

  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
  let transformed_normal = normal_matrix * vertex.normal;

  Vertex {
      position: vertex.position,
      normal: vertex.normal,
      tex_coords: vertex.tex_coords,
      color: vertex.color,
      transformed_position: Vec3::new(
          screen_position.x,
          screen_position.y,
          screen_position.z,
      ),
      transformed_normal,
  }
}


pub fn sun_shader(uniforms: &Uniforms) -> Color {
  let pulsate = ((uniforms.time as f32 * 0.01).sin() + 1.0) / 2.0;

  let surface_noise = uniforms.noise.get_noise_2d(
      uniforms.time as f32 * 0.1,
      uniforms.time as f32 * 0.1,
  );

  let eruption_noise = uniforms.noise.get_noise_2d(
      uniforms.time as f32 * 0.02,
      (uniforms.time as f32 * 0.02).cos(),
  );

  let core_color = Color::new(255, 140, 0);    
  let flare_color = Color::new(255, 69, 0);    
  let corona_color = Color::new(255, 255, 160); 

  let core = core_color.lerp(&flare_color, surface_noise);

  let corona_intensity = (uniforms.time as f32 * 0.005).cos().abs();
  let corona = corona_color * corona_intensity;

  let flare_intensity = if eruption_noise > 0.8 {
      1.5 
  } else {
      1.0 
  };

  let final_color = (core + corona) * pulsate * flare_intensity;

  let halo_color = Color::new(255, 215, 0); 
  let halo_intensity = ((uniforms.time as f32 * 0.002).sin().abs() * 0.5).clamp(0.0, 1.0);

  final_color + halo_color * halo_intensity
}
pub fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let ocean_color = Color::new(0, 102, 204);  
  let land_color = Color::new(34, 139, 34);   
  let desert_color = Color::new(210, 180, 140);  
  let mountain_color = Color::new(139, 137, 137);  
  let cloud_color = Color::new(255, 255, 255);  
  let ice_color = Color::new(240, 248, 255);  

  let t_clouds = uniforms.time as f32 * 0.02;
  let t_surface = uniforms.time as f32 * 0.005;

  let terrain_noise = fbm_noise(
      &uniforms.noise,
      fragment.vertex_position.x * 15.0,
      fragment.vertex_position.y * 15.0,
      6,  
  );

  let biome_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 50.0 + t_surface,
      fragment.vertex_position.y * 50.0 + t_surface,
  );

  let cloud_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 100.0 + t_clouds,
      fragment.vertex_position.y * 100.0 + t_clouds,
  );

  let biome_color = if biome_noise > 0.7 {
      mountain_color.lerp(&land_color, terrain_noise)  
  } else if biome_noise > 0.5 {
      desert_color * (1.0 - terrain_noise).clamp(0.5, 1.0) 
  } else {
      land_color * terrain_noise  
  };

  let surface_color = if biome_noise < 0.3 {
      ocean_color.lerp(&biome_color, biome_noise / 0.3)  
  } else {
      biome_color  
  };

  let final_color = if cloud_noise > 0.8 {
      cloud_color  
  } else if cloud_noise > 0.6 {
      ice_color.lerp(&surface_color, 0.5)  
  } else {
      surface_color  
  };

  let light_factor = 0.6 + 0.4 * terrain_noise * fragment.vertex_position.z.clamp(-1.0, 1.0);

  let illuminated_color = final_color * light_factor;
  illuminated_color * fragment.intensity
}


pub fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let sand_color = Color::new(210, 77, 38);    
  let rock_color = Color::new(150, 75, 45);    
  let crater_color = Color::new(105, 54, 36);  
  let ridge_color = Color::new(130, 60, 35);   
  let t = uniforms.time as f32 * 0.3;
  let spherical_x = fragment.vertex_position.x / fragment.vertex_position.z.abs().max(0.1);
  let spherical_y = fragment.vertex_position.y / fragment.vertex_position.z.abs().max(0.1);
  let fine_noise = fbm_noise(
      &uniforms.noise,
      spherical_x * 60.0 + t,
      spherical_y * 60.0 + t,
      5,
  );

  let crater_noise = uniforms.noise.get_noise_3d(
      spherical_x * 40.0 + t * 0.5,
      spherical_y * 40.0 + t * 0.5,
      fragment.vertex_position.z * 40.0,
  );

  let rock_noise = fbm_noise(
      &uniforms.noise,
      spherical_x * 20.0,
      spherical_y * 20.0,
      6,
  );

  let rock_mask = uniforms.noise.get_noise_2d(
      spherical_x * 25.0,
      spherical_y * 25.0,
  );
  let small_rock_factor = (rock_mask * rock_noise).clamp(0.0, 1.0);

  let surface_color = if crater_noise > 0.75 {
      crater_color * (1.0 - crater_noise).clamp(0.5, 1.0) * fragment.intensity
  } else if small_rock_factor > 0.6 {
      rock_color * small_rock_factor * fragment.intensity
  } else {
      sand_color.lerp(&ridge_color, fine_noise) * fragment.intensity
  };

  let light_factor = 0.5 + 0.5 * fragment.vertex_position.z.clamp(-1.0, 1.0);
  let final_color = surface_color * light_factor;

  final_color
}



fn fbm_noise(noise: &FastNoiseLite, x: f32, y: f32, octaves: usize) -> f32 {
  let mut value = 0.0;
  let mut amplitude = 1.0;
  let  frequency = 1.0;

  for i in 0..octaves {
 
      let offset = i as f32 * 0.1;
      value += noise.get_noise_2d(x * frequency + offset, y * frequency + offset) * amplitude;
      amplitude *= 0.6;  
  }

  value
}


pub fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {

  let band_yellow = Color::new(255, 239, 170); 
  let band_beige = Color::new(230, 220, 170);  
  let band_brown = Color::new(180, 120, 70);   
  let band_dark_brown = Color::new(120, 70, 40); 

  let storm_color = Color::new(255, 69, 0);  

  let t = uniforms.time as f32 * 0.02; 

  let y_position = fragment.vertex_position.y * 15.0;

  let wave_pattern = (y_position + (t * 2.0).sin()).sin(); 
  let fbm_value = fbm_noise(
      &uniforms.noise,
      fragment.vertex_position.x * 1.5 + t * 0.05,
      fragment.vertex_position.y * 3.0,
      6,
  );

  let turbulence = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 10.0 + t * 0.3,
      fragment.vertex_position.y * 10.0,
  );

  let wave_intensity = ((wave_pattern + fbm_value * 0.5) * 0.5 + 0.5) * (1.0 + turbulence * 0.2);

  let band_color = band_yellow
      .lerp(&band_beige, wave_intensity * 0.5) 
      .lerp(&band_brown, wave_intensity * 0.8) 
      .lerp(&band_dark_brown, wave_intensity); 

  let red_spot_dist = ((fragment.vertex_position.x + 0.2).powi(2)
      + (fragment.vertex_position.y - 0.2).powi(2))
      .sqrt();
  let red_spot_intensity = (1.0 - red_spot_dist * 4.0).clamp(0.0, 1.0);

  let final_color = if red_spot_intensity > 0.7 {
      storm_color * red_spot_intensity 
  } else {
      band_color * fragment.intensity 
  };

  final_color * (1.0 + 0.15 * turbulence).clamp(0.0, 1.2)
}


pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_gray = Color::new(180, 180, 180);  
  let crater_edge_color = Color::new(120, 120, 120);  
  let crater_center_color = Color::new(80, 80, 80);  

  let t = uniforms.time as f32 * 0.1;  

  let spherical_x = fragment.vertex_position.x / fragment.vertex_position.z.abs().max(0.1);
  let spherical_y = fragment.vertex_position.y / fragment.vertex_position.z.abs().max(0.1);
  let crater_noise = fbm_noise(&uniforms.noise, spherical_x * 30.0 + t, spherical_y * 30.0, 4);
  let mask_noise = fbm_noise(&uniforms.noise, spherical_x * 60.0, spherical_y * 60.0, 5);
  let depth_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 300.0,
      fragment.vertex_position.y * 300.0,
  );
  let crater_effect = if crater_noise > 0.55 && mask_noise > 0.3 {
      crater_center_color.lerp(&crater_edge_color, depth_noise)
  } else {
      base_gray
  };
  let light_factor = 0.5 + 0.5 * fragment.vertex_position.z.clamp(-1.0, 1.0);
  let illuminated_color = crater_effect * light_factor;
  illuminated_color * fragment.intensity
}



pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let band_color1 = Color::new(210, 180, 140);  
  let band_color2 = Color::new(170, 140, 110);  
  let band_color3 = Color::new(240, 230, 140);  
  let band_color4 = Color::new(200, 160, 100);  

  let ring_color1 = Color::new(192, 192, 192);  
  let ring_color2 = Color::new(169, 169, 169);  
  let ring_color3 = Color::new(220, 220, 220);  


  let radius = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.z.powi(2)).sqrt();
  let angle = fragment.vertex_position.z.atan2(fragment.vertex_position.x);


  let ring_noise = fbm_noise(
      &uniforms.noise,
      radius * 10.0,
      angle * 5.0 + uniforms.time as f32 * 0.05,  
      4, 
  );

  let wave_pattern = (radius * 6.0 + (uniforms.time as f32 * 0.1).sin()).sin();

  let ring_color = if ring_noise > 0.7 {
      ring_color1.lerp(&ring_color2, wave_pattern * 0.5 + 0.5)
  } else if ring_noise > 0.4 {
      ring_color2.lerp(&ring_color3, wave_pattern * 0.5 + 0.5)
  } else {
      ring_color3 * (0.8 + 0.2 * ring_noise)  
  };


  let pos = fragment.vertex_position.normalize();
  let latitude = pos.y;

  let band_noise = fbm_noise(
      &uniforms.noise,
      latitude * 25.0 + uniforms.time as f32 * 0.01, 
      0.0,
      6,  
  );

  let band_color = if band_noise > 0.75 {
      band_color1
  } else if band_noise > 0.5 {
      band_color2
  } else if band_noise > 0.25 {
      band_color3
  } else {
      band_color4
  };

  let ring_opacity = ((2.5 - radius).clamp(0.0, 1.0) * 0.8).max(0.1);

  let final_color = if radius > 1.0 && radius < 2.5 {
      ring_color * fragment.intensity * ring_opacity
  } else {
      band_color * fragment.intensity
  };

  final_color
}

pub fn comet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let core_color = Color::new(255, 105, 180);    
  let surface_color = Color::new(72, 61, 139);   
  let crack_color = Color::new(50, 205, 50);     

  let tail_inner_color = Color::new(0, 255, 255);  
  let tail_outer_color = Color::new(255, 69, 0);   

  let t = uniforms.time as f32 * 0.05;

  let pulsate = (t.sin() * 0.5 + 0.5).clamp(0.3, 1.0); 

  let surface_noise = fbm_noise(
      &uniforms.noise,
      fragment.vertex_position.x * 8.0,
      fragment.vertex_position.y * 8.0,
      4,  
  );


  let crack_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 50.0 + t,
      fragment.vertex_position.y * 50.0 + t,
  );

  let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();

  let tail_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 15.0 + t,
      fragment.vertex_position.y * 15.0 + t,
  );
  let tail_color = tail_inner_color.lerp(&tail_outer_color, tail_noise);
  let tail_intensity = (1.0 - distance / 5.0).clamp(0.0, 1.0) * tail_noise;
  let surface_effect = if crack_noise > 0.6 {
      crack_color.lerp(&surface_color, surface_noise) * (1.0 - crack_noise).clamp(0.5, 1.0)
  } else {
      core_color * pulsate * (0.7 + surface_noise * 0.3)  
  };
  let light_factor = 0.5 + 0.5 * fragment.vertex_position.z.clamp(-1.0, 1.0);
  let illuminated_surface = surface_effect * light_factor;
  let final_color = if distance < 1.0 {
      illuminated_surface * fragment.intensity  
  } else {
      tail_color * tail_intensity * fragment.intensity  
  };

  final_color
}
