pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,         
    pub zbuffer: Vec<f32>,        
    pub emission_buffer: Vec<u32>, 
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],                 
            zbuffer: vec![f32::INFINITY; width * height],     
            emission_buffer: vec![0; width * height],         
            background_color: 0x000000,                       
            current_color: 0xFFFFFF,                          
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
        for depth in self.zbuffer.iter_mut() {
            *depth = f32::INFINITY;
        }
        for emission in self.emission_buffer.iter_mut() {
            *emission = 0;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn point_with_emission(&mut self, x: usize, y: usize, depth: f32, emission: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;

            if self.zbuffer[index] > depth {
                self.buffer[index] = self.current_color;  
                self.emission_buffer[index] = emission;   
                self.zbuffer[index] = depth;              
            }
        }
    }
}
