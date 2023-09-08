use rand::prelude::*;
use rand_distr::Distribution;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

extern crate lazy_static;

use lazy_static::lazy_static;

/// init
lazy_static! {
    /// Vec<> of all the tank classes.
    /// 
    /// Contains data in the following format: (`DefaultTank`, `EvolveTo`, `Cost`)
    /// 
    /// `DefaultTank` is the default values of the tank of this class, all at level 1.
    /// `EvolveTo` is a vec of names of all the classes a tank can evolve to from this class.
    /// `Cost` is how much it costs to evolve to (not from) the particular class.
    /// 
    pub static ref EVOLUTION_TREE: HashMap<String, (Tank, Vec<String>, f64)> = {
        let mut hash_set = HashMap::new();

        hash_set.insert("basic".to_string(), (
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
                    hp: 100.,
                    max_hp: 100.,
                    hp_regen: 2.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 2_000.,
                    projectile_weight: 2.,
                    projectile_collision_size: 10.,
                    projectile_hp_regen: -0.5,
                    projectile_hp: 2.,
                    reload_time: 0.2,
                    inaccuracy: 1.,
                    relative_direction: 0.,
                    time_to_next_shot: 0.
                }],
                power: 30000.,
                rot_power: 50000.,
                bullet_ids: vec![],
                texture: "basic".to_owned(),
                last_hit_id: 0,
                evolution: Evolution::new(),
                camera_zoom: 1.0
            }, vec![
                "double".to_string(),
                "long".to_string(),
                "wide".to_string()],
            0.
        ));

        hash_set.insert("long".to_string(), (
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
                    hp: 100.,
                    max_hp: 100.,
                    hp_regen: 2.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 12_000.,
                    projectile_weight: 9.,
                    projectile_collision_size: 12.,
                    projectile_hp_regen: -3.,
                    projectile_hp: 8.,
                    reload_time: 0.5,
                    inaccuracy: 0.,
                    relative_direction: 0.,
                    time_to_next_shot: 0.
                }],
                power: 25000.,
                rot_power: 60000.,
                bullet_ids: vec![],
                texture: "long".to_owned(),
                last_hit_id: 0,
                evolution: Evolution::new(),
                camera_zoom: 0.7
            }, vec![
                "very long".to_string()],
            0.
        ));

        // Add more entries here using hash_set.insert() as needed

        hash_set
    };
}

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
/// returns vector with lenght `1.0`, or `(0.0, 0.0)` if the input is `(0.0, 0.0)`
fn normalize(v: (f64, f64)) -> (f64, f64) {
    let magnitude = (v.0 * v.0 + v.1 * v.1).sqrt();
    if magnitude == 0. {
        return (0.,0.);
    } else {
        return (v.0 / magnitude, v.1 / magnitude);
    }
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
    hp_regen: f64
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
        let rendersize = (self.physics.collision_size*4.*camera.zoom) as u32;
        let texture = textures.get(&self.texture).unwrap();
        let shape_screen_pos = camera.to_screen_coords((self.physics.x, self.physics.y));

        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(shape_screen_pos), // set center position
                rendersize, rendersize,  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((rendersize as i32 / 2, rendersize as i32 / 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
        
        // render health bar
        if self.physics.hp < self.physics.max_hp {
            canvas.set_draw_color(Color::RGB(63,63,63));
            canvas.draw_line((shape_screen_pos.0 - (50. * camera.zoom) as i32, shape_screen_pos.1 - (60. * camera.zoom) as i32), (shape_screen_pos.0 + (50. * camera.zoom) as i32, shape_screen_pos.1 - (60. * camera.zoom) as i32)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((shape_screen_pos.0 - (50. * camera.zoom) as i32, shape_screen_pos.1 - (60. * camera.zoom) as i32), (shape_screen_pos.0 - (50. * camera.zoom) as i32  + (self.physics.hp/self.physics.max_hp*100.*camera.zoom) as i32, shape_screen_pos.1 - (60. * camera.zoom) as i32)).unwrap();
        }
    }
}

/// Turrets can now only shoot bullets, will change later
#[derive(Clone)]
struct Turret {
    /// should be about 1000x the weight for normal speed
    projectile_impulse: f64,
    /// weight and hp should be similar. less weight = more bouncy, more weight = more penetration
    projectile_weight: f64,
    projectile_collision_size: f64,
    projectile_hp_regen: f64,
    /// also the max damage
    projectile_hp: f64,
    /// in micros, first shot is immediatae
    /// 
    /// should be >0.033 (30 shots per second), because more shots/second than fps makes glitches
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
                random_speed = self.projectile_impulse*(1. + 0.01*normal.sample(&mut thread_rng()));
                random_direction = normal.sample(&mut thread_rng());
            } else {
                random_speed = self.projectile_impulse;
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

/// Stores `xp`, upgraded levels, and tank class. Has functions for upgrading levels and promoting to higher classes. 
#[derive(Clone)]
struct Evolution {
    xp: f64,
    class: String,
}
impl Evolution {
    fn new() -> Self {
        Evolution {
            xp: 1000000.,
            class: "basic".to_string(),
        }
    }

    /// Promotes a tank to a class. Does not check whether the tank can promote to this class.
    /// 
    /// Does not take `&self`, because the evolution information is contained in the `&mut Tank` it takes
    fn promote(tank: &mut Tank, class: String) {
        let old_tank = tank.clone();
        *tank = EVOLUTION_TREE.get(&class).unwrap().0.clone();

        let ev = &mut tank.evolution;
        let ph = &mut tank.physics;
        ev.class = class;
        ev.xp = old_tank.evolution.xp;
        tank.last_hit_id = old_tank.last_hit_id;
        ph.x = old_tank.physics.x;
        ph.y = old_tank.physics.y;
        ph.xvel = old_tank.physics.xvel;
        ph.yvel = old_tank.physics.yvel;
        ph.rot = old_tank.physics.rot;
        ph.rotvel = old_tank.physics.rotvel;
        ph.hp = old_tank.physics.hp;
    }
}

/// A tank. Player, bot, boss etc
#[derive(Clone)]
pub struct Tank {
    physics: Physics,
    /// How much power the tank can apply to it's movement. Will move faster with more power, but slower if it weights more.
    power: f64,
    /// How much power the tank can apply to it's rotation movement. Will rotate faster with more power, but slower if it weights more.
    rot_power: f64,
    turrets: Vec<Turret>,
    bullet_ids: Vec<u128>,
    texture: String,
    /// id of the source of the last bullet that hit this tank. Useful for assiging the kill to a tank, even if the final damage was for example a collision with a shape.
    last_hit_id: u128,
    /// contains all the upgrading and evolution related variables and functions
    evolution: Evolution,
    camera_zoom: f64
}
impl Tank {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let rendersize = (self.physics.collision_size*4.*camera.zoom) as u32;
        let texture = textures.get(&self.texture).unwrap();
        let tank_screen_pos = camera.to_screen_coords((self.physics.x, self.physics.y));

        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(tank_screen_pos), // set center position
                rendersize, rendersize,  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((rendersize as i32 / 2, rendersize as i32 / 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
        
        // render health bar
        if self.physics.hp < self.physics.max_hp {
            canvas.set_draw_color(Color::RGB(63,63,63));
            canvas.draw_line((tank_screen_pos.0 - (50. * camera.zoom) as i32, tank_screen_pos.1 - (60. * camera.zoom) as i32), (tank_screen_pos.0 + (50. * camera.zoom) as i32, tank_screen_pos.1 - (60. * camera.zoom) as i32)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((tank_screen_pos.0 - (50. * camera.zoom) as i32, tank_screen_pos.1 - (60. * camera.zoom) as i32), (tank_screen_pos.0 - (50. * camera.zoom) as i32  + (self.physics.hp/self.physics.max_hp*100.*camera.zoom) as i32, tank_screen_pos.1 - (60. * camera.zoom) as i32)).unwrap();
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
        let x = (coords.0 as f64) / self.zoom + self.x - (self.viewport_size.0 as f64 / 2. / self.zoom);
        let y = (coords.1 as f64) / self.zoom + self.y - (self.viewport_size.1 as f64 / 2. / self.zoom);
        (x, y)
    }

    /// Converts from map to screen coordinates
    fn to_screen_coords(&self, coords: (f64, f64)) -> (i32, i32) {
        let x = ((coords.0 - self.x + (self.viewport_size.0 as f64 / 2. / self.zoom) as f64) * self.zoom) as i32;
        let y = ((coords.1 - self.y + (self.viewport_size.1 as f64 / 2. / self.zoom) as f64) * self.zoom) as i32;
        (x, y)
    }

    fn track(&mut self, _delta: f64, tg: &Physics) {
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
        let rendersize = (self.physics.collision_size*4.*camera.zoom) as u32;
        let texture = textures.get(&self.texture).unwrap();
        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(camera.to_screen_coords((self.physics.x, self.physics.y))), // set center position
                rendersize, rendersize,  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((rendersize as i32 / 2, rendersize as i32 / 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
    }
}

/// Tracks info about a button, like if it is pressed, and what keycode or mouse button activates it
#[derive(Clone, Copy)]
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
/// 
/// By pressing E (stands for Evolve, Evolution) you display evolution information. You can still fire using the mouse, and move with wsad.
/// Pressing E again will hide the information. The info is only visual, and it has info like: [HP: lvl 3, 130 hp, press '1' to upgrade]. You can still evolve without the menu if you know the keys.
/// 
/// Upgrading levels can be done by pressing the number keys (on the number row, not numpad). Promoting classes is done by Left Shift + number key. 
/// 
/// More info in `Evolution` struct
struct Input {
    up: Button,
    down: Button,
    left: Button,
    right: Button,
    fire: Button,

    // u stands for upgrade. these are keys used for upgrading (or promoting when used with shift)
    u1: Button,
    u2: Button,
    u3: Button,
    u4: Button,
    u5: Button,
    u6: Button,
    u7: Button,
    u8: Button,
    u9: Button,
    u0: Button,

    shift: Button,

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

            u1: Button { keycode: Some(Keycode::Num1), mousebutton: None, is_down: false, just: false },
            u2: Button { keycode: Some(Keycode::Num2), mousebutton: None, is_down: false, just: false },
            u3: Button { keycode: Some(Keycode::Num3), mousebutton: None, is_down: false, just: false },
            u4: Button { keycode: Some(Keycode::Num4), mousebutton: None, is_down: false, just: false },
            u5: Button { keycode: Some(Keycode::Num5), mousebutton: None, is_down: false, just: false },
            u6: Button { keycode: Some(Keycode::Num6), mousebutton: None, is_down: false, just: false },
            u7: Button { keycode: Some(Keycode::Num7), mousebutton: None, is_down: false, just: false },
            u8: Button { keycode: Some(Keycode::Num8), mousebutton: None, is_down: false, just: false },
            u9: Button { keycode: Some(Keycode::Num9), mousebutton: None, is_down: false, just: false },
            u0: Button { keycode: Some(Keycode::Num0), mousebutton: None, is_down: false, just: false },

            shift: Button { keycode: Some(Keycode::LShift), mousebutton: None, is_down: false, just: false },

            mouse_pos: (0,0),
            mouse_delta: (0,0), 
        }
    }

    /// Finds what this keycode means (up, down, fire, ..) and updates the respective state
    fn register_keydown(&mut self, keycode: Keycode) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift].iter_mut() {
            b.just = false;
        }
    }
}

/// This is an AI for controlling a tank. set `id` to the id of the tank you want to control
/// 
/// Make sure no tank is controlled by multiple AIs.
struct TankAI {
    /// ID of the tank this AI controls
    id: u128,
    /// how far away the targets (tanks and shapes) must be for the tank to attack or retreat. It will ignore things outside of range entirely, like it does not see them.
    range: f64,
    /// the range at which the tank tries to be from it's target. It will move closer of further accordingly. Set to 0. for smasher tanks to make them try to collide with enemies
    tg_range: f64,
    /// how fast the bullets are, used for aiming. projectile_impulse/projectile_weight of the turret
    bullet_speed: f64,
    /// starts fighting when its health gets above this
    fight_threshold: f64,
    /// starts escaping away when its health below above this
    flight_threshold: f64,
    /// is the tank currently fighting, if false it is flighting
    fighting: bool,
    /// set to `true` to make the tank dodge obstacles. Usually its good to turn on, besides tanks like smasher that do a lot of damage by colliding
    dodge_obstacles: bool,
    /// hashmap of tankid, target (only another). When attacking, it will keep the target unless it gets very far or dies. When retreating, it is usually the closest/biggest threat
    tg_id: u128,
}
impl TankAI {
    /// Controls all the tanks in it's `tankids` - makes them move and shoot based on `Map`
    ///
    /// Needs to access the whole `Map` mutably to modify the tanks it controls.
    /// 
    fn control(&mut self, tanks: &mut HashMap<u128, Tank>, shapes: &mut HashMap<u128, Shape>, mut bullets: &mut HashMap<u128, Bullet>, delta: f64) -> bool {

        
        let id = self.id;

        if tanks.contains_key(&id) {
            let con_tankp = tanks.get(&id).unwrap().physics;
            let mut movedir = (0.,0.);

            // switch between fight and flight. happens at the end of the frame intentianaly
            if (con_tankp.hp / con_tankp.max_hp) > self.fight_threshold {
                self.fighting = true
            } else if (con_tankp.hp / con_tankp.max_hp) < self.flight_threshold {
                self.fighting = false;
            }

            // attack the tank that last hit the controlled tank, if it is in range
            if tanks.contains_key(&tanks.get(&id).unwrap().last_hit_id) && tanks.get(&tanks.get(&id).unwrap().last_hit_id).unwrap().physics.dist(&con_tankp) < self.range {
                self.tg_id = tanks.get(&id).unwrap().last_hit_id;
            }

            // if eighter the tank does not have a target, or it is retreating, find new target. It finds new target when retreating to always retreat from the closest tank
            if !self.fighting || !tanks.contains_key(&self.tg_id) {

                // search for nearest tank
                let mut closest_id = 0_u128;
                let mut closest_dist = self.range;
                for (oid, tank) in tanks.iter() {
                    if tank.physics.dist(&tanks.get(&id).unwrap().physics) < closest_dist && id != *oid {
                        closest_dist = tank.physics.dist(&tanks.get(&id).unwrap().physics);
                        closest_id = *oid;
                    }
                }
                // if nearest tank is in range, set target to this tank.
                if closest_dist < self.range {
                    self.tg_id = closest_id;
                // else if the last hit tank is in range
                }
                // no target was found to attack or retreat from, attack shapes
                else {
                    // search for nearest shape
                    let mut closest_id = 0_u128;
                    let mut closest_dist = self.range;
                    for (oid, shape) in shapes.iter() {
                        if shape.physics.dist(&con_tankp) < closest_dist {
                            closest_dist = shape.physics.dist(&tanks.get(&id).unwrap().physics);
                            closest_id = *oid;
                        }
                    }

                    // if it found a shape, attack and chase it
                    if closest_id != 0 {
                        let clo_shapep = shapes.get(&closest_id).unwrap().physics;
                        let tg_pos = (clo_shapep.x, clo_shapep.y);
                        tanks.get_mut(&id).unwrap().rotate_to((tg_pos.0, tg_pos.1));
                        tanks.get_mut(&id).unwrap().fire(&mut bullets, id);
                        
                        // movedir is set to a very low value, so it is easily overriden by the obstacle avoiding algorithm, to prevent tanks from colliding with low hp shapes when farming shapes
                        movedir = normalize((tg_pos.0 - con_tankp.x, tg_pos.1 - con_tankp.y));
                        movedir = (movedir.0 * 0.01, movedir.1 * 0.01)
                    }
                }
            }

            // does the target exist in the tanks hashmap
            if tanks.contains_key(&self.tg_id) {
                // tank has a target

                // target physics and dist
                let tgp = tanks.get(&self.tg_id).unwrap().physics;
                let tg_dist = con_tankp.dist(&tgp);

                // check if target got out of range
                if tg_dist > self.range {
                    // remove target and continue with next controlled tank
                    self.tg_id = 0;
                    return true
                }

                let tg_pos = (tgp.x, tgp.y);
                // not the actual target velocity, but a vector of how much in front of the tank to fire to hit it properly, which depends on tg velocity, distance and bullet speed
                let tg_vel = ((tgp.xvel - con_tankp.xvel) * ((tg_dist/(0.6 * self.bullet_speed)).exp()) / 3.0, (tgp.yvel - con_tankp.yvel) * ((tg_dist/(0.6 * self.bullet_speed)).exp()) / 3.0);

                // move
                if self.fighting && con_tankp.dist(&tgp) > self.tg_range*1.2 {
                    // move towards
                    movedir = normalize((tg_pos.0 + tg_vel.0 - con_tankp.x, tg_pos.1 + tg_vel.1 - con_tankp.y));
                } else if !self.fighting || con_tankp.dist(&tgp) < self.tg_range*0.8 {
                    // move away
                    movedir = normalize((-(tg_pos.0 + tg_vel.0 - con_tankp.x), -(tg_pos.1 + tg_vel.1 - con_tankp.y)));
                } else {
                    // the tank is at optimal distance, it will now avoid obstacles but at a higher radius
                    if self.dodge_obstacles {
                        for sp in shapes.iter().map(|s| s.1.physics) {
                            // if the shape is close
                            if con_tankp.dist(&sp) < (sp.collision_size + con_tankp.collision_size)*10.  {
                                // move directly away from the shape, overriding the move direction determined before
                                let shape_away_dir = normalize((-(sp.x - con_tankp.x), -(sp.y - con_tankp.y)));
                                movedir = (movedir.0 + shape_away_dir.0*sp.hp, movedir.1 + shape_away_dir.1*sp.hp);

                            }
                        }
                    }
                }

                // attack target tank
                tanks.get_mut(&id).unwrap().rotate_to((tg_pos.0 + tg_vel.0, tg_pos.1 + tg_vel.1));
                tanks.get_mut(&id).unwrap().fire(&mut bullets, id);
                
            }

            // avoid obstacles
            if self.dodge_obstacles {
                for sp in shapes.iter().map(|s| s.1.physics) {
                    // if the shape is close (distance increases when the tank is going fast)
                    if con_tankp.dist(&sp) < (sp.collision_size + con_tankp.collision_size + con_tankp.speed()*0.5)*1.1  {
                        // move directly away from the shape, overriding the move direction determined before
                        let shape_away_dir = normalize((-(sp.x - con_tankp.x), -(sp.y - con_tankp.y)));
                        movedir = (movedir.0 + shape_away_dir.0*sp.hp/con_tankp.hp * 2., movedir.1 + shape_away_dir.1*sp.hp/con_tankp.hp * 2.);

                    }
                }
            }

            tanks.get_mut(&id).unwrap().move_in_dir(movedir, delta);  

        } else {
            // Tank with id 'id' is not in 'tanks', it appearently died. Remove from list of controlled tanks
            return false;
        }
        return true;
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
        if self.tanks.contains_key(k) {
            Some(&self.tanks.get(k).unwrap().physics)
        } else if self.shapes.contains_key(k) {
            Some(&self.shapes.get(k).unwrap().physics)
        } else if self.bullets.contains_key(k) {
            Some(&self.bullets.get(k).unwrap().physics)
        } else {
            panic!()
        }
    }

    /// Finds the physics by u128 key, searches in tanks, bullets and shapes
    fn get_physics_mut(&mut self, k: &u128) -> Option<&mut Physics> {
        if self.shapes.contains_key(k) {
            Some(&mut self.shapes.get_mut(k).unwrap().physics)
        } else if self.bullets.contains_key(k) {
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
            // mutable iterator over the physics' of all tanks and shapes (not bullets, as these are not affected by map boundnaries)
            let combined_iter_mut = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
            .chain(self.shapes.iter_mut().map(|shape| &mut shape.1.physics));

            for o in combined_iter_mut {
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

            // iter over all tanks, shapes, and bullets
            let combined_iter_mut = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
            .chain(self.shapes.iter_mut().map(|shape| &mut shape.1.physics))
            .chain(self.bullets.iter_mut().map(|bullet| &mut bullet.1.physics));

            for o in combined_iter_mut {
                o.update(delta);
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

            for (k, _x, b) in all_vec {
                if b {
                    // perform a collision check between the added object and all active objects (both ways)
                    for a in active.iter() {
                        if self.get_physics(&k).unwrap().collides(self.get_physics(a).unwrap()) {
                            // active object physics
                            let ap = (self.get_physics(a).unwrap()).clone();
                            // key object physics (the just added object)
                            let mut kp = (self.get_physics(&k).unwrap()).clone();

                            self.get_physics_mut(&k).unwrap().collide(&ap, delta);                       
                            self.get_physics_mut(&a).unwrap().collide(&mut kp, delta);

                            // if k is a tank, and a is a bullet
                            // set last hit to source tank
                            if self.tanks.contains_key(&k) && self.bullets.contains_key(&a) {
                                self.tanks.get_mut(&k).unwrap().last_hit_id = self.bullets.get_mut(&a).unwrap().source_tank_id;
                            } // other way around
                            else if self.tanks.contains_key(&a) && self.bullets.contains_key(&k) {
                                self.tanks.get_mut(&a).unwrap().last_hit_id = self.bullets.get_mut(&k).unwrap().source_tank_id;
                            }

                            // if k is a tank, and a is a tank
                            // set last hit
                            if self.tanks.contains_key(&k) && self.tanks.contains_key(&a) {
                                self.tanks.get_mut(&k).unwrap().last_hit_id = *a;
                                // other way around 
                                self.tanks.get_mut(&a).unwrap().last_hit_id = k;
                            }

                            // if k is a tank, and a is a shape
                            // add xp to the tank, if the shape hp is lees than 0 (it just died)
                            if self.tanks.contains_key(&k) && self.shapes.contains_key(&a) {
                                if self.shapes.get(&a).unwrap().physics.hp < 0. {
                                    self.tanks.get_mut(&k).unwrap().evolution.xp += self.shapes.get_mut(&a).unwrap().physics.collision_size.powi(2)*0.01;
                                }
                            } // other way around
                            else if self.tanks.contains_key(&a) && self.shapes.contains_key(&k) {
                                if self.shapes.get(&k).unwrap().physics.hp < 0. {
                                    self.tanks.get_mut(&a).unwrap().evolution.xp += self.shapes.get_mut(&k).unwrap().physics.collision_size.powi(2)*0.01;
                                }
                            }

                            // if k is a bullet, and a is a shape
                            // add xp to the source tank, if it exists, and if the shape hp is lees than 0 (it just died)
                            if self.bullets.contains_key(&k) && self.shapes.contains_key(&a) {
                                if self.tanks.contains_key(&self.bullets.get(&k).unwrap().source_tank_id) && self.shapes.get(&a).unwrap().physics.hp < 0.  {
                                    self.tanks.get_mut(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().evolution.xp += self.shapes.get_mut(&a).unwrap().physics.collision_size.powi(2)*0.01; 
                                }
                            } // other way around
                            else if self.bullets.contains_key(&a) && self.shapes.contains_key(&k) {
                                if self.tanks.contains_key(&self.bullets.get(&a).unwrap().source_tank_id) && self.shapes.get(&k).unwrap().physics.hp < 0. {
                                    self.tanks.get_mut(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().evolution.xp += self.shapes.get_mut(&k).unwrap().physics.collision_size.powi(2)*0.01;
                                }
                            }
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
            let shape_id = thread_rng().gen::<u128>();
            self.shapes.insert(shape_id, Shape {
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
    textures.insert("bullet".to_owned(), texture_creator.load_texture("textures/bullet.png").unwrap());
    textures.insert("square".to_owned(), texture_creator.load_texture("textures/square.png").unwrap());
    textures.insert("hexagon".to_owned(), texture_creator.load_texture("textures/hexagon.png").unwrap());
    textures.insert("triangle".to_owned(), texture_creator.load_texture("textures/triangle.png").unwrap());

    textures.insert("basic".to_owned(), texture_creator.load_texture("textures/basic.png").unwrap());
    textures.insert("long".to_owned(), texture_creator.load_texture("textures/long.png").unwrap());

    // Initialize my own things
    let mut map = Map {
        map_size: (2_000., 2_000.,),
        shapes_max: 500,
        shapes: HashMap::new(),
        tanks: HashMap::new(),
        bullets: HashMap::new(),
        tankais: vec![]
    };
    let mut input = Input::init();
    let playerid: u128 = rng.gen();
    let mut camera = Camera {
        x: 0.,
        y: 0.,
        zoom: 1.,
        target_tank: playerid,
        viewport_size: (1024, 1024)
    };

    // update the physics a little bit before anything else happens
    for x in 0..5000 {
        map.update_physics(0.1);
        if x%500 == 0 {
            println!("loading: {}%", x/50);
        }
    }

    // add player
    map.tanks.insert(
        playerid,
        EVOLUTION_TREE.get(&"basic".to_string()).unwrap().0.clone()
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

        // SPAWN TANKS

        while map.tanks.len() < 10 {
            let ai_tank_id = thread_rng().gen::<u128>();
            map.tankais.push(TankAI {
                id: ai_tank_id,
                range: 1000.,
                tg_range: 200.,
                bullet_speed: 1000.,
                fight_threshold: 0.95,
                flight_threshold: 0.4,
                dodge_obstacles: true,
                fighting: true,
                tg_id: 0,
            });
    
            // add another tank, AI controlled
            // tanks will be network or AI controlled on the server (also player controlled on LAN multiplayer server), and player or AI controlled in singleplayer
            map.tanks.insert(
                ai_tank_id,
                EVOLUTION_TREE.get(&"basic".to_string()).unwrap().0.clone()
            );
            let ph = &mut map.tanks.get_mut(&ai_tank_id).unwrap().physics;
            ph.x = thread_rng().gen::<f64>()*map.map_size.0*2. - map.map_size.0;
            ph.y = thread_rng().gen::<f64>()*map.map_size.1*2. - map.map_size.1;
        }

        // PLAYER CONTROL

        if map.tanks.contains_key(&playerid) {
            let player = map.tanks.get_mut(&playerid).unwrap();

            //movement
            if input.up.is_down && input.left.is_down && !input.down.is_down && !input.right.is_down {
                player.move_in_dir((-0.707,-0.707), delta);
            }
            else if input.down.is_down && input.left.is_down && !input.up.is_down && !input.right.is_down {
                player.move_in_dir((-0.707,0.707), delta);
            }
            else if input.up.is_down && input.right.is_down && !input.down.is_down && !input.left.is_down {
                player.move_in_dir((0.707,-0.707), delta);
            }
            else if input.down.is_down && input.right.is_down && !input.up.is_down && !input.left.is_down {
                player.move_in_dir((0.707,0.707), delta);
            }
            else if input.up.is_down {
                player.move_in_dir((0.,-1.), delta);
            }
            else if input.down.is_down {
                player.move_in_dir((0.,1.), delta);
            }
            else if input.left.is_down {
                player.move_in_dir((-1.,0.), delta);
            }
            else if input.right.is_down {
                player.move_in_dir((1.,0.), delta);
            }
            else {
                // brake
                player.move_in_dir((0.,0.), delta);
            }

            //rotation
            player.rotate_to(camera.to_map_coords(input.mouse_pos));

            //firing
            if input.fire.is_down {
                player.fire(&mut map.bullets, playerid);
            }

            // Evolution

            // promoting
            if input.shift.is_down {
                println!("Current class: {}", player.evolution.class);
                let classes = &EVOLUTION_TREE.get(&player.evolution.class).expect("this class does not exist in the evolution tree").1;
                for x in 0..classes.len() {
                    println!("Press {} to evolve to {:?}", x+1,  classes[x]);
                    let key = match x {
                        0 => input.u1,
                        1 => input.u2,
                        2 => input.u3,
                        3 => input.u4,
                        4 => input.u5,
                        5 => input.u6,
                        6 => input.u7,
                        7 => input.u8,
                        8 => input.u9,
                        _ => {eprintln!("Not enough keys on the number row to be able to evolve to all the tanks. A tank should be able to evolve to at most 9 other tanks"); panic!()}
                    };
                    if key.is_down && key.just {
                        Evolution::promote(player, classes[x].clone());
                    }
                }
            // upgrading levels
            } else {
                if input.u1.just && input.u1.is_down {
                    if player.evolution.xp > 100. {
                        player.physics.max_hp += 10.;
                        player.evolution.xp -= 100.;
                        println!("upgraded max_hp to {:.0}", player.physics.max_hp);
                    }
                    println!("xp: {:.0}", player.evolution.xp);
                }
                if input.u2.just && input.u2.is_down {
                    if player.evolution.xp > 100. {
                        player.physics.hp_regen += 0.2;
                        player.evolution.xp -= 100.;
                        println!("upgraded hp_regen to {:.2}", player.physics.hp_regen);
                    }
                    println!("xp: {:.0}", player.evolution.xp);
                }
                if input.u3.just && input.u3.is_down {
                    if player.evolution.xp > 100. {
                        player.turrets[0].reload_time *= 0.9;
                        player.evolution.xp -= 100.;
                        println!("upgraded to reload_time {:.4}", player.turrets[0].reload_time);
                    }
                    println!("xp: {:.0}", player.evolution.xp);
                }
            }
        }

        // AI CONTROL

        // this will call all the AIs' control functions, and keep only the AIs that return true
        map.tankais.retain_mut(|ai |ai.control(&mut map.tanks, &mut map.shapes, &mut map.bullets, delta));

        // PHYSICS

        // at the end of physics, update all physics
        map.update_physics(delta);


        // CAMERA

        // track tg tank if it exists, otherwise don't move
        if map.tanks.contains_key(&camera.target_tank) {
            camera.track(delta, &map.tanks.get(&camera.target_tank).unwrap().physics);
            camera.zoom = map.tanks.get(&camera.target_tank).unwrap().camera_zoom;
            // camera.zoom = 0.25;
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

        // TEST PRINTS
        // println!("fps: {:.0}", 1./delta);
        // println!("player_xp: {:.0}", map.tanks.get(&playerid).unwrap().evolution.xp);
    }
}
