pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,         // Buffer de color
    pub zbuffer: Vec<f32>,        // Buffer de profundidad
    pub emission_buffer: Vec<u32>, // Buffer de emisión para materiales emisivos
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    // Constructor de Framebuffer, inicializando los buffers
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],                 // Buffer de color inicializado en negro
            zbuffer: vec![f32::INFINITY; width * height],     // Buffer de profundidad inicializado en infinito
            emission_buffer: vec![0; width * height],         // Buffer de emisión inicializado en cero
            background_color: 0x000000,                       // Fondo negro por defecto
            current_color: 0xFFFFFF,                          // Color blanco por defecto
        }
    }

    // Método para limpiar todos los buffers
    pub fn clear(&mut self) {
        // Limpiar el buffer de color
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
        // Limpiar el buffer de profundidad
        for depth in self.zbuffer.iter_mut() {
            *depth = f32::INFINITY;
        }
        // Limpiar el buffer de emisión
        for emission in self.emission_buffer.iter_mut() {
            *emission = 0;
        }
    }

    // Método para establecer el color del fondo
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    // Método para establecer el color actual para dibujar
    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    // Dibuja un punto con soporte para emisión
    pub fn point_with_emission(&mut self, x: usize, y: usize, depth: f32, emission: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;

            // Verificamos si la nueva profundidad es menor que la actual
            if self.zbuffer[index] > depth {
                self.buffer[index] = self.current_color;  // Actualizamos color
                self.emission_buffer[index] = emission;   // Guardamos el valor de emisión
                self.zbuffer[index] = depth;              // Actualizamos profundidad
            }
        }
    }
}
