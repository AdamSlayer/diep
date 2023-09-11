extern crate lazy_static;

use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{Tank, Physics, Turret};

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
                    weight: 60.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 35.,
                    hp: 60.,
                    max_hp: 60.,
                    hp_regen: 2.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 3_000.,
                    projectile_weight: 3.,
                    projectile_collision_size: 12.,
                    projectile_hp_regen: -0.5,
                    projectile_hp: 3.,
                    reload_time: 0.6,
                    inaccuracy: 1.,
                    relative_position: (0.,-52.),
                    ..Default::default()
                }],
                power: 15000.,
                rot_power: 450.,
                texture: "basic".to_owned(),
                ..Default::default()
            }, vec![
                "double".to_string(),
                "long".to_string(),
                "wide".to_string(),
                "bomber".to_string(),
                "trapper".to_string(),
                "shotgun".to_string()],
            1000.
        ));


        hash_set.insert("shotgun".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 80.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 40.,
                    hp: 80.,
                    max_hp: 80.,
                    hp_regen: 2.,
                },
                turrets: vec![],
                power: 25000.,
                rot_power: 700.,
                texture: "basic".to_owned(),
                ..Default::default()                
            }, vec![

            ],
            1000.
        ));

        for a in 0..41 {
            let angle = (a as f64 - 20.) / 10.;
            hash_set.get_mut("shotgun").unwrap().0.turrets.push(Turret {
                projectile_impulse: 16_000.,
                projectile_weight: 16.,
                projectile_collision_size: 5.,
                projectile_hp_regen: -3.0,
                projectile_hp: 3.,
                reload_time: 5.0,
                inaccuracy: 6.,
                relative_position: (angle.to_radians().sin()*50.,-angle.to_radians().cos()*50.),
                relative_direction: angle,
                ..Default::default()
            })
        }


        hash_set.insert("trapper".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 200.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 55.,
                    hp: 200.,
                    max_hp: 200.,
                    hp_regen: 4.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 5_000.,
                    projectile_weight: 100.,
                    projectile_collision_size: 5.,
                    projectile_hp_regen: -1.,
                    projectile_hp: 100.,
                    reload_time: 0.5,
                    inaccuracy: 0.,
                    relative_position: (0.,-65.),
                    projectile_texture: "trap".to_string(),
                    ..Default::default()
                }],
                power: 40000.,
                rot_power: 800.,
                texture: "basic".to_owned(),
                ..Default::default()                
            }, vec![
                "double".to_string(),
                "long".to_string(),
                "wide".to_string(),
                "bomber".to_string()],
            1000.
        ));


        hash_set.insert("machine".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 200.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 75.,
                    hp: 200.,
                    max_hp: 200.,
                    hp_regen: 4.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 8_000.,
                    projectile_weight: 6.,
                    projectile_collision_size: 15.,
                    projectile_hp_regen: -3.,
                    projectile_hp: 6.,
                    reload_time: 0.3,
                    inaccuracy: 3.,
                    relative_position: (0.,-95.),
                    ..Default::default()
                }],
                power: 9000.,
                rot_power: 120.,
                texture: "basic".to_owned(),
                ..Default::default()                
            }, vec![

            ],
            1000.
        ));


        hash_set.insert("bomber".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 70.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 45.,
                    hp: 70.,
                    max_hp: 70.,
                    hp_regen: 2.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 2_000.,
                    projectile_weight: 20.,
                    projectile_collision_size: 12.,
                    projectile_hp_regen: -3.,
                    projectile_hp: 15.,
                    reload_time: 3.,
                    inaccuracy: 1.,
                    relative_position: (0.,-70.),
                    projectile_texture: "bomb".to_string(),
                    ..Default::default()
                }],
                power: 25000.,
                rot_power: 600.,
                texture: "wide".to_owned(),
                ..Default::default()                
            }, vec![
                "fatbomber".to_string()
            ],
            1000.
        ));


        hash_set.insert("fatbomber".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 120.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 60.,
                    hp: 120.,
                    max_hp: 120.,
                    hp_regen: 3.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 15_000.,
                    projectile_weight: 100.,
                    projectile_collision_size: 36.,
                    projectile_hp_regen: -8.,
                    projectile_hp: 40.,
                    reload_time: 12.,
                    inaccuracy: 1.,
                    relative_position: (0.,-95.),
                    projectile_texture: "bomb".to_string(),
                    ..Default::default()
                }],
                power: 35000.,
                rot_power: 800.,
                texture: "wide".to_owned(),
                ..Default::default()                
            }, vec![
                // evolve to
            ],
            1000.
        ));

        hash_set.insert("long".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 40.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 35.,
                    hp: 40.,
                    max_hp: 40.,
                    hp_regen: 1.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 15_000.,
                    projectile_weight: 11.,
                    projectile_collision_size: 11.,
                    projectile_hp_regen: -8.,
                    projectile_hp: 14.,
                    reload_time: 1.6,
                    relative_position: (0.,-60.),
                    ..Default::default()
                }],
                power: 7500.,
                rot_power: 80.,
                texture: "long".to_owned(),
                ..Default::default()                
            }, vec![
            ],
            1000.
        ));

        hash_set.insert("double".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 70.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 40.,
                    hp: 70.,
                    max_hp: 70.,
                    hp_regen: 2.,
                },
                turrets: vec![
                Turret {
                    projectile_impulse: 800.,
                    projectile_weight: 2.0,
                    projectile_collision_size: 8.,
                    projectile_hp_regen: -1.2,
                    projectile_hp: 2.,
                    reload_time: 0.4,
                    inaccuracy: 2.,
                    relative_position: (-20.,-50.),
                    ..Default::default()
                },
                Turret {
                    projectile_impulse: 800.,
                    projectile_weight: 2.0,
                    projectile_collision_size: 8.,
                    projectile_hp_regen: -1.2,
                    projectile_hp: 2.,
                    reload_time: 0.4,
                    inaccuracy: 2.,
                    relative_position: (20.,-50.),
                    ..Default::default()
                },],
                power: 15000.,
                rot_power: 400.,
                texture: "double".to_owned(),
                ..Default::default()                
            }, vec![
                "machine".to_string()],
            1000.
        ));

        hash_set.insert("wide".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 100.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 50.,
                    hp: 100.,
                    max_hp: 100.,
                    hp_regen: 2.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 55_000.,
                    projectile_weight: 90.,
                    projectile_collision_size: 30.,
                    projectile_hp_regen: -20.,
                    projectile_hp: 80.,
                    reload_time: 2.0,
                    inaccuracy: 1.,
                    relative_position: (0.,-75.),
                    ..Default::default()
                }],
                power: 8000.,
                rot_power: 120.,
                texture: "wide".to_owned(),
                ..Default::default()                
            }, vec![
                "superwide".to_string(),
                ],
            1000.
        ));

        hash_set.insert("superwide".to_string(), (
            Tank {
                physics: Physics {
                    x: 0.,
                    y: 0.,
                    xvel: 0.,
                    yvel: 0.,
                    weight: 200.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 65.,
                    hp: 200.,
                    max_hp: 200.,
                    hp_regen: 4.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 130_000.,
                    projectile_weight: 180.,
                    projectile_collision_size: 40.,
                    projectile_hp_regen: -50.,
                    projectile_hp: 150.,
                    reload_time: 4.,
                    inaccuracy: 1.,
                    relative_position: (0.,-110.),
                    ..Default::default()
                }],
                power: 10000.,
                rot_power: 150.,
                texture: "wide".to_owned(),
                ..Default::default()                
            }, vec![
                // evolve to
            ],
            3000.
        ));

        // Add more entries here using hash_set.insert() as needed

        hash_set
    };
}