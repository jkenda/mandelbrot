use winit::{
    dpi::PhysicalPosition,
    event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, MouseButton, MouseScrollDelta}};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Properties {
    pub center: [f64; 2],
    pub zoom: f64,
    aspect: f32,
    math64: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Properties32 {
    pub center: [f32; 2],
    pub zoom: f32,
    aspect: f32,
    math64: u32,
}

impl Default for Properties {
    fn default() -> Self {
        Properties {
            center: [-0.75, 0.0],
            zoom: 1.2,
            aspect: 1.0,
            math64: 0,
        }
    }
}

impl From<Properties> for Properties32 {
    fn from(properties: Properties) -> Self {
        Properties32 {
            center: [
                properties.center[0] as f32,
                properties.center[1] as f32,
            ],
            zoom: properties.zoom as f32,
            aspect: properties.aspect,
            math64: 0,
        }
    }
}


pub struct CameraController {
    window_size: (f64, f64),
    properties: Properties,
    speed: f64,
    mouse_position: PhysicalPosition<f64>,
    is_mouse_left_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f64, width: u32, height: u32) -> Self {
        Self {
            window_size: (width as f64, height as f64),
            speed,
            properties: Default::default(),
            mouse_position: Default::default(),
            is_mouse_left_pressed: Default::default(),
        }
    }

    pub fn properties(&self) -> Properties {
        self.properties
    }

    pub fn properties32(&self) -> Properties32 {
        Properties32::from(self.properties)
    }

    pub fn mouse_position(&self) -> PhysicalPosition<f64> {
        self.mouse_position
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                let update = is_pressed == true;
                match keycode {
                    VirtualKeyCode::H | VirtualKeyCode::Left => {
                        self.move_center(PhysicalPosition::new(-self.speed * self.properties.zoom, 0.0));
                        update
                    }
                    VirtualKeyCode::J | VirtualKeyCode::Down => {
                        self.move_center(PhysicalPosition::new(0.0, -self.speed * self.properties.zoom));
                        update
                    }
                    VirtualKeyCode::K | VirtualKeyCode::Up => {
                        self.move_center(PhysicalPosition::new(0.0, self.speed * self.properties.zoom));
                        update
                    }
                    VirtualKeyCode::L | VirtualKeyCode::Right => {
                        self.move_center(PhysicalPosition::new(self.speed * self.properties.zoom, 0.0));
                        update
                    }
                    VirtualKeyCode::A | VirtualKeyCode::PageUp => {
                        self.zoom(PhysicalPosition::new(0.0, 0.0), -self.speed);
                        update
                    }
                    VirtualKeyCode::S | VirtualKeyCode::PageDown => {
                        self.zoom(PhysicalPosition::new(0.0, 0.0), self.speed);
                        update
                    }
                    VirtualKeyCode::Space => {
                        self.properties = Default::default();
                        update
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.is_mouse_left_pressed = *state == ElementState::Pressed;
                false
            },
            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                let (width, height) = self.window_size;

                let prev_position = self.mouse_position;
                let curr_position = PhysicalPosition::new(
                      position.x * 2.0 / width  - 1.0,
                    -(position.y * 2.0 / height - 1.0));

                let dx = curr_position.x - prev_position.x;
                let dy = curr_position.y - prev_position.y;

                self.mouse_position = curr_position;

                if self.is_mouse_left_pressed {
                    self.move_center(PhysicalPosition::new(
                        -dx * self.properties.zoom * 2.0,
                        -dy * self.properties.zoom * 2.0));
                    true
                }
                else {
                    false
                }
            }
            WindowEvent::TouchpadMagnify { delta, phase: _, .. } => {
                self.zoom(self.mouse_position, *delta)
            }
            WindowEvent::MouseWheel { device_id: _, delta, .. } => {
                let (_x, delta) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => ((x * 0.11) as f64, (y * 0.11) as f64),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x * 0.001, pos.y * 0.001),
                };

                self.zoom(self.mouse_position, -delta)
            }
            _ => false,
        }
    }

    fn zoom(&mut self, center: PhysicalPosition<f64>, delta: f64) -> bool {
        if delta > 0.0 && self.properties.zoom >= 5.0 {
            return false
        }

        let factor = 1.0 + delta;

        self.properties.zoom *= factor;

        self.move_center(PhysicalPosition::new(
                center.x * (1.0 - factor) * self.properties.zoom,
                center.y * (1.0 - factor) * self.properties.zoom));

        self.properties.math64 = if self.properties.zoom < 1.0 / 10_000.0 { 1 } else { 0 };
        true
    }

    fn move_center(&mut self, delta: PhysicalPosition<f64>) {
        self.properties.center[0] += delta.x;
        self.properties.center[1] += delta.y;
    }

    pub fn update_window_size(&mut self, width: u32, height: u32) {
        self.window_size = (width as f64, height as f64);
        self.properties.aspect = width as f32 / height as f32;
    }
}

