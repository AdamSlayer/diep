#[allow(unused)]

extern crate sdl2;
use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// From A to B
fn angle_diff(a: f64, b: f64) -> f64 {
    let mut diff = b - a;
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}


/// Contains position, velocity, weight, and has some useful methods
/// 
/// Clone needed for firing bullets, whuch first duplicate tank physics and then have a push applied
#[derive(Debug, Clone, Copy)]
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
    /// Applies a one time push in a specified direction, suddenly changing velocity. Has lower impact on heavier objects.
    fn push(&mut self, f: (f64, f64)) {
        self.xvel += f.0/self.weight;
        self.yvel += f.1/self.weight;
    }

    /// Applies a one time force push in a specified direction from -1 to 1, suddenly changing rotation velocity. Has lower impact on heavier objects.
    fn push_rot(&mut self, f: f64) {
        self.rotvel += f/self.weight;
    }
    
    fn update(&mut self, delta: u128) {
        self.x += self.xvel * (delta as f64 / 1_000_000.);
        self.y += self.yvel * (delta as f64 / 1_000_000.);
        self.rot += self.rotvel * (delta as f64 / 1_000_000.);
        self.rot = self.rot%360.;

        self.xvel *= (-(delta as f64)/1_000_000.).exp();
        self.yvel *= (-(delta as f64)/1_000_000.).exp();
        self.rotvel *= (-(delta as f64)/1_000_000.).exp();
    }

    /// returns the speed of the object - sqrt(xvel**2 + yvel**2)
    fn speed(&self) -> f64 {
        f64::sqrt(self.xvel.powi(2) + self.yvel.powi(2))
    }
}

/// Square, triangle, pentagon
struct Shape {
    physics: Physics,
    hp: u32
}

/// Turrets can now only shoot bullets, will change later
struct Turret {
    projectile_speed: f64,
    projectile_weight: f64,
    /// in micros, first shot is immediatae
    reload_time: u128,
    /// mean in degrees, gaussian propability distribution
    /// also randomizes projectile speed, at rate 1 degree = 1% speed
    inaccuracy: f64,
    /// in degrees, turret facing relative to tank facing
    relative_direction: f64,

    // start of changing properties

    time_to_next_shot: u128
}
impl Turret {
    /// Returns an Option<Bullet> if fired, and None otherwise.
    /// Tank physics can be physics of anything, theoretically allowing bullets of shapes to fire bullets too if they have a turret
    fn fire(&mut self, tank_physics: &Physics, tank_id: u128) -> Option<Bullet> {
        if self.time_to_next_shot > 0 {
            None
        } else {
            let random_speed: f64;
            let random_direction: f64;
            if self.inaccuracy != 0. {
                // Create a Gaussian distribution with the specified standard deviation
                let normal = rand_distr::Normal::new(0., self.inaccuracy).expect("Invalid parameters for normal distribution");

                // Generate random numbers from the Gaussian distribution
                random_speed = self.projectile_speed*(1. + 0.01*normal.sample(&mut thread_rng()));
                random_direction = normal.sample(&mut thread_rng());
            } else {
                random_speed = self.projectile_speed;
                random_direction = 0.;
            }

            // calculate bullet speed vector relative to tank
            let fire_vector = (
                random_speed * (self.relative_direction+tank_physics.rot + random_direction).to_radians().sin(),
                -random_speed* (self.relative_direction+tank_physics.rot + random_direction).to_radians().cos()
            );

            // duplicate tank physics
            let mut bullet_physics = tank_physics.clone();

            // set bullet weight and push it
            bullet_physics.weight = self.projectile_weight;
            bullet_physics.push(fire_vector);

            self.time_to_next_shot = self.reload_time;

            Some(Bullet {
                physics: bullet_physics,
                dead_speed: bullet_physics.speed() * 0.5,
                source_tank_id: tank_id
            })
        }
    }
}

/// A tank. Player, bot, boss etc
struct Tank {
    physics: Physics,
    hp: u32,
    /// How much power the tank can apply to it's movement. Will move faster with more power, but slower if it weights more.
    power: f64,
    /// How much power the tank can apply to it's rotation movement. Will rotate faster with more power, but slower if it weights more.
    rot_power: f64,
    turrets: Vec<Turret>,
    bullet_ids: Vec<u128>
}
impl Tank {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get("tank").unwrap();
        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(camera.to_screen_coords((self.physics.x, self.physics.y))), // set center position
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

    /// Applies rotation force to the tank, rotating it towards a point over time. `to` is on map coordinates
    fn rotate_to(&mut self, to: (f64, f64)) {
        let tg_angle = -f64::atan2(self.physics.x - to.0, self.physics.y - to.1).to_degrees();
        self.physics.push_rot(angle_diff(self.physics.rot + self.physics.rotvel/2., tg_angle)/180.*self.rot_power);
    }

    /// Fires from all the tank's reloaded turrets
    /// Will make the bullets belong to `source_id` (for sake of eg. who did the kill)
    fn fire(&mut self, bullets: &mut HashMap<u128, Bullet>, source_id: u128) {
        for t in &mut self.turrets {
            let bullet = t.fire(&self.physics, source_id);
            if bullet.is_some() {
                let x:u128 = thread_rng().gen();
                bullets.insert(x, bullet.unwrap());
            }
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
    target_tank: u128,
    viewport_size: (i32, i32)

} 
impl Camera {
    /// Converts from screen to map coordinates
    fn to_map_coords(&self, coords: (i32, i32)) -> (f64, f64) {
        let x = (coords.0 as f64) / self.zoom + self.x - (self.viewport_size.0 / 2) as f64;
        let y = (coords.1 as f64) / self.zoom + self.y - (self.viewport_size.1 / 2) as f64;
        (x, y)
    }

    /// Converts from map to screen coordinates
    fn to_screen_coords(&self, coords: (f64, f64)) -> (i32, i32) {
        let x = ((coords.0 - self.x + (self.viewport_size.0 / 2) as f64) * self.zoom) as i32;
        let y = ((coords.1 - self.y + (self.viewport_size.1 / 2) as f64) * self.zoom) as i32;
        (x, y)
    }

    fn track(&mut self, delta: u128, tg: &Physics) {
        self.x = tg.x;
        self.y = tg.y;
    }
}

/// A bullet or a drone(controlled bullet)
#[derive(Debug)]
struct Bullet {
    physics: Physics,
    /// the speed at which the bullet will be removed
    dead_speed: f64,
    source_tank_id: u128
}
impl Bullet {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get("bullet").unwrap();
        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(camera.to_screen_coords((self.physics.x, self.physics.y))), // set center position
                30, 30  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((15,15)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
    }
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
    /// 0,0 is at the center of the map. this is the distance of the walls in x and y. actual size is thenfore double this
    map_size: (f64, f64),
    /// All the squares, triangles and pentagons on the map
    shapes: Vec<Shape>, // no need to find a specific shape, so no hashmap but just Vec<>
    /// maximum number of shapes on the map, maxes for each shape type will be derived from this
    shapes_max: usize,
    /// All the tanks on the map, including player, bots, bosses etc.
    tanks: HashMap<u128, Tank>, // hashmap because of quicker searching for the tank when it's bullet kills something
    /// All the things shot by tanks - bullets or drones. Projectiles that make other things (rocket laucher tank, factory tank) aren't supported
    bullets: HashMap<u128, Bullet> // hashmap to easily iterate over all the tank's bullets, for example when the tank dies, or when there would be a shield that only blocks some tank's bullets (teams?)
}
impl Map {
    /// Makes an empty map. Does not add the player tank or anything else.
    fn new() -> Self {
        Map {
            map_size: (10000., 10000.,),
            shapes_max: 100,
            shapes: Vec::new(),
            tanks: HashMap::new(),
            bullets: HashMap::new(),
        }
    }

    /// renders grid, walls, maybe more in the future
    fn render(&self, canvas: &mut Canvas<Window> , camera: &Camera) {
        for x in ((camera.x - 1./camera.zoom*camera.viewport_size.0 as f64).floor() as i32..(camera.x + 1./camera.zoom*camera.viewport_size.0 as f64).ceil() as i32).filter(|x| x%40 == 0) {
            canvas.set_draw_color(Color::GRAY);
            canvas.draw_line(Point::from((x - camera.x as i32, i32::MAX)), Point::from((x - camera.x as i32, i32::MIN))).expect("failed to draw line");
        }

        for y in ((camera.y - 1./camera.zoom*camera.viewport_size.1 as f64).floor() as i32..(camera.y + 1./camera.zoom*camera.viewport_size.1 as f64).ceil() as i32).filter(|y| y%40 == 0) {
            canvas.set_draw_color(Color::GRAY);
            canvas.draw_line(Point::from((i32::MAX, y - camera.y as i32)), Point::from((i32::MIN, y - camera.y as i32))).expect("failed to draw line");
        }
    }

    /// This contains a lot of things, and is called every frame. Includes velocity/position calculations, removing slow bullets, spawns shapes, more in the future
    /// 
    /// Updates the positions based on velocities of all objects, and slows down velocities by frincion/resistance
    /// 
    /// Updates times to realod for all turrets (only tank turrets now)
    /// 
    /// Removes all bullets s.t. the bullet speed is below it's dead speed
    /// 
    /// Randomly spawns shapes
    /// 
    fn update_physics(&mut self, delta: u128) {
        // mutable iterator over the physics' of all tanks, bullets and shapes
        let combined_iter = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
        .chain(self.bullets.iter_mut().map(|tank| &mut tank.1.physics))
        .chain(self.shapes.iter_mut().map(|tank| &mut tank.physics));

        for o in combined_iter {
            o.update(delta);
        }

        for tank in self.tanks.values_mut() {
            for turret in &mut tank.turrets {
                // substracts delta from time to next shot, but doesn't go below zero
                turret.time_to_next_shot = turret.time_to_next_shot - turret.time_to_next_shot.min(delta);
            }
        }

        // removes all bullets with speed less than dead_speed
        self.bullets.retain(|id, bullet| bullet.physics.speed() >= bullet.dead_speed);

        // spawn shapes
        if self.shapes.len() < self.shapes_max {
            // TODO add shapes
        }

    }
}

fn main() {
    // INIT

    // Initialize sld2 related things
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("SDL2 Window", 1024, 1024)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize other libraries. So far only the rand crate
    let mut rng = rand::thread_rng();

    // load textures. will be moved to its own function in the future
    let texture_creator = canvas.texture_creator();
    // HashMap of all the textures used in the game. Later will read all textures form the textures folder and add them to the hashmap by the filename without the extension
    let mut textures: HashMap<String, Texture> = HashMap::new();
    textures.insert("tank".to_owned(), texture_creator.load_texture("textures/t.png").unwrap());
    textures.insert("bullet".to_owned(), texture_creator.load_texture("textures/b.png").unwrap());

    // Initialize my own things
    let mut map = Map::new();
    let mut input = Input::init();
    let playerid: u128 = rng.gen();
    let mut camera = Camera {
        x: 0.,
        y: 0.,
        zoom: 1.,
        target_tank: playerid,
        viewport_size: (1024, 1024)
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
            turrets: vec![Turret {
                projectile_speed: 10000.,
                projectile_weight: 10.,
                reload_time: 10_000,
                inaccuracy: 2.,
                relative_direction: 0.,
                time_to_next_shot: 0
            }],
            hp: 100,
            power: 300.,
            rot_power: 3000.,
            bullet_ids: vec![],
        }
    );

    let mut last_frame_start;
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

        //movement
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

        //rotation
        map.tanks.get_mut(&playerid).unwrap().rotate_to(camera.to_map_coords(input.mouse_pos));

        //firing
        if input.fire.is_down {
            map.tanks.get_mut(&playerid).unwrap().fire(&mut map.bullets, playerid);
        }

        // PHYSICS

        // at the end of physics, update all physics
        map.update_physics(delta);


        // CAMERA
        camera.track(delta, &map.tanks.get(&camera.target_tank).expect("Failed to get the tank to track with camera").physics);


        // RENDER
        // render the things you want to appear on top last

        // Clear the screen with black color
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // render map walls and grid
        map.render(&mut canvas, &camera);


        // Render all bullets
        for bullet in &map.bullets {
            bullet.1.render(&mut canvas, &mut camera, &textures);
        }

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
