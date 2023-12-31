use network::run_network;
use rand::prelude::*;
use rand_distr::Distribution;
use rand_distr::num_traits::Pow;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::libc::time;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::Window;
use tank_tree::EVOLUTION_TREE;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::{fs, thread};
use std::net::{TcpListener, TcpStream};
use std::time::{Instant, self};
use std::error::Error;

mod tank_tree;
mod network;

const GAMEMODE: Gamemode = Gamemode::FFA;

#[derive(PartialEq, Eq)]
enum Gamemode {
    FFA,
    Survival,

}

/// From A to B, in radians
fn angle_diff(a: f64, b: f64) -> f64 {
    let mut diff = b - a;
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}

fn vector_diff(v1: (f64, f64), v2: (f64, f64)) -> (f64, f64) {
    (v2.0 - v1.0, v2.1 - v1.1)
}

fn vector_lenght(v: (f64, f64)) -> f64 {
    f64::sqrt(v.0.powi(2) + v.1.powi(2))
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
        let speed_diff = vector_lenght(vector_diff((b.xvel, b.yvel), (self.xvel, self.yvel)));
        let s = delta*(speed_diff+64.)*8.;
        self.xvel *= (-delta*4.).exp();
        self.yvel *= (-delta*4.).exp();
        let n = normalize((self.x - b.x, self.y - b.y));
        self.push((n.0*s*((b.weight.sqrt()+self.weight.sqrt())), n.1*s*((b.weight.sqrt()+self.weight.sqrt()))));
        self.hp -= (s/16.).min(b.hp);
        self.push_rot(delta*speed_diff*angle_diff(f64::atan2(b.x - self.x, b.y - self.y).to_degrees(), f64::atan2((b.x + b.xvel) - (self.x + self.xvel), (b.y + b.yvel) - (self.y + self.yvel)).to_degrees())/-1.);
        
    }

    /// same as collide, but does not affect HP
    fn collide_position_only(&mut self, b: &Physics, delta: f64) {
        let speed_diff = vector_lenght(vector_diff((b.xvel, b.yvel), (self.xvel, self.yvel)));
        let s = delta*(speed_diff+64.)*8.;
        self.xvel *= (-delta*4.).exp();
        self.yvel *= (-delta*4.).exp();
        let n = normalize((self.x - b.x, self.y - b.y));
        self.push((n.0*s*((b.weight.sqrt()+self.weight.sqrt())), n.1*s*((b.weight.sqrt()+self.weight.sqrt()))));
        self.push_rot(delta*speed_diff*angle_diff(f64::atan2(b.x - self.x, b.y - self.y).to_degrees(), f64::atan2((b.x + b.xvel) - (self.x + self.xvel), (b.y + b.yvel) - (self.y + self.yvel)).to_degrees())/-1.);
        
    }

     /// for mbombs
     fn stick_to(&mut self, b: &Physics, delta: f64) {
        let n = normalize((self.x - b.x, self.y - b.y));
        self.xvel *= (delta * 16.).exp();
        self.yvel *= (delta * 16.).exp();
        self.push((n.0 * delta * 65536., n.1 * delta * 65536.));
        
    }

    /// Only checks if the object touch. For the collision to do anything use 'collide'
    fn collides(&self, b: &Physics) -> bool {
        (self.collision_size + b.collision_size) > self.dist(&b)
    }
}

/// Square, triangle, pentagon, 12gon
struct Shape {
    physics: Physics,
    /// Also affects behavour
    texture: String,
    /// Is true if the shape has not reached max hp in its lifetime yet. Will give no xp and will have increased hp regen (will be visually visible in some future update)
    just_spawned_mode: bool
}
impl Shape {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let rendersize = self.physics.collision_size*4.*camera.zoom*((camera.viewport_size.0.pow(2)+camera.viewport_size.1.pow(2)) as f64).sqrt()/1024.;
        let texture = &textures.get(&self.texture).unwrap();
        let shape_screen_pos = camera.to_screen_coords((self.physics.x, self.physics.y));

        canvas.copy_ex(
            &texture, None,
            Rect::from_center(
                Point::from(shape_screen_pos), // set center position
                rendersize as u32, rendersize as u32,  // set render width and height
            ),
            self.physics.rot, // set rotation
            Point::from((rendersize as i32 / 2, rendersize as i32 / 2)), // set center of rotation, in screen coordinates (not texture coordinates)
            false, false).unwrap();
        
        // render health bar, if not in just spawned mode
        if self.physics.hp < self.physics.max_hp && !self.just_spawned_mode {
            canvas.set_draw_color(Color::RGB(63,15,31));
            canvas.draw_line((shape_screen_pos.0 - (50. * rendersize / 266.) as i32, shape_screen_pos.1 - (60. * rendersize / 266.) as i32), (shape_screen_pos.0 + (50. * rendersize / 266.) as i32, shape_screen_pos.1 - (60. * rendersize / 266.) as i32)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((shape_screen_pos.0 - (50. * rendersize / 266.) as i32, shape_screen_pos.1 - (60. * rendersize / 266.) as i32), (shape_screen_pos.0 - (50. * rendersize / 266.) as i32  + (self.physics.hp/self.physics.max_hp*100. * rendersize / 266.) as i32, shape_screen_pos.1 - (60. * rendersize / 266.) as i32)).unwrap();
        }
    }
}

/// Turrets can now only shoot bullets, will change later
#[derive(Clone, Debug)]
struct Turret {
    /// should be about 1000x the weight for normal speed
    projectile_impulse: f64,
    /// weight and hp should be similar. less weight = more bouncy, more weight = more penetration
    projectile_weight: f64,
    projectile_collision_size: f64,
    projectile_hp_regen: f64,
    /// also the max damage
    projectile_hp: f64,
    /// affects projectile type, like bullet, bomb drone, trap etc
    projectile_texture: String,
    /// in micros, first shot is immediatae
    /// 
    /// should be >0.033 (30 shots per second), because more shots/second than fps makes glitches
    reload_time: f64,
    /// mean in degrees, gaussian propability distribution
    /// also randomizes projectile speed, at rate 1 degree = 1% speed
    inaccuracy: f64,
    /// in degrees, turret facing relative to tank facing
    relative_direction: f64,
    /// position where the bullet spawns
    relative_position: (f64, f64),

    // start of changing properties

    time_to_next_shot: f64
}
impl Default for Turret {
    fn default() -> Self {
        Self {
            projectile_impulse: 1000.,
            projectile_weight: 1.,
            projectile_collision_size: 10.,
            projectile_hp_regen: -1.,
            projectile_hp: 1.,
            projectile_texture: "bullet".to_string(),
            reload_time: 1.,
            inaccuracy: 0.,
            relative_direction: 0.,
            relative_position: (0.,-100.),
            time_to_next_shot: 0.,
        }
    }
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
            bullet_physics.x += (self.relative_direction+tank_physics.rot).to_radians().cos()*(self.relative_position.0);
            bullet_physics.x -= (self.relative_direction+tank_physics.rot).to_radians().sin()*(self.relative_position.1);
            bullet_physics.y += (self.relative_direction+tank_physics.rot).to_radians().sin()*(self.relative_position.0);
            bullet_physics.y += (self.relative_direction+tank_physics.rot).to_radians().cos()*(self.relative_position.1);
            bullet_physics.hp_regen = self.projectile_hp_regen;
            bullet_physics.hp = self.projectile_hp;
            bullet_physics.max_hp = self.projectile_hp;

            self.time_to_next_shot = self.reload_time;

            Some(Bullet {
                physics: bullet_physics,
                source_tank_id: tank_id,
                texture: self.projectile_texture.to_owned()
            })
        }
    }
}

/// Stores `xp`, upgraded levels, and tank class. Has functions for upgrading levels and promoting to higher classes.
/// 
/// Available upgrades (might change in the future): 1:max hp, 2:hp regeneration, 3:reload time, 4:projectile hp(projectile hp regen decreases accordingly), 5: movement speed(also affects rotation speed), 6: projectile speed(impulse)
/// 
/// Promoting to a higher class will delete all upgrades, will likely change in the future
#[derive(Clone, Debug)]
struct Evolution {
    xp: f64,
    class: String,
    hp_level: u8,
    regen_level: u8,
    reload_level: u8,
    damage_level: u8,
    speed_level: u8,
    bulletspeed_level: u8,
    /// how much xp is added to someone who kills this
    killvalue: f64,
}
impl Evolution {
    fn new() -> Self {
        Evolution {
            xp: 10000.,
            class: "basic".to_string(),
            hp_level: 0,
            regen_level: 0,
            reload_level: 0,
            damage_level: 0,
            speed_level: 0,
            bulletspeed_level: 0,
            killvalue: 0.
        }
    }

    fn add_xp(&mut self, xp: f64) {
        self.xp += xp*2.;
        // you get half the xp the tank earned within its lifetime for killing it
        self.killvalue += xp;
    }
    /// Promotes a tank to a class. Does not check whether the tank can promote to this class.
    /// 
    /// Does not take `&self`, because the evolution information is contained in the `&mut Tank` it takes
    fn promote(tank: &mut Tank, class: String) {
        if tank.evolution.xp >= EVOLUTION_TREE.get(&class).unwrap().2 {
            tank.evolution.xp -= EVOLUTION_TREE.get(&class).unwrap().2;

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

            ev.hp_level = old_tank.evolution.hp_level;
            ev.regen_level = old_tank.evolution.regen_level;
            ev.damage_level = old_tank.evolution.damage_level;
            ev.reload_level = old_tank.evolution.reload_level;
            ev.speed_level = old_tank.evolution.speed_level;
            ev.bulletspeed_level = old_tank.evolution.bulletspeed_level;
            Evolution::level_refresh(tank);
        } else {
            println!("Not enough xp to promote. You need {} xp", EVOLUTION_TREE.get(&class).unwrap().2);
        }
    }

    /// Makes the physical properties (hp, power, hp_regen etc.) of the tank match it's class and levels. Always call after changing a level or a class.
    fn level_refresh(tank: &mut Tank) {

        // the default tank for this class
        let default_tank = EVOLUTION_TREE.get(&tank.evolution.class).unwrap().0.clone();

        // set all the upgradable values to default for the class
        tank.physics.max_hp = default_tank.physics.max_hp;
        tank.physics.collision_size = default_tank.physics.collision_size;
        tank.physics.hp_regen = default_tank.physics.hp_regen;
        tank.power = default_tank.power;
        tank.rot_power = default_tank.rot_power;
        for x in 0..tank.turrets.len() {
            tank.turrets[x].projectile_hp = default_tank.turrets[x].projectile_hp;
            tank.turrets[x].projectile_hp_regen = default_tank.turrets[x].projectile_hp_regen;
            tank.turrets[x].projectile_impulse = default_tank.turrets[x].projectile_impulse;
            tank.turrets[x].projectile_weight = default_tank.turrets[x].projectile_weight;
            tank.turrets[x].reload_time = default_tank.turrets[x].reload_time;
        }

        for l in 0..tank.evolution.hp_level.min(10) {
            tank.physics.max_hp *= 1. + 0.15 * (1.6-0.1*l as f64);
            tank.physics.collision_size *= 1. + 0.01 * (1.6-0.1*l as f64);
        }
    
        for l in 0..tank.evolution.regen_level.min(10) {
            tank.physics.hp_regen *= 1. + 0.13 * (1.6-0.1*l as f64);
        }
    
        for l in 0..tank.evolution.reload_level.min(10) {
            for x in 0..tank.turrets.len() {
                tank.turrets[x].reload_time *= 1. - 0.10 * (1.6-0.1*l as f64);
            }
        }
    
        // this increases projectile_impulse, projectile_weight, projectile_hp and projectile_hp_regen all by the same coefficient
        for l in 0..tank.evolution.damage_level.min(10) {
            for x in 0..tank.turrets.len() {
                tank.turrets[x].projectile_hp *= 1. + 0.08 * (1.6-0.1*l as f64);
                tank.turrets[x].projectile_hp_regen *= 1. + 0.08 * (1.6-0.1*l as f64);
                tank.turrets[x].projectile_impulse *= 1. + 0.08 * (1.6-0.1*l as f64);
                tank.turrets[x].projectile_weight *= 1. + 0.08 * (1.6-0.1*l as f64);
            }
        }
    
        for l in 0..tank.evolution.speed_level.min(10) {
            tank.power *= 1. + 0.08 * (1.6-0.1*l as f64);
            tank.rot_power *= 1. + 0.08 * (1.6-0.1*l as f64);
        }
    
        for l in 0..tank.evolution.bulletspeed_level.min(10) {
            for x in 0..tank.turrets.len() {
                tank.turrets[x].projectile_impulse *= 1. + 0.07 * (1.6-0.1*l as f64);
            }
        }
    }
}

/// A tank. Player, bot, boss etc
#[derive(Clone, Debug)]
pub struct Tank {
    physics: Physics,
    /// How much power the tank can apply to it's movement. Will move faster with more power, but slower if it weights more.
    power: f64,
    /// How much power the tank can apply to it's rotation movement. Will rotate faster with more power, but slower if it weights more.
    rot_power: f64,
    turrets: Vec<Turret>,
    bullet_ids: HashSet<u128>,
    texture: String,
    /// id of the source of the last bullet that hit this tank. Useful for assiging the kill to a tank, even if the final damage was for example a collision with a shape.
    last_hit_id: u128,
    /// contains all the upgrading and evolution related variables and functions
    evolution: Evolution,
    firing_to: (f64, f64),
}
impl Default for Tank {
    /// BASIC tank, might not be updated with latest changed to BASIC
    fn default() -> Self {
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
                hp: 120.,
                max_hp: 120.,
                hp_regen: 4.,
            },
            turrets: vec![Turret {
                projectile_impulse: 3_000.,
                projectile_weight: 3.,
                projectile_collision_size: 12.,
                projectile_hp_regen: -0.5,
                projectile_hp: 3.,
                reload_time: 0.3,
                inaccuracy: 1.,
                relative_position: (0.,-52.),
                ..Default::default()
            }],
            power: 30000.,
            rot_power: 450.,
            texture: "basic".to_owned(),
            bullet_ids: HashSet::new(),
            evolution: Evolution::new(),
            last_hit_id: 0,
            firing_to: (0.,0.)
            
        }
    }
}
impl Tank {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let rendersize = (self.physics.collision_size*8.*camera.zoom*((camera.viewport_size.0.pow(2)+camera.viewport_size.1.pow(2)) as f64).sqrt()/1024.) as u32;
        let texture = textures.get(&self.texture).expect(&format!{"failed to load texture: {}", &self.texture});
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

        canvas.filled_circle(tank_screen_pos.0 as i16, tank_screen_pos.1 as i16, (rendersize as f64/8.).ceil() as i16, Color::RGB(0, 0, 255)).unwrap();
        
        // render health bar
        if self.physics.hp < self.physics.max_hp {
            canvas.set_draw_color(Color::RGB(63,15,31));
            canvas.draw_line((tank_screen_pos.0 - (50 * rendersize / 266) as i32, tank_screen_pos.1 - (60 * rendersize / 266) as i32), (tank_screen_pos.0 + (50 * rendersize / 266) as i32, tank_screen_pos.1 - (60 * rendersize / 266) as i32)).unwrap();
            canvas.draw_line((tank_screen_pos.0 - (50 * rendersize / 266) as i32, tank_screen_pos.1-1 - (60 * rendersize / 266) as i32), (tank_screen_pos.0 + (50 * rendersize / 266) as i32, tank_screen_pos.1-1 - (60 * rendersize / 266) as i32)).unwrap();
            canvas.set_draw_color(Color::RGB(0,255,0));
            canvas.draw_line((tank_screen_pos.0 - (50 * rendersize / 266) as i32, tank_screen_pos.1 - (60 * rendersize / 266) as i32), (tank_screen_pos.0 - (50 * rendersize / 266) as i32  + (self.physics.hp/self.physics.max_hp*100. * rendersize as f64 / 266.) as i32, tank_screen_pos.1 - (60 * rendersize / 266) as i32)).unwrap();
            canvas.draw_line((tank_screen_pos.0 - (50 * rendersize / 266) as i32, tank_screen_pos.1-1 - (60 * rendersize / 266) as i32), (tank_screen_pos.0 - (50 * rendersize / 266) as i32  + (self.physics.hp/self.physics.max_hp*100. * rendersize as f64 / 266.) as i32, tank_screen_pos.1-1 - (60 * rendersize / 266) as i32)).unwrap();
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
    fn rotate_to(&mut self, to: (f64, f64), delta: f64) {
        let tg_angle = -f64::atan2(self.physics.x - to.0, self.physics.y - to.1).to_degrees();
        self.physics.push_rot(angle_diff(self.physics.rot, tg_angle).clamp(-25., 25.)*self.rot_power*delta*40.);
        self.physics.rotvel *= (-delta*10.).exp();
        self.firing_to = to;
    }

    /// Fires from all the tank's reloaded turrets
    /// Will make the bullets belong to `source_id` (for sake of eg. who did the kill)
    fn fire(&mut self, bullets: &mut HashMap<u128, Bullet>, source_id: u128) {
        for t in &mut self.turrets {
            let bullet = t.fire(&self.physics, source_id);
            if bullet.is_some() {
                let x:u128 = thread_rng().gen();
                self.bullet_ids.insert(x);
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
        let zoom = self.zoom * ((self.viewport_size.0.pow(2) + self.viewport_size.1.pow(2)) as f64).sqrt() / 1024.;
        let x = (coords.0 as f64) / zoom + self.x - (self.viewport_size.0 as f64 / 2. / zoom);
        let y = (coords.1 as f64) / zoom + self.y - (self.viewport_size.1 as f64 / 2. / zoom);
        (x, y)
    }

    /// Converts from map to screen coordinates
    fn to_screen_coords(&self, coords: (f64, f64)) -> (i32, i32) {
        let zoom = self.zoom * ((self.viewport_size.0.pow(2) + self.viewport_size.1.pow(2)) as f64).sqrt() / 1024.;
        let x = ((coords.0 - self.x + (self.viewport_size.0 as f64 / 2. / zoom) as f64) * zoom) as i32;
        let y = ((coords.1 - self.y + (self.viewport_size.1 as f64 / 2. / zoom) as f64) * zoom) as i32;
        (x, y)
    }

    /// x, y is in map coords
    fn visible(&self, (x,y) : (f64, f64), radius:f64) -> bool{
        let (x, y) = self.to_screen_coords((x, y));

        let radius = radius*4.*self.zoom*((self.viewport_size.0.pow(2)+self.viewport_size.1.pow(2)) as f64).sqrt()/1024.;

        x > -radius as i32 && x < self.viewport_size.0 + radius as i32 &&
        y > -radius as i32 && y < self.viewport_size.1 + radius as i32
    }

    fn track(&mut self, _delta: f64, tg: &Physics) {
        self.x = tg.x;
        self.y = tg.y;
    }
}

/// A bullet or a drone(controlled bullet)
#[derive(Debug, Clone)]
struct Bullet {
    physics: Physics,
    source_tank_id: u128,
    texture: String,
}
impl Bullet {
    fn render(&self, canvas: &mut Canvas<Window>, camera: &Camera, textures: &HashMap<String, Texture>) {
        let rendersize = (self.physics.collision_size*4.*camera.zoom*((camera.viewport_size.0.pow(2)+camera.viewport_size.1.pow(2)) as f64).sqrt()/1024.) as u32;
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
/// Releasing E will hide the information. The info is only visual, and it has info like: [HP: lvl 3, 130 hp, press '1' to upgrade]. You can still evolve without the menu if you know the keys.
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
    evolve: Button,

    zoom_in: Button,
    zoom_out: Button,

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

            zoom_in: Button { keycode: Some(Keycode::X), mousebutton: None, is_down: false, just: false },
            zoom_out: Button { keycode: Some(Keycode::Z), mousebutton: None, is_down: false, just: false },

            shift: Button { keycode: Some(Keycode::LShift), mousebutton: None, is_down: false, just: false },
            evolve: Button { keycode: Some(Keycode::E), mousebutton: None, is_down: false, just: false },

            mouse_pos: (0,0),
            mouse_delta: (0,0), 
        }
    }

    /// Finds what this keycode means (up, down, fire, ..) and updates the respective state
    fn register_keydown(&mut self, keycode: Keycode) {
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift, &mut self.evolve, &mut self.zoom_in, &mut self.zoom_out].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift, &mut self.evolve, &mut self.zoom_in, &mut self.zoom_out].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift, &mut self.evolve, &mut self.zoom_in, &mut self.zoom_out].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift, &mut self.evolve, &mut self.zoom_in, &mut self.zoom_out].iter_mut() {
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
        for b in [&mut self.up, &mut self.down, &mut self.left, &mut self.right, &mut self.fire, &mut self.u0, &mut self.u1, &mut self.u2, &mut self.u3, &mut self.u4, &mut self.u5, &mut self.u6, &mut self.u7, &mut self.u8, &mut self.u9, &mut self.shift, &mut self.evolve, &mut self.zoom_in, &mut self.zoom_out].iter_mut() {
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
    /// is the tank currently fighting, if false it is flighting
    fighting: bool,
    /// set to `true` to make the tank dodge obstacles. Usually its good to turn on, besides tanks like smasher that do a lot of damage by colliding
    dodge_obstacles: bool,
    next_upgrade_is_promotion: bool,
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
            let mut con_tank = &mut tanks.get_mut(&id).unwrap();
            let mut movedir = (0.,0.);

            self.tg_range = if con_tank.evolution.class == "shotgun" {
                128.
            } else {
                (con_tank.turrets[0].projectile_impulse/con_tank.turrets[0].projectile_weight).sqrt()  *  (con_tank.turrets[0].projectile_hp/-con_tank.turrets[0].projectile_hp_regen).sqrt()  *  8.
            };
            self.bullet_speed = con_tank.turrets[0].projectile_impulse/con_tank.turrets[0].projectile_weight;
            

            if con_tank.evolution.xp > if self.next_upgrade_is_promotion {1000.} else {100.} {
                // random bools
                let mut rb = [false;10];
                for b in 0..10 {
                    rb[b] = thread_rng().gen_bool(0.1);
                }
                if self.next_upgrade_is_promotion {
                    let classes = &EVOLUTION_TREE.get(&con_tank.evolution.class).expect("this class does not exist in the evolution tree").1;
                    for x in 0..classes.len() {
                        let key = match x {
                            0 => rb[1],
                            1 => rb[2],
                            2 => rb[3],
                            3 => rb[4],
                            4 => rb[5],
                            5 => rb[6],
                            6 => rb[7],
                            7 => rb[8],
                            8 => rb[9],
                            _ => {eprintln!("Not enough keys on the number row to be able to evolve to all the tanks. A tank should be able to evolve to at most 9 other tanks"); panic!()}
                        };
                        if key {
                            Evolution::promote(&mut con_tank, classes[x].clone());
                        }
                    }
                // upgrading levels
                } else {
                    rb[1] = thread_rng().gen_bool(0.05);
                    rb[2] = thread_rng().gen_bool(0.15);
                    rb[3] = thread_rng().gen_bool(0.45);
                    rb[4] = thread_rng().gen_bool(0.6);
                    rb[5] = thread_rng().gen_bool(0.2);
                    rb[6] = thread_rng().gen_bool(1.);
                    if rb[1] {
                        if con_tank.evolution.hp_level < 10 {
                            con_tank.evolution.hp_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    else if rb[2] {
                        if con_tank.evolution.regen_level < 10 {
                            con_tank.evolution.regen_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    else if rb[3] {
                        if con_tank.evolution.reload_level < 10 {
                            con_tank.evolution.reload_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    else if rb[4] {
                        if con_tank.evolution.damage_level < 10 {
                            con_tank.evolution.damage_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    else if rb[5] {
                        if con_tank.evolution.speed_level < 10 {
                            con_tank.evolution.speed_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    else if rb[6] {
                        if con_tank.evolution.bulletspeed_level < 10 {
                            con_tank.evolution.bulletspeed_level += 1;
                            con_tank.evolution.xp -= 100.;
                        }
                    }
                    Evolution::level_refresh(&mut con_tank);
                }
                self.next_upgrade_is_promotion = thread_rng().gen_bool(0.05);
            }

            // attack the tank that last hit the controlled tank, if it is in range
            if tanks.contains_key(&tanks.get(&id).unwrap().last_hit_id) && tanks.get(&tanks.get(&id).unwrap().last_hit_id).unwrap().physics.dist(&con_tankp) < self.range && tanks.get(&id).unwrap().last_hit_id != id {
                self.tg_id = tanks.get(&id).unwrap().last_hit_id;
                tanks.get_mut(&id).unwrap().last_hit_id = 0;
            }

            // if the tank does not have a target, find new target. Does not execute when the tank is attacking a tank back
            else if !tanks.contains_key(&self.tg_id) {

                // search for nearest tank
                let mut closest_id = 0_u128;
                let mut closest_dist = self.range;
                for (oid, tank) in tanks.iter() {
                    if tank.physics.dist(&tanks.get(&id).unwrap().physics) < closest_dist && id != *oid {
                        closest_dist = tank.physics.dist(&tanks.get(&id).unwrap().physics);
                        closest_id = *oid;
                    }
                }

                self.fighting = true;
                if tanks.contains_key(&closest_id) {
                    let clo_tankp = tanks.get(&closest_id).unwrap().physics;
                    // switch between fight and flight. depends on closest tank only. attacks only if the tank has at least 2x as much HP as enemy
                    if con_tankp.hp / clo_tankp.hp < 2. {
                        self.fighting = false
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
                let tg_vel = if tanks.get_mut(&id).unwrap().texture == "spawner" || tanks.get_mut(&id).unwrap().texture == "infector"  || tanks.get_mut(&id).unwrap().texture == "anthill"  || tanks.get_mut(&id).unwrap().texture == "trapspawner" {
                    (0.,0.)
                } else {
                    ((tgp.xvel - con_tankp.xvel) * ((tg_dist/(0.6 * self.bullet_speed)).exp()) / 5.0, (tgp.yvel - con_tankp.yvel) * ((tg_dist/(0.6 * self.bullet_speed)).exp()) / 5.0)
                };

                // move
                if self.fighting && (con_tankp.dist(&tgp) > self.tg_range*1.2) {
                    // move towards
                    movedir = normalize((tg_pos.0 + tg_vel.0 - con_tankp.x, tg_pos.1 + tg_vel.1 - con_tankp.y));
                } else if (!self.fighting) || (con_tankp.dist(&tgp) < self.tg_range*0.8) {
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
                tanks.get_mut(&id).unwrap().rotate_to((tg_pos.0 + tg_vel.0, tg_pos.1 + tg_vel.1), delta);
                tanks.get_mut(&id).unwrap().fire(&mut bullets, id);
                
            } else {
                self.tg_id = 0;

                // search for nearest shape
                let mut closest_id = 0_u128;
                let mut best_rating: f64 = 0.;
                for (oid, shape) in shapes.iter() {
                    if shape.physics.collision_size / (shape.physics.hp + con_tankp.dist(&shape.physics)) > best_rating && shape.texture != "12gon" && !shape.just_spawned_mode {
                        // favor shapes with lot of xp, short distance and low hp
                        best_rating = shape.physics.collision_size / (shape.physics.hp * 4. + shape.physics.hp_regen * 0.5 + con_tankp.dist(&shape.physics));
                        closest_id = *oid;
                    }
                }

                // if it found a shape, attack and chase it
                if closest_id != 0 {
                    let clo_shapep = shapes.get(&closest_id).unwrap().physics;
                    let tg_pos = (clo_shapep.x, clo_shapep.y);
                    tanks.get_mut(&id).unwrap().rotate_to((tg_pos.0, tg_pos.1), delta);
                    tanks.get_mut(&id).unwrap().fire(&mut bullets, id);
                    
                    // movedir is set to a very low value, so it is easily overriden by the obstacle avoiding algorithm, to prevent tanks from colliding with low hp shapes when farming shapes
                    movedir = normalize((tg_pos.0 - con_tankp.x, tg_pos.1 - con_tankp.y));
                    if con_tankp.hp + clo_shapep.hp > 0.8*con_tankp.max_hp {
                        movedir = (movedir.0 * 2., movedir.1 * 2.)
                    } else {
                        movedir = (movedir.0 * 0.01, movedir.1 * 0.01)
                    }
                }
            }

            // avoid obstacles
            if self.dodge_obstacles {
                for sp in shapes.iter().map(|s| s.1.physics) {
                    // if the shape is close (distance increases when the tank is going fast)
                    if con_tankp.dist(&sp) < (sp.collision_size + con_tankp.collision_size + con_tankp.speed()*0.5)*1.1  {
                        // move directly away from the shape, overriding the move direction determined before
                        let shape_away_dir = normalize((-(sp.x - con_tankp.x), -(sp.y - con_tankp.y)));
                        movedir = (movedir.0 + shape_away_dir.0*sp.hp/con_tankp.hp * 4., movedir.1 + shape_away_dir.1*sp.hp/con_tankp.hp * 4.);

                    }
                }
            }

            // avoid bullets
            if self.dodge_obstacles {
                for (bullet, source) in bullets.iter().map(|s| (s.1, s.1.source_tank_id)) {
                    // if the bullet is close (distance increases when the tank is going fast)
                    if con_tankp.dist(&bullet.physics) < (bullet.physics.collision_size + con_tankp.collision_size + bullet.physics.speed()*1. * if bullet.texture == "bomb" || bullet.texture == "mbomb" {1024.} else {1.}) && source != id  {
                        let bdist = con_tankp.dist(&bullet.physics);

                        // move directly away from the bullet, overriding the move direction determined before
                        let bullet_away_dir = normalize((-(bullet.physics.x + bullet.physics.xvel - con_tankp.x), -(bullet.physics.y + bullet.physics.yvel - con_tankp.y)));
                        if bullet.texture == "bomb" || bullet.texture == "mbomb" {
                            movedir = (movedir.0 + bullet_away_dir.0*bullet.physics.hp/con_tankp.hp /bdist * 131072., movedir.1 + bullet_away_dir.1*bullet.physics.hp/con_tankp.hp /bdist * 131072.);
                        } else {
                            movedir = (movedir.0 + bullet_away_dir.0*bullet.physics.hp/con_tankp.hp /bdist * 4., movedir.1 + bullet_away_dir.1*bullet.physics.hp/con_tankp.hp /bdist * 4.);
                        }

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

    /// Finds the physics by u128 key, searches in tanks, bullets and shapes.
    fn get_physics(&self, k: &u128) -> Option<&Physics> {
        if self.shapes.contains_key(k) {
            Some(&self.shapes.get(k)?.physics)
        } else if self.bullets.contains_key(k) {
            Some(&self.bullets.get(k)?.physics)
        } else {
            Some(&self.tanks.get(k)?.physics)
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

        self.tanks.retain(|_, v| v.physics.speed() <= 50_000.);
        self.shapes.retain(|_, v| v.physics.speed() <= 50_000.);

        // spawn shapes, max 16 per frame
        for _ in 0..((self.shapes_max as f64 - self.shapes.len() as f64) * delta).clamp(0.,16.) as usize {

            let (x, y) = (thread_rng().gen_range(-self.map_size.0..self.map_size.0), thread_rng().gen_range(-self.map_size.0..self.map_size.0));

            // from 0.8 to 1.2, squared 0.64 to 1.44
            let mut size = thread_rng().gen::<f64>() * 0.4 + 0.8;
            let mut is_hexagon = thread_rng().gen_bool(0.1);
            let is_triangle = thread_rng().gen_bool(0.5);
            let mut is_12gon = false;

            if is_hexagon || ((self.shapes_max - self.shapes.len()) > (self.shapes_max as f64 * 0.1) as usize) {
                if self.shapes.iter().filter(|i| i.1.texture == "12gon").count() < (self.shapes_max as f64 * 0.01) as usize {
                    size *= 3.;
                    is_12gon = true;
                    is_hexagon = true;
                }
            }

            if is_hexagon {
                size *= 4.;
            }

            if is_triangle && !is_hexagon {
                size *= 1.2;
            }
            let shape_id = thread_rng().gen::<u128>();
            self.shapes.insert(shape_id, Shape {
                physics: Physics {
                    x,
                    y,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 1.,
                    rot: thread_rng().gen::<f64>()*360.,
                    rotvel: 0.,
                    collision_size: 20. * size,
                    hp: 4.,
                    max_hp: if is_hexagon {
                        if is_12gon {
                            300. * size.powi(2)
                        } else {
                            30. * size.powi(2)
                        }
                    } else if is_triangle{
                        2.0 * size.powi(2)
                    } else {
                        10. * size.powi(2)
                    },
                    // hp regen is multiplied by 64, because it is later divided by 64 when shape reaches full hp.
                    hp_regen: 16. *
                    if is_hexagon {
                        if is_12gon {
                            1. * size.powi(2)
                        } else {
                            0.15 * size.powi(2)
                        }
                    } else if is_triangle{
                        3. * size.powi(2)
                    } else {
                        0.5 * size.powi(2)
                    },
                },
                texture: if is_12gon {
                    "12gon".to_owned()
                }
                else if is_hexagon {
                    "hexagon".to_owned()
                } else if is_triangle{
                    "triangle".to_owned()
                } else {
                    "square".to_owned()
                },
                just_spawned_mode: true,
            });
        }


        // things that happen for one (uprate physics, wall collision)
        {
            // mutable iterator over the physics' of all tanks and shapes (not bullets, as these are not affected by map boundnaries)
            let combined_iter_mut = self.tanks.iter_mut().map(|tank| &mut tank.1.physics)
            .chain(self.shapes.iter_mut().map(|shape| &mut shape.1.physics));

            for o in combined_iter_mut {
                if o.x.abs() > self.map_size.0 {
                    o.push(((self.map_size.0*o.x.signum() - o.x)*delta*2048., 0.));
                    o.xvel *= (-(delta)*100. / o.weight).exp();
                    o.yvel *= (-(delta)*100. / o.weight).exp();
                }
                if o.y.abs() > self.map_size.1 {
                    o.push((0., (self.map_size.1*o.y.signum() - o.y)*delta*2048.));
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

            // traps slow down 4x faster
            for (id, b) in self.bullets.iter_mut() {
                if b.texture == "trap" || b.texture == "trapbomb" {
                    b.physics.xvel *= (-delta * 1.5).exp();
                    b.physics.yvel *= (-delta * 1.5).exp();
                }
            }

            // remove <0 hp tanks
            self.tanks.retain(|_, v| v.physics.hp >= 0.);

            // for shapes and bullets, hexagons and bombs must be removed differently

            // find hexes that died
            let mut hex_to_remove = Vec::new();
            for (id, shape) in self.shapes.iter() {
                if shape.texture == "hexagon" && shape.physics.hp <= 0. && !shape.just_spawned_mode {
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
                        just_spawned_mode: false,
                    });
                }
            }


            // find bullets that died, also remove bullet ids from tank source
            let mut bombs_to_remove = Vec::new();
            for (id, bullet) in self.bullets.iter() {
                if (bullet.texture == "bomb"  || bullet.texture == "mbomb"  || bullet.texture == "trapbomb") && bullet.physics.hp <= 0. {
                    bombs_to_remove.push(id.clone());
                }
            }

            for id in bombs_to_remove {
                // handle dead bombs here
                let bomb = &mut self.bullets.get(&id).unwrap().clone();
                let combined_iter_mut = self.tanks.iter_mut().filter(|t| *t.0 != id).map(|tank| &mut tank.1.physics)
                .chain(self.shapes.iter_mut().map(|shape| &mut shape.1.physics))
                .chain(self.bullets.iter_mut().map(|bullet| &mut bullet.1.physics));
                for o in combined_iter_mut {
                    if o.dist(&bomb.physics) < bomb.physics.collision_size.powi(2) {
                        let s = (o.dist(&bomb.physics) - bomb.physics.collision_size.powi(2))*64.;
                        let dir = normalize(vector_diff((o.x, o.y), (bomb.physics.x, bomb.physics.y)));
                        o.push((s*dir.0, s*dir.1));
                    }
                }

                if bomb.physics.hp <= 0. {
                    for x in 0..(bomb.physics.collision_size as i32) {
                        let angle = (x*(360/bomb.physics.collision_size as i32)) as f64;
    
                        let bomb = &mut self.bullets.get(&id).unwrap().clone();
                        let size = 16.;
                        bomb.physics.collision_size = size*0.5;
                        bomb.physics.weight = bomb.physics.weight;
                        bomb.physics.max_hp = bomb.physics.max_hp;
                        bomb.physics.hp = bomb.physics.max_hp;
                        bomb.physics.hp_regen = if bomb.texture != "trapbomb" {-bomb.physics.max_hp/2.} else {-bomb.physics.max_hp/20.};
                        bomb.physics.xvel += angle.to_radians().sin() * size * if bomb.texture != "trapbomb" {72.} else {16.} * (1. + thread_rng().gen::<f64>());
                        bomb.physics.yvel += angle.to_radians().cos() * size * if bomb.texture != "trapbomb" {72.} else {16.} * (1. + thread_rng().gen::<f64>());
                        bomb.physics.x += angle.to_radians().sin() * size * 2. * thread_rng().gen::<f64>();
                        bomb.physics.y += angle.to_radians().cos() * size * 2. * thread_rng().gen::<f64>();

                        let id = thread_rng().gen();
                        self.bullets.insert(id, Bullet {
                            physics: bomb.physics,
                            texture: if bomb.texture != "trapbomb" {"bullet".to_owned()} else {"trap".to_owned()},
                            source_tank_id: bomb.source_tank_id,
                        });
                        if self.tanks.contains_key(&bomb.source_tank_id) {
                            self.tanks.get_mut(&bomb.source_tank_id).unwrap().bullet_ids.insert(id);
                        }
                    }
                }
            }


            for (id, bullet) in self.bullets.iter() {
                if self.tanks.contains_key(&bullet.source_tank_id) && bullet.physics.hp <= 0. {
                    self.tanks.get_mut(&bullet.source_tank_id).unwrap().bullet_ids.remove(id);
                }
            }


            // remove all dead shapes now
            self.shapes.retain(|_, v| v.physics.hp > 0.);
            // remove all dead bullets now
            self.bullets.retain(|_, v| v.physics.hp > 0.);


            // DRONES

            // for tank that makes drones
            for (id, t) in self.tanks.iter().filter(|t| t.1.texture == "spawner" || t.1.texture == "infector" || t.1.texture == "anthill" || t.1.texture == "trapspawner")  {
                // for drone in tank's bulletids
                for d_id in t.bullet_ids.iter() {
                    // move drone in direction to tank's firing_to
                    let d = &mut self.bullets.get_mut(&d_id).unwrap();
                    if d.texture == "drone" {
                        let vdiff = vector_diff((d.physics.x, d.physics.y), t.firing_to);
                        let dir;
                        if vector_lenght(vdiff) > 128. {
                            dir = normalize(vector_diff((d.physics.x, d.physics.y), t.firing_to));
                        } else {
                            dir = (vdiff.0/128., vdiff.1/128.);
                        }
                        d.physics.xvel += dir.0 * delta * 2048.;
                        d.physics.yvel += dir.1 * delta * 2048.;
                        d.physics.xvel *= (-delta).exp();
                        d.physics.yvel *= (-delta).exp();
                    }
                }
            }

            

            // just spawned mode
            for shape in self.shapes.values_mut() {
                if shape.just_spawned_mode {
                    shape.physics.weight = (shape.physics.hp * 4096.).sqrt().max(1.);
                    if shape.texture == "12gon" {
                        shape.physics.collision_size = ((shape.physics.hp*1.333).sqrt()).max(1.);
                    } else if shape.texture == "hexagon" {
                        shape.physics.collision_size = ((shape.physics.hp*13.33).sqrt()).max(1.);
                    } else if shape.texture == "triangle" {
                        shape.physics.collision_size = ((shape.physics.hp*120.).sqrt()).max(1.);
                    } else {
                        shape.physics.collision_size = ((shape.physics.hp*40.).sqrt()).max(1.);
                    }

                    if shape.physics.hp >= shape.physics.max_hp {
                        shape.just_spawned_mode = false;
                        shape.physics.hp_regen /= 16.;
                    }
                }
            }
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
                            let mut ap = (self.get_physics(&a).unwrap()).clone();
                            // key object physics (the just added object)
                            let mut kp = (self.get_physics(&k).unwrap()).clone();

                            // mbombs
                            if self.bullets.contains_key(&a) && self.bullets.get(&a).unwrap().texture == "mbomb" {
                                self.get_physics_mut(&a).unwrap().stick_to(&mut kp, delta*-0.5);
                            }
                            else if self.bullets.contains_key(&k) && self.bullets.get(&k).unwrap().texture == "mbomb" {
                                self.get_physics_mut(&k).unwrap().stick_to(&mut ap, delta*-0.5);
                            }

                            // disbled collision for bullet with bullet
                            else if self.bullets.contains_key(&a) && self.bullets.contains_key(&k) {
                                // can be used to handle some bullets differently

                                if self.bullets.get(&a).unwrap().texture == "trap" || self.bullets.get(&k).unwrap().texture == "trap"
                                || self.bullets.get(&a).unwrap().texture == "trapbomb" || self.bullets.get(&k).unwrap().texture == "trapbomb"
                                || self.bullets.get(&a).unwrap().texture == "bomb" || self.bullets.get(&k).unwrap().texture == "bomb"
                                 {
                                    self.get_physics_mut(&k).unwrap().collide_position_only(&ap, delta);
                                    self.get_physics_mut(&a).unwrap().collide_position_only(&mut kp, delta);
                                }
                            } else if self.tanks.contains_key(&a) && self.tanks.get(&a).unwrap().bullet_ids.contains(&k) || self.tanks.contains_key(&k) && self.tanks.get(&k).unwrap().bullet_ids.contains(&a) {
                                // DISABLE
                            } else {
                                // normal collision
                                self.get_physics_mut(&k).unwrap().collide(&ap, delta);                       
                                self.get_physics_mut(&a).unwrap().collide(&mut kp, delta);
                            }

                            // if k is a tank, and a is a bullet
                            // set last hit to source tank
                            if self.tanks.contains_key(&k) && self.bullets.contains_key(&a) {
                                self.tanks.get_mut(&k).unwrap().last_hit_id = self.bullets.get_mut(&a).unwrap().source_tank_id;
                                // add xp for kill, if the tank that killed is alive
                                if self.tanks.get(&k).unwrap().physics.hp < 0. && self.tanks.contains_key(&self.bullets.get(&a).unwrap().source_tank_id) {
                                    self.tanks.get_mut(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().evolution.xp += self.tanks.get_mut(&k).unwrap().evolution.killvalue;
                                    // TODO add kill
                                }
                            } // other way around
                            else if self.tanks.contains_key(&a) && self.bullets.contains_key(&k) {
                                self.tanks.get_mut(&a).unwrap().last_hit_id = self.bullets.get_mut(&k).unwrap().source_tank_id;
                                // add xp for kill, if the tank that killed is alive
                                if self.tanks.get(&a).unwrap().physics.hp < 0. && self.tanks.contains_key(&self.bullets.get(&k).unwrap().source_tank_id) {
                                    self.tanks.get_mut(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().evolution.xp += self.tanks.get_mut(&a).unwrap().evolution.killvalue;
                                    // TODO add kill
                                }
                            }

                            // if k is a tank, and a is a tank
                            // set last hit
                            else if self.tanks.contains_key(&k) && self.tanks.contains_key(&a) {
                                self.tanks.get_mut(&k).unwrap().last_hit_id = *a;
                                // other way around 
                                self.tanks.get_mut(&a).unwrap().last_hit_id = k;
                            }

                            // if k is a tank, and a is a shape
                            // add xp to the tank, if the shape hp is lees than 0 (it just died), and if the shape is not in just spawned mode
                            else if self.tanks.contains_key(&k) && self.shapes.contains_key(&a) {
                                if self.shapes.get(&a).unwrap().physics.hp < 0. && !self.shapes.get(&a).unwrap().just_spawned_mode {
                                    self.tanks.get_mut(&k).unwrap().evolution.add_xp(self.shapes.get_mut(&a).unwrap().physics.collision_size.powi(2)*0.01);
                                }
                            } // other way around
                            else if self.tanks.contains_key(&a) && self.shapes.contains_key(&k) && !self.shapes.get(&k).unwrap().just_spawned_mode {
                                if self.shapes.get(&k).unwrap().physics.hp < 0. {
                                    self.tanks.get_mut(&a).unwrap().evolution.add_xp(self.shapes.get_mut(&k).unwrap().physics.collision_size.powi(2)*0.01);
                                }
                            }

                            // if k is a bullet, and a is a shape
                            // add xp to the source tank, if it exists, and if the shape hp is lees than 0 (it just died)
                            else if self.bullets.contains_key(&k) && self.shapes.contains_key(&a) {
                                let bullet = &mut &self.bullets.get(&k).unwrap();
                                if self.tanks.contains_key(&self.bullets.get(&k).unwrap().source_tank_id) && self.shapes.get(&a).unwrap().physics.hp < 0.  {
                                    self.tanks.get_mut(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().evolution.add_xp(self.shapes.get_mut(&a).unwrap().physics.collision_size.powi(2)*0.01);

                                    // infector tank
                                    if self.tanks.get(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().texture == "infector" && self.shapes.get(&a).unwrap().texture == "triangle" &&
                                    self.tanks.get_mut(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().bullet_ids.len() < 256 {
                                        let uuid = thread_rng().gen::<u128>();
                                        let mut ph = self.shapes.get(&a).unwrap().physics.clone();
                                        ph.max_hp = (bullet.physics.max_hp * 2.).min(128.);
                                        ph.hp_regen = -ph.max_hp*0.1;
                                        ph.hp = ph.max_hp * 2.;
                                        self.bullets.insert(uuid, Bullet {
                                            physics: ph,
                                            source_tank_id: self.bullets.get(&k).unwrap().source_tank_id,
                                            texture: "drone".to_owned(),
                                        });
                                        self.tanks.get_mut(&self.bullets.get(&k).unwrap().source_tank_id).unwrap().bullet_ids.insert(uuid);
                                    }
                                }
                            } // other way around
                            else if self.bullets.contains_key(&a) && self.shapes.contains_key(&k) {
                                if self.tanks.contains_key(&self.bullets.get(&a).unwrap().source_tank_id) && self.shapes.get(&k).unwrap().physics.hp < 0. {
                                    let bullet = &mut &self.bullets.get(&a).unwrap();
                                    self.tanks.get_mut(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().evolution.add_xp(self.shapes.get_mut(&k).unwrap().physics.collision_size.powi(2)*0.01);

                                    // infector tank
                                    if self.tanks.get(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().texture == "infector" && self.shapes.get(&k).unwrap().texture == "triangle" &&
                                    self.tanks.get_mut(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().bullet_ids.len() < 256 {
                                        let uuid = thread_rng().gen::<u128>();
                                        let mut ph = self.shapes.get(&k).unwrap().physics.clone();
                                        ph.max_hp = (bullet.physics.max_hp * 2.).min(128.);
                                        ph.hp_regen = -ph.max_hp*0.1;
                                        ph.hp = ph.max_hp * 2.;
                                        self.bullets.insert(uuid, Bullet {
                                            physics: ph,
                                            source_tank_id: self.bullets.get(&a).unwrap().source_tank_id,
                                            texture: "drone".to_owned(),
                                        });
                                        self.tanks.get_mut(&self.bullets.get(&a).unwrap().source_tank_id).unwrap().bullet_ids.insert(uuid);
                                    }
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
    }
}

fn main() {
    // INIT

    // spawn network thread
    thread::spawn(|| run_network());

    // Initialize sld2 related things
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("SDL2 Window", 2560, 1440)
        // .fullscreen()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let font = ttf_context.load_font("textures/poopins.ttf", 36).unwrap();

    // Initialize other libraries. So far only the rand crate
    let mut rng = rand::thread_rng();


    // load textures. will be moved to its own function in the future
    let texture_creator = canvas.texture_creator();
    // HashMap of all the textures used in the game. Later will read all textures form the textures folder and add them to the hashmap by the filename without the extension
    let mut textures: HashMap<String, Texture> = HashMap::new();

    let paths = fs::read_dir("./svg/").unwrap();
    for path in paths {
        let path = path.unwrap().path().to_str().unwrap().to_string();
        println!("{}", path[6..path.len()-4].to_string());
        textures.insert(path[6..path.len()-4].to_string(), texture_creator.load_texture(path).unwrap());
    }
    
    textures.insert("bullet".to_owned(), texture_creator.load_texture("textures/bullet.png").unwrap());
    textures.insert("trap".to_owned(), texture_creator.load_texture("textures/trap.png").unwrap());
    textures.insert("bomb".to_owned(), texture_creator.load_texture("textures/bomb.png").unwrap());
    textures.insert("mbomb".to_owned(), texture_creator.load_texture("textures/bomb.png").unwrap());
    textures.insert("trapbomb".to_owned(), texture_creator.load_texture("textures/trap.png").unwrap());
    textures.insert("drone".to_owned(), texture_creator.load_texture("textures/triangle.png").unwrap());

    textures.insert("square".to_owned(), texture_creator.load_texture("textures/square.png").unwrap());
    textures.insert("hexagon".to_owned(), texture_creator.load_texture("textures/hexagon.png").unwrap());
    textures.insert("triangle".to_owned(), texture_creator.load_texture("textures/triangle.png").unwrap());
    textures.insert("12gon".to_owned(), texture_creator.load_texture("textures/12gon.png").unwrap());

    // wide not yet done
    textures.insert("wide".to_owned(), texture_creator.load_texture("svg/basic.svg").unwrap());

    // Initialize my own things
    let mut map = Map {
        map_size: (10_000., 10_000.,),
        shapes_max: 0,
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
        viewport_size: (2560, 1440)
    };

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

        while map.tanks.len() < 100 && (GAMEMODE == Gamemode::FFA || delta == 0.01 ) {
            let ai_tank_id = thread_rng().gen::<u128>();

            // add AI tank
            // tanks will be network or AI controlled on the server (also player controlled on LAN multiplayer server), and player or AI controlled in singleplayer
            let class = "basic";
            map.tanks.insert(
                ai_tank_id,
                EVOLUTION_TREE.get(&class.to_owned()).unwrap().0.clone()
            );
            let ph = &mut map.tanks.get_mut(&ai_tank_id).unwrap().physics;
            ph.x = thread_rng().gen::<f64>()*map.map_size.0*2. - map.map_size.0;
            ph.y = thread_rng().gen::<f64>()*map.map_size.1*2. - map.map_size.1;
            // will be clamped to max hp automatically
            ph.hp = 10000.;

            let ev = &mut map.tanks.get_mut(&ai_tank_id).unwrap().evolution;
            ev.hp_level = thread_rng().gen_range(0..1);
            ev.regen_level = thread_rng().gen_range(0..1);
            ev.reload_level = thread_rng().gen_range(0..1);
            ev.damage_level = thread_rng().gen_range(0..1);
            ev.speed_level = thread_rng().gen_range(0..1);
            ev.bulletspeed_level = thread_rng().gen_range(0..1);
            ev.class = class.to_owned();

            let tank =&mut map.tanks.get_mut(&ai_tank_id).unwrap();
            Evolution::level_refresh(tank);            

            map.tankais.push(TankAI {
                id: ai_tank_id,
                range: 3072.,
                tg_range: if tank.evolution.class == "shotgun" {
                    128.
                } else {
                    (tank.turrets[0].projectile_impulse/tank.turrets[0].projectile_weight).sqrt()  *  (tank.turrets[0].projectile_hp/-tank.turrets[0].projectile_hp_regen).sqrt()  *  8.
                },
                bullet_speed: tank.turrets[0].projectile_impulse/tank.turrets[0].projectile_weight,
                dodge_obstacles: true,
                fighting: true,
                tg_id: 0,
                next_upgrade_is_promotion: false
            });
        }

        if !map.tanks.contains_key(&playerid) {
            // add player
            map.tanks.insert(
                playerid,
                EVOLUTION_TREE.get(&"basic".to_string()).unwrap().0.clone()
            );

            let ph = &mut map.tanks.get_mut(&playerid).unwrap().physics;
            ph.x = thread_rng().gen::<f64>()*map.map_size.0*2. - map.map_size.0;
            ph.y = thread_rng().gen::<f64>()*map.map_size.1*2. - map.map_size.1;
            // will be clamped to max hp automatically
            // ph.hp = 10000.;

            // let ev = &mut map.tanks.get_mut(&playerid).unwrap().evolution;
            // ev.hp_level = thread_rng().gen_range(10..11);
            // ev.regen_level = thread_rng().gen_range(10..11);
            // ev.reload_level = thread_rng().gen_range(10..11);
            // ev.damage_level = thread_rng().gen_range(10..11);
            // ev.speed_level = thread_rng().gen_range(10..11);
            // ev.bulletspeed_level = thread_rng().gen_range(10..11);

            Evolution::level_refresh(map.tanks.get_mut(&playerid).unwrap()); 

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
            player.rotate_to(camera.to_map_coords(input.mouse_pos), delta);

            //firing
            if input.fire.is_down {
                player.fire(&mut map.bullets, playerid);
            }

            // Evolution

            // promoting
            if input.shift.is_down {
                let classes = &EVOLUTION_TREE.get(&player.evolution.class).expect("this class does not exist in the evolution tree").1;
                for x in 0..classes.len() {
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
                    if player.evolution.xp > 100. && player.evolution.hp_level < 10 {
                        player.evolution.hp_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
                }
                if input.u2.just && input.u2.is_down {
                    if player.evolution.xp > 100. && player.evolution.regen_level < 10 {
                        player.evolution.regen_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
                }
                if input.u3.just && input.u3.is_down {
                    if player.evolution.xp > 100. && player.evolution.reload_level < 10 {
                        player.evolution.reload_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
                }
                if input.u4.just && input.u4.is_down {
                    if player.evolution.xp > 100. && player.evolution.damage_level < 10 {
                        player.evolution.damage_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
                }
                if input.u5.just && input.u5.is_down {
                    if player.evolution.xp > 100. && player.evolution.speed_level < 10 {
                        player.evolution.speed_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
                }
                if input.u6.just && input.u6.is_down {
                    if player.evolution.xp > 100. && player.evolution.bulletspeed_level < 10 {
                        player.evolution.bulletspeed_level += 1;
                        player.evolution.xp -= 100.;
                        Evolution::level_refresh(player);
                    }
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
            if input.zoom_out.is_down && input.zoom_out.just {
                camera.zoom *= 0.96;
            }
            if input.zoom_in.is_down && input.zoom_in.just {
                camera.zoom *= 1.04;
            }
        }


        // RENDER
        // render the things you want to appear on top last

        // Clear the screen with black color
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // render map walls and grid
        map.render(&mut canvas, &camera);

        // Render all bullets
        for bullet in map.bullets.iter().filter(|(id, b)| camera.visible((b.physics.x, b.physics.y), b.physics.collision_size)) {
            bullet.1.render(&mut canvas, &camera, &textures);
        }

        // Render all shapes
        for bullet in map.shapes.iter().filter(|(id, b)| camera.visible((b.physics.x, b.physics.y), b.physics.collision_size)) {
            bullet.1.render(&mut canvas, &camera, &textures);
        }

        // Render all tanks
        for tank in map.tanks.iter().filter(|(id, b)| camera.visible((b.physics.x, b.physics.y), b.physics.collision_size)) {
            tank.1.render(&mut canvas, &camera, &textures);
        }


        // render text info, later will be better
        if map.tanks.contains_key(&playerid) && input.evolve.is_down {
            let xp = map.tanks.get(&playerid).unwrap().evolution.xp;
            let class = map.tanks.get(&playerid).unwrap().evolution.class.to_uppercase();
            let hp_level = map.tanks.get(&playerid).unwrap().evolution.hp_level;
            let regen_level = map.tanks.get(&playerid).unwrap().evolution.regen_level;
            let reload_level = map.tanks.get(&playerid).unwrap().evolution.reload_level;
            let damage_level = map.tanks.get(&playerid).unwrap().evolution.damage_level;
            let speed_level = map.tanks.get(&playerid).unwrap().evolution.speed_level;
            let bulletspeed_level = map.tanks.get(&playerid).unwrap().evolution.bulletspeed_level;
            let players = map.tanks.len();

            let text = format!(
                "XP: {:.0}\nClass: {}\n\nLevels:\nMAX HP: {}\nHP REGENERATION: {}\nRELOAD SPEED: {}\nBULLET DAMAGE: {}\nMOVEMENT SPEED: {}\nBULLET SPEED: {}, players: {}",
                xp, class, hp_level, regen_level, reload_level, damage_level, speed_level, bulletspeed_level, players
            );

            let mut y_offset = 40; // Adjust the Y offset for each line

            for line in text.lines() {
                // Skip rendering empty or whitespace-only lines
                if line.trim().is_empty() {
                    continue;
                }

                let surface = font
                    .render(line)
                    .blended(Color::RGB(255, 255, 255))
                    .map_err(|e| e.to_string())
                    .unwrap();
                let texture_creator: TextureCreator<_> = canvas.texture_creator();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string())
                    .unwrap();
                let texture_query = texture.query();
                let dest_rect = Rect::new(40, y_offset, texture_query.width, texture_query.height);
                canvas.copy(&texture, None, dest_rect).unwrap();

                y_offset += texture_query.height as i32; // Increase Y offset for the next line
            }

            // leaderbord

            let mut best_players: Vec<f64> = 
            map.tanks.iter().map(|t| t.1.evolution.killvalue*2.).collect();
            best_players.sort_by(|a, b| b.partial_cmp(a).unwrap());
            println!("top tank");
            // for t in map.tanks.values() {
            //     if t.evolution.killvalue*2. == best_players[0] {
            //         println!("evolution: {:?}", t)
            //     }
            // }
            best_players.resize(5, 0.);

            let text = format!(
                "TOP PLAYERS BY XP EARNED: \n1. {:.0}\n2. {:.0}\n3. {:.0}\n4. {:.0}\n5. {:.0}",
                best_players[0], best_players[1], best_players[2], best_players[3], best_players[4],
            );

            let mut y_offset = 40; // Adjust the Y offset for each line

            for line in text.lines() {
                // Skip rendering empty or whitespace-only lines
                if line.trim().is_empty() {
                    continue;
                }

                let surface = font
                    .render(line)
                    .blended(Color::RGB(255, 255, 255))
                    .map_err(|e| e.to_string())
                    .unwrap();
                let texture_creator: TextureCreator<_> = canvas.texture_creator();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string())
                    .unwrap();
                let texture_query = texture.query();
                let dest_rect = Rect::new(camera.viewport_size.0 - 512, y_offset, texture_query.width, texture_query.height);
                canvas.copy(&texture, None, dest_rect).unwrap();

                y_offset += texture_query.height as i32; // Increase Y offset for the next line
            }


        }

        canvas.present();

        // the game will slow down at below 30 fps
        delta = Instant::now().duration_since(last_frame_start).as_secs_f64().min(0.033333);
        if delta > 0.02 {
            println!("low fps: {:.0}", 1./delta);
        }

        // TEST PRINTS
        // println!("fps: {:.0}", 1./delta);

        // gamemode stuff

        // Survival map gets smaller
        if GAMEMODE == Gamemode::Survival {
            map.map_size.0 -= (delta * 16.).min(map.map_size.0*delta/128.);
            map.map_size.1 -= (delta * 16.).min(map.map_size.1*delta/128.);
        }

        map.shapes_max = ((map.map_size.0 * map.map_size.1) / 16384.) as usize;
    }
}
