#[allow(unused)]

extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::collections::HashMap;
use std::time::{Duration, Instant};



/// Contains position, velocity, weight, and has some useful methods
struct Physics {
    x: f64,
    y: f64,
    xvel: f64,
    yvel: f64,
    weight: f64,
    /// degrees
    rot: f64,
    /// degrees per second
    rotvel: f64,
}
impl Physics {
    /// Applies a one time force in a specified direction, suddenly changing velocity. Has lower impact on heavier objects.
    fn push(&mut self, f: (f64, f64)) {
        self.xvel += f.0/self.weight;
        self.yvel += f.1/self.weight;
    }
    
    fn update(&mut self, delta: u128) {
        self.x += self.xvel * (delta as f64 / 1_000_000.);
        self.y += self.yvel * (delta as f64 / 1_000_000.);
        self.rot += self.rotvel * (delta as f64 / 1_000_000.);

        self.xvel *= (-(delta as f64)/1_000_000.).exp();
        self.yvel *= (-(delta as f64)/1_000_000.).exp();
        self.rotvel *= (-(delta as f64)/1_000_000.).exp();
    }
}

/// Square, triangle, pentagon
struct Shape {
    physics: Physics,
    hp: u32
}

/// A tank. Player, bot, boss etc
struct Tank {
    physics: Physics,
    hp: u32,
    /// How much power the tank can allpy to it's movement. Will move faster with more power, but slower is it weights more.
    power: f64,
}
impl Tank {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get("tank").unwrap();
        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from((self.physics.x as i32, self.physics.y as i32)), // set center position
                100, 100  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((50,50)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
    }


    fn move_in_dir(&mut self, dir: (f64, f64)) {
        // noramlize vector
        let magnitude = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        if magnitude != 0.0 {
            let normalized_dir = (dir.0 / magnitude, dir.1 / magnitude);
            self.physics.push((normalized_dir.0*self.power, normalized_dir.1*self.power));
        } else {
            println!("attempt to move a tank in the direction (0.,0.)");
        }
    }
}

/// xy pos, zoom and target tank the camera follows(usally player tank).
/// 
/// TODO some camera settings, and following other things than tanks
struct Camera {
    x: f64,
    y: f64,
    /// Bigger value => things look bigger (basically scale)
    zoom: f64,
    target_tank: u128

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

    /// Updates the positions based on velocities of all objects, and slows down velocities by frincion/resistance
    fn update_physics(&mut self, delta: u128) {
        // mutable iterator over the physics' of all tanks, bullets and shapes
        let combined_iter = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
        .chain(self.bullets.iter_mut().map(|tank| &mut tank.1.physics))
        .chain(self.shapes.iter_mut().map(|tank| &mut tank.physics));

        for o in combined_iter {
            o.update(delta);

            println!("x: {}", o.x);
        }
    }
}

fn main() {
    // INIT

    // Initialize sld2 related things
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("SDL2 Window", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize other libraries. So far only the rand crate
    let mut rng = rand::thread_rng();

    // load textures. will be moved to its own function in the future
    let texture_creator = canvas.texture_creator();
    // Hashmap of all the textures used in the game. Later will read all textures form the textures folder and add them to the hashmap by the filename without the extension
    let mut textures: HashMap<String, Texture> = HashMap::new();
    textures.insert("tank".to_owned(), texture_creator.load_texture("textures/t.png").unwrap());

    // Initialize my own things
    let mut map = Map::new();
    let mut input = Input::init();
    let playerid: u128 = rng.gen();
    let mut camera = Camera {
        x: 0.,
        y: 0.,
        zoom: 1.,
        target_tank: playerid
    };

    // add player
    map.tanks.insert(
        playerid,
        Tank {
            physics: Physics {
                x: 0.,
                y: 0.,
                xvel: 0.,
                yvel: 0.,
                weight: 100.,
                rot: 0.,
                rotvel: 9000.
            },
            hp: 100,
            power: 100.
        }
    );

    let mut last_frame_start = Instant::now();
    // How long the last frame took, in micros, 1 millisecond for the first frame
    let mut delta = 1_000;

    'running: loop {

        last_frame_start = Instant::now();
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

        // PLAYER CONTROL
        if input.up.is_down {
            map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.,-1.));
        }
        if input.down.is_down {
            map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.,1.));
        }
        if input.left.is_down {
            map.tanks.get_mut(&playerid).unwrap().move_in_dir((-1.,0.));
        }
        if input.right.is_down {
            map.tanks.get_mut(&playerid).unwrap().move_in_dir((1.,0.));
        }

        // PHYSICS

        // at the end of physics, update all physics
        map.update_physics(delta);


        // RENDER
        
        // Clear the screen with black color
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();


        // Render all tanks
        for tank in &map.tanks {
            tank.1.render(&mut canvas, &mut camera, &textures);
        }

        canvas.present();

        // Add a small delay to avoid using too much CPU
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        delta = Instant::now().duration_since(last_frame_start).as_micros();
    }
}
