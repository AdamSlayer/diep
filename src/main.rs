#[allow(unused)]

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::time::Duration;

/// Contains position, velocity, weight, and has some useful methods
struct Physics {
    x: f64,
    y: f64,
    xvel: f64,
    yvel: f64,
    weight: f64
}
impl Physics {
    /// Applies a one time force in a specified direction, suddenly changing velocity. Has lower impact on heavier objects.
    fn push() {
        todo!()
    }
}

/// Square, triangle, pentagon
struct Shape {
    physics: Physics,
}

/// A tank. Player, bot, boss etc
struct Tank {
    physics: Physics,

}

/// A bullet or a drone(controlled bullet)
struct Bullet {
    physics: Physics,

}

/// Tracks info about a button, like if it is pressed, and what keycodes and/or mouse button activates it
struct Button {
    /// If it is activated by a key, otherwise is None
    keycode: Option<Keycode>,
    /// If it is activated by a mouse button, otherwise is None
    mousebutton: Option<MouseButton>,
    is_down: bool,
    /// if is_down changed this frame
    just: bool
}
/// Tracks which keys(and also mouse buttons) are currently down, because sdl2 only has KeyDown and KeyUp events. Also tracks what keys were just pressed or released (meaning this frame)
/// Allows easy keybind settings.
/// 
/// For mouse, it tracks this frame position delta and current position.
struct Input {
    up: Button,
    down: Button,
    left: Button,
    right: Button,
    fire: Button,

    mouse_pos: (i32,i32),
    mouse_delta: (i32,i32)
}
impl Input {
    /// Initializes the struct. Will read the keybinds from a settings file in the future
    fn init() -> Self {
        Input {
            up: Button { keycode: Some(Keycode::W), mousebutton: None, is_down: false, just: false },
            down: Button { keycode: Some(Keycode::S), mousebutton: None, is_down: false, just: false },
            left: Button { keycode: Some(Keycode::A), mousebutton: None, is_down: false, just: false },
            right: Button { keycode: Some(Keycode::D), mousebutton: None, is_down: false, just: false },
            fire: Button { keycode: None, mousebutton: Some(MouseButton::Left), is_down: false, just: false },
            mouse_pos: (0,0),
            mouse_delta: (0,0), 
        }
    }

    /// Finds what this keycode means (up, down, fire, ..) and updates the respective state
    fn register_keydown(&mut self, keycode: Keycode) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire].iter_mut() {
            if b.keycode.is_some() {
                if b.keycode.unwrap() == keycode {
                    b.is_down = true;
                    b.just = true;
                }
            }
        }
    }

    /// Finds what this keycode means (up, down, fire, ..) and updates the respective state
    fn register_keyup(&mut self, keycode: Keycode) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire].iter_mut() {
            if b.keycode.is_some() {
                if b.keycode.unwrap() == keycode {
                    b.is_down = false;
                    b.just = true;
                }
            }
        }
    }

    /// Finds what this mouse button means (up, down, fire, ..) and updates the respective state
    fn register_mouse_button_down(&mut self, mousebutton: MouseButton) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire].iter_mut() {
            if b.mousebutton.is_some() {
                if b.mousebutton.unwrap() == mousebutton {
                    b.is_down = true;
                    b.just = true;
                }
            }
        }
    }

    /// Finds what this mouse button means (up, down, fire, ..) and updates the respective state
    fn register_mouse_button_up(&mut self, mousebutton: MouseButton) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire].iter_mut() {
            if b.mousebutton.is_some() {
                if b.mousebutton.unwrap() == mousebutton {
                    b.is_down = false;
                    b.just = true;
                }
            }
        }
    }

    /// Call this once every loop, before taking input. Now it only changes just to false for all keys
    fn refresh(&mut self) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire].iter_mut() {
            b.just = false;
        }
    }
}

/// Main struct that stores everything - tanks, shapes, bullets, walls etc.
/// Does not store information about which tank is the player.
struct Map {
    /// All the squares, triangles and pentagons on the map
    shapes: Vec<Shape>, // no need to find a specific shape, so no hashmap but just Vec<>
    /// All the tanks on the map, including player, bots, bosses etc.
    tanks: HashMap<u128, Tank>, // hashmap because of quicker searching for the tank when it's bullet kills something
    /// All the things shot by tanks - bullets or drones. Projectiles that make other things (rocket laucher tank, factory tank) aren't supported
    bullets: HashMap<u128, Bullet> // hashmap to easily iterate over all the tank's bullets, for example when the tank dies, or when there would be a shield that only blocks some tank's bullets (teams?)
}
impl Map {
    /// Makes an empty map. Does not add the player tank or anything else.
    fn new() -> Self {
        Map {
            shapes: Vec::new(),
            tanks: HashMap::new(),
            bullets: HashMap::new(),
        }
    }
}

fn main() {
    // INIT

    // initialize sld2 related things
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("SDL2 Window", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // initialize my own things
    let mut map = Map::new();
    let mut input = Input::init();

    'running: loop {

        input.refresh();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("Closed by Event::Quit");
                    break 'running
                }

                Event::KeyDown { keycode, .. } => {
                    let keycode = keycode.unwrap();

                    // close if key was escape, possibly remove in the future and handle it elsewhere
                    if keycode == Keycode::Escape {
                        println!("Closed by Escape key");
                        break 'running
                    }
                    
                    input.register_keydown(keycode);
                }

                Event::KeyUp { keycode, .. } => {
                    let keycode = keycode.unwrap();                    
                    input.register_keyup(keycode);
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    let mousebutton = mouse_btn;
                    input.register_mouse_button_down(mousebutton);
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    let mousebutton = mouse_btn;
                    input.register_mouse_button_up(mousebutton);
                }

                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    input.mouse_delta = (xrel, yrel);
                    input.mouse_pos = (x,y);
                }
                _ => {}
            }
        }

        println!("{:#?}", input.fire.is_down);

        // Clear the screen with black color and render
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        // Add a small delay to avoid using too much CPU
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
