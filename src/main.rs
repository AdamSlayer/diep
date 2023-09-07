#[allow(unused)]

extern crate sdl2;
use rand::prelude::*;
use rand_distr::num_traits::Pow;
use rand_distr::{Distribution, Normal};
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::collections::{HashMap, HashSet};
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

fn normalize(v: (f64, f64)) -> (f64, f64) {
    // noramlize vector
    let magnitude = (v.0 * v.0 + v.1 * v.1).sqrt();
    return (v.0 / magnitude, v.1 / magnitude);
}


/// Contains position/rotation, velocity, weight, hp related variables
/// 
/// Cloning is not expensive
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
    /// radius of the collision circle
    collision_size: f64,
    /// current health
    hp: f64,
    max_hp: f64,
    /// per second
    hp_regen: f64,
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
    
    /// update velocity by friction, position/rotation by velocity, and hp by hp_regen
    fn update(&mut self, delta: f64) {
        self.x += self.xvel * (delta);
        self.y += self.yvel * (delta);
        self.rot += self.rotvel * (delta);
        self.rot = self.rot%360.;

        self.xvel *= (-delta*0.5).exp();
        self.yvel *= (-delta*0.5).exp();
        self.rotvel *= (-delta*1.).exp();

        self.hp = (self.hp + self.hp_regen*(delta)).min(self.max_hp);
    }

    /// returns the speed of the object - sqrt(xvel**2 + yvel**2)
    fn speed(&self) -> f64 {
        f64::sqrt(self.xvel.powi(2) + self.yvel.powi(2))
    }

    fn dist(&self, other: &Physics) -> f64 {
        f64::sqrt((self.x - other.x).powi(2) + (self.y - other.y).powi(2))
    }

    /// Only moves self, need to be called in reverse to move `b`
    fn collide(&mut self, b: &Physics, delta: f64) {
        if (self.collision_size + b.collision_size) > self.dist(&b) {
            // how much the collision circles are overlaping * b.weight * delta
            let s = delta*131_072.;
            self.xvel *= (-delta*4.).exp();
            self.yvel *= (-delta*4.).exp();
            let n = normalize((self.x - b.x, self.y - b.y));
            self.push((n.0*s, n.1*s));
            self.hp -= (s/1024.).min(b.hp);
            self.push_rot(delta*b.speed()*angle_diff(f64::atan2(b.x - self.x, b.y - self.y).to_degrees(), f64::atan2((b.x + b.xvel) - (self.x + self.xvel), (b.y + b.yvel) - (self.y + self.yvel)).to_degrees())/-1.);
        }
    }

    /// Only checks if the object touch. For the collision to do anything use 'collide'
    fn collides(&self, b: &Physics) -> bool {
        (self.collision_size + b.collision_size) > self.dist(&b)
    }
}

/// Square, triangle, pentagon
struct Shape {
    physics: Physics,
    /// Also affects behavour
    texture: String
}
impl Shape {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get(&self.texture).unwrap();
        let shape_screen_pos = camera.to_screen_coords((self.physics.x, self.physics.y));

        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(shape_screen_pos), // set center position
                self.physics.collision_size as u32 * 4, self.physics.collision_size as u32 * 4  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((self.physics.collision_size as i32 * 2, self.physics.collision_size as i32 * 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
        
        if self.physics.hp < self.physics.max_hp {
            canvas.set_draw_color(Color::RGB(63,63,63));
            canvas.draw_line((shape_screen_pos.0 - 50, shape_screen_pos.1 - 70), (shape_screen_pos.0 + 50, shape_screen_pos.1 - 70)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((shape_screen_pos.0 - 50, shape_screen_pos.1 - 70), (shape_screen_pos.0 - 50 + (self.physics.hp/self.physics.max_hp*100.) as i32, shape_screen_pos.1 - 70)).unwrap();
        }
    }
}

/// Turrets can now only shoot bullets, will change later
struct Turret {
    projectile_speed: f64,
    projectile_weight: f64,
    projectile_collision_size: f64,
    projectile_hp_regen: f64,
    projectile_hp: f64,
    /// in micros, first shot is immediatae
    reload_time: f64,
    /// mean in degrees, gaussian propability distribution
    /// also randomizes projectile speed, at rate 1 degree = 1% speed
    inaccuracy: f64,
    /// in degrees, turret facing relative to tank facing
    relative_direction: f64,

    // start of changing properties

    time_to_next_shot: f64
}
impl Turret {
    /// Returns an Option<Bullet> if fired, and None otherwise.
    /// Tank physics can be physics of anything, theoretically allowing bullets of shapes to fire bullets too if they have a turret
    fn fire(&mut self, tank_physics: &Physics, tank_id: u128) -> Option<Bullet> {
        if self.time_to_next_shot > 0. {
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
            bullet_physics.collision_size = self.projectile_collision_size;
            bullet_physics.push(fire_vector);
            bullet_physics.x += self.relative_direction+tank_physics.rot.to_radians().sin()*(tank_physics.collision_size + self.projectile_collision_size)*1.01;
            bullet_physics.y -= self.relative_direction+tank_physics.rot.to_radians().cos()*(tank_physics.collision_size + self.projectile_collision_size)*1.01;
            bullet_physics.hp_regen = self.projectile_hp_regen;
            bullet_physics.hp = self.projectile_hp;

            self.time_to_next_shot = self.reload_time;

            Some(Bullet {
                physics: bullet_physics,
                source_tank_id: tank_id,
                texture: "bullet".to_owned()
            })
        }
    }
}

/// A tank. Player, bot, boss etc
struct Tank {
    physics: Physics,
    /// How much power the tank can apply to it's movement. Will move faster with more power, but slower if it weights more.
    power: f64,
    /// How much power the tank can apply to it's rotation movement. Will rotate faster with more power, but slower if it weights more.
    rot_power: f64,
    turrets: Vec<Turret>,
    bullet_ids: Vec<u128>,
    texture: String
}
impl Tank {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get(&self.texture).unwrap();
        let tank_screen_pos = camera.to_screen_coords((self.physics.x, self.physics.y));

        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(tank_screen_pos), // set center position
                self.physics.collision_size as u32 * 4, self.physics.collision_size as u32 * 4  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((self.physics.collision_size as i32 * 2,self.physics.collision_size as i32 * 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
        
        if self.physics.hp < self.physics.max_hp {
            canvas.set_draw_color(Color::RGB(63,63,63));
            canvas.draw_line((tank_screen_pos.0 - 50, tank_screen_pos.1 - 70), (tank_screen_pos.0 + 50, tank_screen_pos.1 - 70)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((tank_screen_pos.0 - 50, tank_screen_pos.1 - 70), (tank_screen_pos.0 - 50 + (self.physics.hp/self.physics.max_hp*100.) as i32, tank_screen_pos.1 - 70)).unwrap();
        }
    }

    /// `dir` doesn't need to be normalized
    fn move_in_dir(&mut self, dir: (f64, f64), delta: f64) {
        // noramlize vector
        let magnitude = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        let normalized_dir = (dir.0 / magnitude, dir.1 / magnitude);
        if (dir == (0., 0.)) || angle_diff(f64::atan2(dir.0, dir.1).to_degrees(), f64::atan2(self.physics.xvel, self.physics.yvel).to_degrees()).abs() > 90.0_f64 {
            self.physics.xvel *= (-delta*4.).exp();
            self.physics.yvel *= (-delta*4.).exp();
            if dir == (0., 0.) {
                let magnitude = (self.physics.xvel * self.physics.xvel + self.physics.yvel * self.physics.yvel).sqrt();
                if magnitude != 0. {
                    let normalized_vel = (self.physics.xvel / magnitude, self.physics.yvel / magnitude);
                    self.physics.push((-normalized_vel.0*self.power*delta, -normalized_vel.1*self.power*delta));
                }
                return
            }
        }
        self.physics.push((normalized_dir.0*self.power*delta, normalized_dir.1*self.power*delta));
    }

    /// Applies rotation force to the tank, rotating it towards a point over time. `to` is on map coordinates
    fn rotate_to(&mut self, to: (f64, f64)) {
        let tg_angle = -f64::atan2(self.physics.x - to.0, self.physics.y - to.1).to_degrees();
        self.physics.push_rot(angle_diff(self.physics.rot + self.physics.rotvel/4., tg_angle)/180.*self.rot_power);
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

    fn track(&mut self, delta: f64, tg: &Physics) {
        self.x = tg.x;
        self.y = tg.y;
    }
}

/// A bullet or a drone(controlled bullet)
#[derive(Debug)]
struct Bullet {
    physics: Physics,
    source_tank_id: u128,
    texture: String
}
impl Bullet {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let texture = textures.get(&self.texture).unwrap();
        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(camera.to_screen_coords((self.physics.x, self.physics.y))), // set center position
                self.physics.collision_size as u32*4, self.physics.collision_size as u32*4,  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((self.physics.collision_size as i32 * 2, self.physics.collision_size as i32 * 2)), // set center of rotation, in screen coordinates (not texture coordinates)
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

/// This is an AI for controlling a tank. Just add the tank ID to the `tankids` HashSet to let this AI control it.
/// 
/// Make sure no tank is controlled by multiple AIs. Tanks are automatically removed when they die.
/// 
/// You can use multiple AIs to control tanks on the map, each with different settings.
struct TankAI {
    /// IDs of all the tanks this AI controls. Make sure no tank is controlled by multiple AIs.
    tankids: HashSet<u128>,
    /// how far away the target must be for the tank to attack or retreat. It will ignore tanks outside of range entirely
    range: f64,
    /// the range at which the tank tries to be from it's target. It will move closer of further accordingly. Set to 0. for smasher tanks to make them try to collide with enemies
    tg_range: f64,
    /// how fast the bullets are, used for aiming. projectile_speed/projectile_weight of the turret
    bullet_speed: f64,
    /// between 0 and 1, how much health it needs to keep attacking, when it has less it will escape away
    fight_or_flight_threshold: f64,
    /// set to `true` to make the tank dodge obstacles. Usually its good to turn on, besides tanks like smasher that do a lot of damage by colliding
    dodge_obstacles: bool
}
impl TankAI {
    /// Controls all the tanks in it's `tankids` - makes them move and shoot based on `Map`
    ///
    /// Needs to access the whole `Map` mutably to modify the tanks it controls.
    /// 
    fn control(&mut self, tanks: &mut HashMap<u128, Tank>, shapes: &mut HashMap<u128, Shape>, mut bullets: &mut HashMap<u128, Bullet>, delta: f64) {
        let mut ids_to_remove = vec![];
        for id in &self.tankids {
            // controlled tank's physics

            if tanks.get(&id).is_some() {
                let con_tankp = tanks.get(&id).unwrap().physics;
                let mut movedir = (0.,0.);

                // search for nearest tank
                let mut closest_id = 0_u128;
                let mut closest_dist = self.range;
                for (oid, tank) in tanks.iter() {
                    if tank.physics.dist(&tanks.get(id).unwrap().physics) < closest_dist && id != oid {
                        closest_dist = tank.physics.dist(&tanks.get(id).unwrap().physics);
                        closest_id = *oid;
                    }
                }
                // if it found nearest tank within range attack of retreat
                if closest_dist < self.range {
                    // closest tank's physics
                    let clo_tankp = tanks.get(&closest_id).unwrap().physics;
                    let tg_pos = (clo_tankp.x, clo_tankp.y);
                    let tg_vel = ((clo_tankp.xvel - con_tankp.xvel) * ((closest_dist/(0.6 * self.bullet_speed)).exp()) / 3.0, (clo_tankp.yvel - con_tankp.yvel) * ((closest_dist/(0.6 * self.bullet_speed)).exp()) / 3.0);
                    tanks.get_mut(id).unwrap().rotate_to((tg_pos.0 + tg_vel.0, tg_pos.1 + tg_vel.1));
                    tanks.get_mut(id).unwrap().fire(&mut bullets, *id);

                    // move
                    if (con_tankp.hp / con_tankp.max_hp) > self.fight_or_flight_threshold && con_tankp.dist(&clo_tankp) > self.tg_range {
                        // move towards
                        movedir = (tg_pos.0 + tg_vel.0 - con_tankp.x, tg_pos.1 + tg_vel.1 - con_tankp.y);
                    } else {
                        // move away
                        movedir = (-(tg_pos.0 + tg_vel.0 - con_tankp.x), -(tg_pos.1 + tg_vel.1 - con_tankp.y));
                    }
                // if it didn't find a tank within range, attack closest shape
                } else {
                    // search for nearest shape
                    let mut closest_id = 0_u128;
                    let mut closest_dist = 10_f64.powi(100);
                    for (oid, shape) in shapes.iter() {
                        if shape.physics.dist(&tanks.get(id).unwrap().physics) < closest_dist && id != oid {
                            closest_dist = shape.physics.dist(&tanks.get(id).unwrap().physics);
                            closest_id = *oid;
                        }
                    }
                    // if it found a shape, attack and chase it
                    if closest_id != 0 {
                        let clo_shapep = shapes.get(&closest_id).unwrap().physics;
                        let tg_pos = (clo_shapep.x, clo_shapep.y);
                        tanks.get_mut(id).unwrap().rotate_to((tg_pos.0, tg_pos.1));
                        tanks.get_mut(id).unwrap().fire(&mut bullets, *id);
                        movedir = (tg_pos.0 - con_tankp.x, tg_pos.1 - con_tankp.y);
                    }

                }

                // dodge obstacles
                if self.dodge_obstacles {
                    for sp in shapes.iter().map(|s| s.1.physics) {
                        // if the shape is close, and if the shape hp is more than 10% of tank's hp
                        if con_tankp.dist(&sp) < (sp.collision_size + con_tankp.collision_size)*1.6 && sp.hp > con_tankp.hp*0.1 {
                            // move directly away from the shape, overriding the move direction determined before
                            let shape_away_dir = normalize((-(sp.x - con_tankp.x), -(sp.y - con_tankp.y)));
                            movedir = normalize(movedir);
                            movedir = (movedir.0 + shape_away_dir.0, movedir.1 + shape_away_dir.1);

                        }
                    }
                }

                tanks.get_mut(id).unwrap().move_in_dir(movedir, delta);

            } else {
                // Tank with id 'id' is not in 'tanks', it appearently died. Remove from list of controlled tanks
                ids_to_remove.push(*id);
            }
        }
        for id in ids_to_remove {
            self.tankids.remove(&id);
        }
    }
}

/// Main struct that stores everything - tanks, shapes, bullets, walls etc.
/// Does not store information about which tank is the player.
struct Map {
    /// 0,0 is at the center of the map. this is the distance of the walls in x and y. actual size is thenfore double this
    map_size: (f64, f64),
    /// All the squares, triangles and pentagons on the map
    shapes: HashMap<u128, Shape>, // no need to find a specific shape, so no hashmap but just Vec<>
    /// maximum number of shapes on the map, maxes for each shape type will be derived from this
    shapes_max: usize,
    /// All the tanks on the map, including player, bots, bosses etc.
    tanks: HashMap<u128, Tank>, // hashmap because of quicker searching for the tank when it's bullet kills something
    /// All the things shot by tanks - bullets or drones. Projectiles that make other things (rocket laucher tank, factory tank) aren't supported
    bullets: HashMap<u128, Bullet>, // hashmap to easily iterate over all the tank's bullets, for example when the tank dies, or when there would be a shield that only blocks some tank's bullets (teams?)
    /// a Vec<> of all the different AIs on the map. Each AI controls some tanks, 
    tankais: Vec<TankAI>
}
impl Map {
    /// Makes an empty map. Does not add the player tank or anything else.
    fn new() -> Self {
        Map {
            map_size: (5_000., 5_000.,),
            shapes_max: 500,
            shapes: HashMap::new(),
            tanks: HashMap::new(),
            bullets: HashMap::new(),
            tankais: vec![TankAI {
                tankids: HashSet::new(),
                range: 1000.,
                tg_range: 300.,
                bullet_speed: 1000.,
                fight_or_flight_threshold: 0.4,
                dodge_obstacles: true
            }]
        }
    }

    /// renders grid, walls, maybe more in the future
    fn render(&self, canvas: &mut Canvas<Window> , camera: &Camera) {
        for x in ((camera.x - 1./camera.zoom*camera.viewport_size.0 as f64).floor() as i32..(camera.x + 1./camera.zoom*camera.viewport_size.0 as f64).ceil() as i32).filter(|x| x%100 == 0) {
            canvas.set_draw_color(Color::GRAY);
            canvas.draw_line(Point::from(camera.to_screen_coords((x as f64, self.map_size.0))), Point::from(camera.to_screen_coords((x as f64, -self.map_size.0)))).expect("failed to draw line");
        }

        for y in ((camera.y - 1./camera.zoom*camera.viewport_size.1 as f64).floor() as i32..(camera.y + 1./camera.zoom*camera.viewport_size.1 as f64).ceil() as i32).filter(|y| y%100 == 0) {
            canvas.set_draw_color(Color::GRAY);
            canvas.draw_line(Point::from(camera.to_screen_coords((self.map_size.1, y as f64))), Point::from(camera.to_screen_coords((-self.map_size.1, y as f64)))).expect("failed to draw line");
        }
    }

    /// Finds the physics by u128 key, searches in tanks, bullets and shapes
    fn get_physics(&self, k: &u128) -> Option<&Physics> {
        if self.tanks.get(k).is_some() {
            Some(&self.tanks.get(k).unwrap().physics)
        } else if self.shapes.get(k).is_some() {
            Some(&self.shapes.get(k).unwrap().physics)
        } else if self.bullets.get(k).is_some() {
            Some(&self.bullets.get(k).unwrap().physics)
        } else {
            panic!()
        }
    }

    /// Finds the physics by u128 key, searches in tanks, bullets and shapes
    fn get_physics_mut(&mut self, k: &u128) -> Option<&mut Physics> {
        if self.shapes.get(k).is_some() {
            Some(&mut self.shapes.get_mut(k).unwrap().physics)
        } else if self.bullets.get(k).is_some() {
            Some(&mut self.bullets.get_mut(k).unwrap().physics)
        } else {
            Some(&mut self.tanks.get_mut(k).unwrap().physics)
        }
    }

    /// This contains a lot of things, and is called every frame. Includes velocity/position calculations, removing slow bullets, spawns shapes, regens health, more in the future
    /// 
    /// Updates the positions based on velocities of all objects, and slows down velocities by frincion/resistance
    /// 
    /// Updates times to realod for all turrets (only tank turrets now)
    /// 
    /// Removes all bullets s.t. the bullet speed is below it's dead speed
    /// 
    /// Randomly spawns shapes
    /// 
    fn update_physics(&mut self, delta: f64) {
        // things that happen for one (uprate physics, wall collision)
        {
            // mutable iterator over the physics' of all tanks, bullets and shapes
            let combined_iter_mut = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
            .chain(self.bullets.iter_mut().map(|bullet| &mut bullet.1.physics))
            .chain(self.shapes.iter_mut().map(|shape| &mut shape.1.physics));

            for o in combined_iter_mut {
                o.update(delta);
                if o.x.abs() > self.map_size.0 {
                    o.push(((self.map_size.0*o.x.signum() - o.x)*delta*1000., 0.));
                    o.xvel *= (-(delta)*100. / o.weight).exp();
                    o.yvel *= (-(delta)*100. / o.weight).exp();
                }
                if o.y.abs() > self.map_size.1 {
                    o.push((0., (self.map_size.1*o.y.signum() - o.y)*delta*1000.));
                    o.xvel *= (-(delta)*100. / o.weight).exp();
                    o.yvel *= (-(delta)*100. / o.weight).exp();
                }
            }

            // remove <0 hp objects
            self.tanks.retain(|_, v| v.physics.hp > 0.);
            self.bullets.retain(|_, v| v.physics.hp > 0.);

            // for shapes, hexagons must be removed differently because they split into squares

            // find hexes that died
            let mut hex_to_remove = Vec::new();
            for (id, shape) in self.shapes.iter() {
                if shape.texture == "hexagon" && shape.physics.hp <= 0. {
                    hex_to_remove.push(id.clone());
                }
            }

            for id in hex_to_remove {
                // handle dead hexes here
                for _ in 0..6 {
                    let mut s_physics = self.shapes.get(&id).unwrap().physics.clone();
                    let size = (thread_rng().gen::<f64>()*0.4+0.8) * s_physics.collision_size/20./4.;
                    s_physics.collision_size = 20.*size;
                    s_physics.weight = 100. * size.powi(2);
                    s_physics.max_hp = 10. * size.powi(2);
                    s_physics.hp_regen = 1. * size.powi(2);
                    s_physics.hp = s_physics.max_hp;
                    s_physics.x += (thread_rng().gen::<f64>()-0.5) * size * 140.;
                    s_physics.y += (thread_rng().gen::<f64>()-0.5) * size * 140.;
                    self.shapes.insert(thread_rng().gen(), Shape {
                        physics: s_physics,
                        texture: "square".to_owned(),
                    });
                }
            }

            // remove all dead shapes now
            self.shapes.retain(|_, v| v.physics.hp > 0.);
        }
        // things that happen for pairs, only one is mutable (collisions)
        {
            // (key, physics) pairs
            let iter_all = self.bullets.iter().map(|(k, v)| (k, v.physics)).chain(self.shapes.iter().map(|(k, v)| (k, v.physics))).chain(self.tanks.iter().map(|(k, v)| (k, v.physics)));

            // (key, x_position) pairs
            let minx_iter = iter_all.clone().map(|(k, v)| (k, v.x - v.collision_size));
            let maxx_iter = iter_all.map(|(k, v)| (k, v.x + v.collision_size));

            // all the minx and maxx values of all objects are here, min is value.2==true and max is false
            let mut all_vec: Vec<(u128, f64, bool)> = Vec::new();
            for x in minx_iter {
                all_vec.push((*x.0, x.1, true));
            }
            for x in maxx_iter {
                all_vec.push((*x.0, x.1, false));
            }

            // sort the vector, unstable is a bit faster and stable sorting is not needed here
            all_vec.sort_unstable_by(|(_, v1, _), (_, v2, _)| v1.partial_cmp(v2).unwrap());

            // hashset instead of vec for faster searching. Hashset is like a Hashmap with only the keys
            let mut active: HashSet<u128> = HashSet::new();

            for (k, x, b) in all_vec {
                if b {
                    // perform a collision check between the added object and all active objects (both ways)
                    for a in active.iter() {
                        if self.get_physics(&k).unwrap().collides(self.get_physics(a).unwrap()) {
                            // active object physics
                            let ap = (self.get_physics(a).unwrap()).clone();
                            // key object physics
                            let mut kp = (self.get_physics(&k).unwrap()).clone();

                            self.get_physics_mut(&k).unwrap().collide(&ap, delta);                       
                            self.get_physics_mut(&a).unwrap().collide(&mut kp, delta);
                        }
                    }
                    // add the object at the end to prevent collision with itself
                    active.insert(k);
                } else {
                    active.remove(&k);
                }
            }



            
        }

        for tank in self.tanks.values_mut() {
            for turret in &mut tank.turrets {
                // substracts delta from time to next shot, but doesn't go below zero
                turret.time_to_next_shot -= turret.time_to_next_shot.min(delta);
            }
        }

        // spawn shapes
        while self.shapes.len() < self.shapes_max {
            // from 0.6 to 1.4, squared 0.36 to 1.96
            let mut size = thread_rng().gen::<f64>() * 0.8 + 0.6;
            let is_hexagon = thread_rng().gen_bool(0.1);
            let is_triangle = thread_rng().gen_bool(0.4);
            if is_hexagon {
                size *= 4.;
            }

            if is_triangle && !is_hexagon {
                size *= 1.2;
            }
            
            self.shapes.insert(thread_rng().gen::<u128>(), Shape {
                physics: Physics {
                    x: thread_rng().gen_range(-self.map_size.0..self.map_size.0),
                    y: thread_rng().gen_range(-self.map_size.1..self.map_size.1),
                    xvel: 0.,
                    yvel: 0.,
                    weight: 100. * size.powi(2),
                    rot: thread_rng().gen::<f64>()*360.,
                    rotvel: 0.,
                    collision_size: 20. * size,
                    hp: 0.,
                    max_hp: if is_hexagon {
                        30. * size.powi(2)
                    } else if is_triangle{
                        3.3 * size.powi(2)
                    } else {
                        10. * size.powi(2)
                    },
                    hp_regen: if is_hexagon {
                        0.1 * size.powi(2)
                    } else if is_triangle{
                        2.5 * size.powi(2)
                    } else {
                        0.5 * size.powi(2)
                    },
                },
                texture: if is_hexagon {
                    "hexagon".to_owned()
                } else if is_triangle{
                    "triangle".to_owned()
                } else {
                    "square".to_owned()
                },
            });
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
    textures.insert("basic".to_owned(), texture_creator.load_texture("textures/basic.png").unwrap());
    textures.insert("bullet".to_owned(), texture_creator.load_texture("textures/bullet.png").unwrap());
    textures.insert("square".to_owned(), texture_creator.load_texture("textures/square.png").unwrap());
    textures.insert("hexagon".to_owned(), texture_creator.load_texture("textures/hexagon.png").unwrap());
    textures.insert("triangle".to_owned(), texture_creator.load_texture("textures/triangle.png").unwrap());

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
                rotvel: 0.,
                collision_size: 35.,
                hp: 1000.,
                max_hp: 1000.,
                hp_regen: 10.,
            },
            turrets: vec![Turret {
                /// its actually the force of impulse. should be about 100x the weight for normal speed
                projectile_speed: 1_000.,
                /// weight and hp should be similar. less weight = more bouncy, more weight = more penetration
                projectile_weight: 1.,
                projectile_collision_size: 10.,
                projectile_hp_regen: -0.5,
                /// also the max damage
                projectile_hp: 1.,
                /// should be >0.033 (30 shots per second), because more shots/second than fps makes glitches
                reload_time: 0.05,
                inaccuracy: 1.,
                relative_direction: 0.,
                time_to_next_shot: 0.
            }],
            power: 30000.,
            rot_power: 50000.,
            bullet_ids: vec![],
            texture: "basic".to_owned(),
        }
    );

    let ai_id = thread_rng().gen::<u128>();
    map.tankais[0].tankids.insert(ai_id);

    // add another tank, AI controlled
    // tanks will be network or AI controlled on the server (also player controlled on LAN multiplayer server), and player or AI controlled in singleplayer
    map.tanks.insert(
        ai_id,
        Tank {
            physics: Physics {
                x: 100.,
                y: 100.,
                xvel: 0.,
                yvel: 0.,
                weight: 100.,
                rot: 0.,
                rotvel: 0.,
                collision_size: 35.,
                hp: 100.,
                max_hp: 100.,
                hp_regen: 10.,
            },
            turrets: vec![Turret {
                /// its actually the force of impulse. should be about 100x the weight for normal speed
                projectile_speed: 1_000.,
                /// weight and hp should be similar. less weight = more bouncy, more weight = more penetration
                projectile_weight: 1.,
                projectile_collision_size: 10.,
                projectile_hp_regen: -0.5,
                /// also the max damage
                projectile_hp: 1.,
                /// should be >0.033 (30 shots per second), because more shots/second than fps makes glitches
                reload_time: 0.09,
                inaccuracy: 1.,
                relative_direction: 0.,
                time_to_next_shot: 0.
            }],
            power: 30000.,
            rot_power: 50000.,
            bullet_ids: vec![],
            texture: "basic".to_owned(),
        }
    );

    let mut last_frame_start;
    // How long the last frame took, in micros, 1 millisecond for the first frame
    let mut delta = 0.01;

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

        if map.tanks.get(&playerid).is_some() {
            //movement
            if input.up.is_down && input.left.is_down && !input.down.is_down && !input.right.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((-0.707,-0.707), delta);
            }
            else if input.down.is_down && input.left.is_down && !input.up.is_down && !input.right.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((-0.707,0.707), delta);
            }
            else if input.up.is_down && input.right.is_down && !input.down.is_down && !input.left.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.707,-0.707), delta);
            }
            else if input.down.is_down && input.right.is_down && !input.up.is_down && !input.left.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.707,0.707), delta);
            }
            else if input.up.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.,-1.), delta);
            }
            else if input.down.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.,1.), delta);
            }
            else if input.left.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((-1.,0.), delta);
            }
            else if input.right.is_down {
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((1.,0.), delta);
            }
            else {
                // brake
                map.tanks.get_mut(&playerid).unwrap().move_in_dir((0.,0.), delta);
            }

            //rotation
            map.tanks.get_mut(&playerid).unwrap().rotate_to(camera.to_map_coords(input.mouse_pos));

            //firing
            if input.fire.is_down {
                map.tanks.get_mut(&playerid).unwrap().fire(&mut map.bullets, playerid);
            }
        }

        // AI CONTROL

        for ai in &mut map.tankais {
            ai.control(&mut map.tanks, &mut map.shapes, &mut map.bullets, delta);
        }

        // PHYSICS

        // at the end of physics, update all physics
        map.update_physics(delta);


        // CAMERA

        // track tg tank if it exists, otherwise don't move
        if map.tanks.get(&camera.target_tank).is_some() {
            camera.track(delta, &map.tanks.get(&camera.target_tank).unwrap().physics);
        }


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

        // Render all shapes
        for bullet in &map.shapes {
            bullet.1.render(&mut canvas, &mut camera, &textures);
        }

        // Render all tanks
        for tank in &map.tanks {
            tank.1.render(&mut canvas, &mut camera, &textures);
        }

        canvas.present();

        delta = Instant::now().duration_since(last_frame_start).as_secs_f64();
        println!("fps: {:.0}", 1./delta);
    }
}
