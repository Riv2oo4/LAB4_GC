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

  // Añadimos una pequeña distorsión (animación) a los vértices en función del tiempo.
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

/// Fragment shader principal que selecciona los shaders específicos según el planeta.
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Aquí se seleccionan los shaders según el contexto del cuerpo celeste.
    earth_shader(fragment, uniforms)  // Por ejemplo, llama al shader de la Tierra
}

pub fn sun_shader(uniforms: &Uniforms) -> Color {
  // Brillo oscilante (pulso del Sol)
  let pulsate = ((uniforms.time as f32 * 0.01).sin() + 1.0) / 2.0;

  // Gradiente turbulento en la superficie solar
  let surface_noise = uniforms.noise.get_noise_2d(
      uniforms.time as f32 * 0.1,
      uniforms.time as f32 * 0.1,
  );

  // Efecto para las erupciones solares
  let eruption_noise = uniforms.noise.get_noise_2d(
      uniforms.time as f32 * 0.02,
      (uniforms.time as f32 * 0.02).cos(),
  );

  // Colores para el núcleo y las capas externas
  let core_color = Color::new(255, 140, 0);    // Naranja del núcleo
  let flare_color = Color::new(255, 69, 0);    // Rojo intenso para erupciones
  let corona_color = Color::new(255, 255, 160); // Amarillo suave para la corona

  // Interpolación del núcleo con erupciones solares
  let core = core_color.lerp(&flare_color, surface_noise);

  // Intensidad de la corona pulsante
  let corona_intensity = (uniforms.time as f32 * 0.005).cos().abs();
  let corona = corona_color * corona_intensity;

  // Efecto de erupción: destellos aleatorios que se activan de vez en cuando
  let flare_intensity = if eruption_noise > 0.8 {
      1.5 // Erupción fuerte
  } else {
      1.0 // Estado normal
  };

  // Combinación del núcleo, corona y destellos solares
  let final_color = (core + corona) * pulsate * flare_intensity;

  // Simulación del halo exterior con emisión suave
  let halo_color = Color::new(255, 215, 0); // Amarillo dorado para el halo
  let halo_intensity = ((uniforms.time as f32 * 0.002).sin().abs() * 0.5).clamp(0.0, 1.0);

  // Color combinado con el halo
  final_color + halo_color * halo_intensity
}

pub fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Definimos colores para diferentes biomas y elementos
  let ocean_color = Color::new(0, 102, 204);  // Azul del océano
  let land_color = Color::new(34, 139, 34);   // Verde para tierra
  let desert_color = Color::new(210, 180, 140);  // Arena del desierto
  let mountain_color = Color::new(139, 137, 137);  // Gris para montañas
  let cloud_color = Color::new(255, 255, 255);  // Nubes
  let ice_color = Color::new(240, 248, 255);  // Ártico/Antártico

  // Variables de tiempo para animación independiente de nubes y océanos
  let t_clouds = uniforms.time as f32 * 0.02;
  let t_surface = uniforms.time as f32 * 0.005;

  // Ruido para biomas dinámicos
  let biome_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 50.0 + t_surface,
      fragment.vertex_position.y * 50.0 + t_surface,
  );

  // Ruido independiente para las nubes
  let cloud_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 100.0 + t_clouds,
      fragment.vertex_position.y * 100.0 + t_clouds,
  );

  // Determinamos el color del bioma según el valor del ruido
  let biome_color = if biome_noise > 0.7 {
      mountain_color
  } else if biome_noise > 0.5 {
      desert_color
  } else {
      land_color
  };

  // Interpolación para mezcla suave entre agua y tierra
  let surface_color = if biome_noise < 0.3 {
      ocean_color.lerp(&biome_color, biome_noise / 0.3)
  } else {
      biome_color
  };

  // Aplicamos nubes dinámicas encima de la superficie
  let final_color = if cloud_noise > 0.8 {
      cloud_color
  } else if cloud_noise > 0.6 {
      ice_color.lerp(&surface_color, 0.5)  // Mezcla con hielo en áreas frías
  } else {
      surface_color
  };

  // Efecto de iluminación: Gradiente para día/noche según la posición Z
  let light_factor = 0.5 + 0.5 * fragment.vertex_position.z.clamp(-1.0, 1.0);
  let illuminated_color = final_color * light_factor;

  illuminated_color * fragment.intensity  // Ajuste final según la intensidad del fragmento
}



pub fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores característicos de Marte
  let sand_color = Color::new(210, 77, 38);   // Arena rojiza
  let rock_color = Color::new(139, 69, 19);   // Rocas oscuras
  let crater_color = Color::new(105, 54, 36); // Interior del cráter oscuro

  // Tiempo para animaciones suaves
  let t = uniforms.time as f32 * 0.01;

  // **Ruido para la distribución de cráteres**
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 50.0 + t,  // Coordenadas ajustadas para ruido más grueso
      fragment.vertex_position.y * 50.0 + t,
  );

  // **Ruido para la profundidad y textura fina de los cráteres**
  let depth_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 100.0,  // Mayor frecuencia para texturas finas
      fragment.vertex_position.y * 100.0,
  );

  // **Lógica de colores basada en ruido**
  let color = if noise_value > 0.7 {
      // Cráter profundo con textura
      crater_color * (1.0 - depth_noise).clamp(0.5, 1.0) * fragment.intensity
  } else if noise_value > 0.4 {
      // Rocas o superficie elevada
      rock_color * fragment.intensity
  } else {
      // Arena rojiza en el resto de la superficie
      sand_color * fragment.intensity
  };

  color
}

// Función auxiliar para generar ruido Fractal Brownian Motion (FBM)
// Función auxiliar para generar ruido Fractal Brownian Motion (FBM)
fn fbm_noise(noise: &FastNoiseLite, x: f32, y: f32, octaves: usize) -> f32 {
  let mut value = 0.0;
  let mut amplitude = 1.0;
  let mut frequency = 1.0;

  for i in 0..octaves {
      // Añadimos un pequeño offset aleatorio por octava para romper la simetría.
      let offset = i as f32 * 0.1;
      value += noise.get_noise_2d(x * frequency + offset, y * frequency + offset) * amplitude;
      amplitude *= 0.6;  // Cambiamos la reducción para evitar suavizados extremos
      frequency *= 2.0;
  }

  value
}


pub fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores de las bandas gaseosas
  let band_yellow = Color::new(255, 239, 170); // Amarillo suave
  let band_beige = Color::new(230, 220, 170);  // Beige claro
  let band_brown = Color::new(180, 120, 70);   // Marrón claro
  let band_dark_brown = Color::new(120, 70, 40); // Marrón oscuro

  let storm_color = Color::new(255, 69, 0);  // Gran Mancha Roja

  let t = uniforms.time as f32 * 0.02; // Control del tiempo para animaciones

  // **Frecuencia aumentada para más bandas**
  let y_position = fragment.vertex_position.y * 15.0;

  // **Ondas dinámicas en movimiento** combinando seno y ruido FBM
  let wave_pattern = (y_position + (t * 2.0).sin()).sin(); // Ondulación basada en tiempo
  let fbm_value = fbm_noise(
      &uniforms.noise,
      fragment.vertex_position.x * 1.5 + t * 0.05,
      fragment.vertex_position.y * 3.0,
      6,
  );

  // **Turbulencia adicional** para movimiento irregular de las bandas
  let turbulence = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 10.0 + t * 0.3,
      fragment.vertex_position.y * 10.0,
  );

  // **Suavizamos la transición entre bandas con LERP**
  let wave_intensity = ((wave_pattern + fbm_value * 0.5) * 0.5 + 0.5) * (1.0 + turbulence * 0.2);

  let band_color = band_yellow
      .lerp(&band_beige, wave_intensity * 0.5) // Amarillo a beige
      .lerp(&band_brown, wave_intensity * 0.8) // Beige a marrón claro
      .lerp(&band_dark_brown, wave_intensity); // Marrón claro a oscuro

  // **Gran Mancha Roja**: Controlamos su ubicación y tamaño
  let red_spot_dist = ((fragment.vertex_position.x + 0.2).powi(2)
      + (fragment.vertex_position.y - 0.2).powi(2))
      .sqrt();
  let red_spot_intensity = (1.0 - red_spot_dist * 4.0).clamp(0.0, 1.0);

  // **Mezclamos la tormenta con las bandas dinámicas**
  let final_color = if red_spot_intensity > 0.7 {
      storm_color * red_spot_intensity // Mancha Roja activa
  } else {
      band_color * fragment.intensity // Bandas normales
  };

  // **Ajuste dinámico de brillo** para un efecto más natural
  final_color * (1.0 + 0.15 * turbulence).clamp(0.0, 1.2)
}


pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores de la superficie lunar
  let base_gray = Color::new(180, 180, 180);  // Gris claro
  let crater_edge_color = Color::new(120, 120, 120);  // Gris medio
  let crater_center_color = Color::new(80, 80, 80);  // Gris oscuro

  let t = uniforms.time as f32 * 0.1;  // Animación en el tiempo

  // **Ruido basado en coordenadas esféricas** para evitar cortes
  let spherical_x = fragment.vertex_position.x / fragment.vertex_position.z.abs().max(0.1);
  let spherical_y = fragment.vertex_position.y / fragment.vertex_position.z.abs().max(0.1);

  // Generación de cráteres: Más pequeños y distribuidos con FBM
  let crater_noise = fbm_noise(&uniforms.noise, spherical_x * 30.0 + t, spherical_y * 30.0, 4);

  // Máscara para dispersión aleatoria de los cráteres
  let mask_noise = fbm_noise(&uniforms.noise, spherical_x * 60.0, spherical_y * 60.0, 5);

  // Detalles más pequeños en los cráteres (profundidad)
  let depth_noise = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * 300.0,
      fragment.vertex_position.y * 300.0,
  );

  // Lógica para los cráteres: Mayor densidad y suavidad en los bordes
  let crater_effect = if crater_noise > 0.55 && mask_noise > 0.3 {
      crater_center_color.lerp(&crater_edge_color, depth_noise)
  } else {
      base_gray
  };

  // Iluminación basada en Z para simular fases lunares
  let light_factor = 0.5 + 0.5 * fragment.vertex_position.z.clamp(-1.0, 1.0);
  let illuminated_color = crater_effect * light_factor;

  // Aplicamos la intensidad del fragmento al color final
  illuminated_color * fragment.intensity
}


pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let band_color1 = Color::new(210, 180, 140);  
  let band_color2 = Color::new(160, 130, 110);  
  let ring_color1 = Color::new(192, 192, 192);  
  let ring_color2 = Color::new(169, 169, 169);  

  let y_position = fragment.vertex_position.y * 10.0 + uniforms.time as f32 * 0.01;
  let noise_value = uniforms.noise.get_noise_2d(y_position, fragment.vertex_position.x * 10.0);

  let band_color = if noise_value > 0.5 { band_color1 } else { band_color2 };

  let radius = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();
  let angle = fragment.vertex_position.y.atan2(fragment.vertex_position.x);

  let ring_noise = uniforms.noise.get_noise_2d(radius * 10.0, angle * 10.0);
  let ring_color = if ring_noise > 0.5 { ring_color1 } else { ring_color2 };

  if radius > 1.0 && radius < 2.0 {
      ring_color * fragment.intensity
  } else {
      band_color * fragment.intensity
  }
}




pub fn comet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  // Colores del cometa: Cabeza brillante y cola azulada
  let head_color = Color::new(255, 255, 255);  // Blanco brillante (núcleo)
  let tail_color = Color::new(135, 206, 235);  // Azul claro (cola)

  // Parámetro temporal para animación suave
  let t = uniforms.time as f32 * 0.4;  

  // Coordenadas polares para dar forma a la cola del cometa
  let angle = (fragment.vertex_position.y).atan2(fragment.vertex_position.x);
  let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();

  // Ruido para animar la forma dinámica de la cola
  let tail_noise = uniforms.noise.get_noise_3d(
      fragment.vertex_position.x * 10.0 + t,
      fragment.vertex_position.y * 10.0 + t,
      fragment.vertex_position.z * 5.0,
  );

  // Ruido adicional para dispersar las partículas de la cola
  let particle_noise = uniforms.noise.get_noise_2d(
      angle * 20.0,
      distance * 5.0 + t,
  );

  // Intensidad de la cola que disminuye con la distancia del núcleo
  let tail_intensity = (1.0 - distance / 5.0).clamp(0.0, 1.0) * tail_noise;

  // Lógica de color: El núcleo es brillante, la cola se desvanece con la distancia
  let color = if distance < 1.0 {
      head_color * fragment.intensity  // Núcleo brillante
  } else {
      let tail_alpha = (particle_noise * tail_intensity).clamp(0.0, 1.0);
      tail_color * tail_alpha * fragment.intensity  // Cola dinámica y dispersa
  };

  color
}

