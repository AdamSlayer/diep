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
                    weight: 20.,
                    rot: 0.,
                    rotvel: 0.,
                    collision_size: 30.,
                    hp: 20.,
                    max_hp: 20.,
                    hp_regen: 1.,
                },
                turrets: vec![Turret {
                    projectile_impulse: 2_000.,
                    projectile_weight: 3.,
                    projectile_collision_size: 12.,
                    projectile_hp_regen: -1.5,
                    projectile_hp: 3.,
                    reload_time: 1.,
                    inaccuracy: 1.,
                    relative_position: (0.,-52.),
                    ..Default::default()
                }],
                power: 3000.,
                rot_power: 50.,
                texture: "basic".to_owned(),
                ..Default::default()
            }, vec![
                "double".to_string(),
                "sniper".to_string(),
                "bomber".to_string(),
                "trapper".to_string(),
                "spawner".to_string(),],
            1000.
        ));


        {
            hash_set.insert("double".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 25.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 40.,
                        hp: 25.,
                        max_hp: 25.,
                        hp_regen: 1.1,
                    },
                    turrets: vec![
                    Turret {
                        projectile_impulse: 1300.,
                        projectile_weight: 2.15,
                        projectile_collision_size: 11.,
                        projectile_hp_regen: -1.1,
                        projectile_hp: 2.15,
                        reload_time: 1.,
                        inaccuracy: 2.,
                        relative_position: (-20.,-50.),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1300.,
                        projectile_weight: 2.15,
                        projectile_collision_size: 11.,
                        projectile_hp_regen: -1.1,
                        projectile_hp: 2.15,
                        reload_time: 1.,
                        inaccuracy: 2.,
                        relative_position: (20.,-50.),
                        ..Default::default()
                    },],
                    power: 3000.,
                    rot_power: 50.,
                    texture: "double".to_owned(),
                    ..Default::default()                
                }, vec![
                    "hailstorm".to_string(),
                    "triple".to_string(),
                    "cross".to_string()
                ],
                1000.
            ));


            hash_set.insert("triple".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 30.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 30.,
                        max_hp: 30.,
                        hp_regen: 1.2,
                    },
                    turrets: vec![
                    Turret {
                        projectile_impulse: 1200.,
                        projectile_weight: 2.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -1.1,
                        projectile_hp: 2.,
                        reload_time: 1.,
                        inaccuracy: 1.,
                        relative_position: (-25.,-55.),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1200.,
                        projectile_weight: 2.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -1.1,
                        projectile_hp: 2.,
                        reload_time: 1.,
                        inaccuracy: 1.,
                        relative_position: (0.,-65.),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1200.,
                        projectile_weight: 2.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -1.1,
                        projectile_hp: 2.,
                        reload_time: 1.,
                        inaccuracy: 1.,
                        relative_position: (25.,-55.),
                        ..Default::default()
                    },],
                    power: 3000.,
                    rot_power: 50.,
                    texture: "triple".to_owned(),
                    ..Default::default()                
                }, vec![
                ],
                1000.
            ));


            hash_set.insert("cross".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 80.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 60.,
                        hp: 80.,
                        max_hp: 80.,
                        hp_regen: 1.5,
                    },
                    turrets: vec![
                        Turret {
                            projectile_impulse: 2000.,
                            projectile_weight: 3.,
                            projectile_collision_size: 15.,
                            projectile_hp_regen: -2.,
                            projectile_hp: 5.,
                            reload_time: 1.,
                            inaccuracy: 1.,
                            relative_position: (0.,0.),
                            relative_direction: 0.,
                            ..Default::default()
                        },
                        Turret {
                            projectile_impulse: 2000.,
                            projectile_weight: 3.,
                            projectile_collision_size: 15.,
                            projectile_hp_regen: -2.,
                            projectile_hp: 5.,
                            reload_time: 1.,
                            inaccuracy: 1.,
                            relative_position: (0.,0.),
                            relative_direction: 180.,
                            ..Default::default()
                        },
                        Turret {
                            projectile_impulse: 2000.,
                            projectile_weight: 3.,
                            projectile_collision_size: 15.,
                            projectile_hp_regen: -2.,
                            projectile_hp: 5.,
                            reload_time: 1.,
                            inaccuracy: 1.,
                            relative_position: (0.,0.),
                            relative_direction: 90.,
                            ..Default::default()
                        },
                        Turret {
                            projectile_impulse: 2000.,
                            projectile_weight: 3.,
                            projectile_collision_size: 15.,
                            projectile_hp_regen: -2.,
                            projectile_hp: 5.,
                            reload_time: 1.,
                            inaccuracy: 1.,
                            relative_position: (0.,0.),
                            relative_direction: 270.,
                            ..Default::default()
                        },
                    ],
                    power: 4500.,
                    rot_power: 50.,
                    texture: "cross".to_owned(),
                    ..Default::default()                
                }, vec![
                ],
                1000.
            ));


            hash_set.insert("hailstorm".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 100.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 60.,
                        hp: 100.,
                        max_hp: 100.,
                        hp_regen: 3.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 2000.,
                        projectile_weight: 2.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -2.,
                        projectile_hp: 3.,
                        reload_time: 0.2,
                        inaccuracy: 5.,
                        relative_position: (0.,-85.),
                        ..Default::default()
                    }],
                    power: 4000.,
                    rot_power: 50.,
                    texture: "hailstorm".to_owned(),
                    ..Default::default()                
                }, vec![
    
                ],
                1000.
            ));
        }


        {
            hash_set.insert("spawner".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 25.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 40.,
                        hp: 25.,
                        max_hp: 25.,
                        hp_regen: 1.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 2_000.,
                        projectile_weight: 6.,
                        projectile_collision_size: 15.,
                        projectile_hp_regen: -1.2,
                        projectile_hp: 12.,
                        reload_time: 2.,
                        inaccuracy: 1.,
                        relative_position: (0.,-52.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 50.,
                    texture: "spawner".to_owned(),
                    ..Default::default()
                }, vec![
                    "infector".to_owned(),
                    "anthill".to_owned(),
                    "trapspawner".to_owned(),
                ],
                1000.
            ));


            hash_set.insert("anthill".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 35.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 35.,
                        max_hp: 35.,
                        hp_regen: 1.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 5.,
                        reload_time: 1.6,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: 0.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 5.,
                        reload_time: 1.6,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: 120.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 5.,
                        reload_time: 1.6,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: -120.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 2500.,
                    rot_power: 40.,
                    texture: "anthill".to_owned(),
                    ..Default::default()
                }, vec![
                    "infector".to_owned(),
                    "anthill".to_owned(),
                ],
                1000.
            ));

            hash_set.insert("infector".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 30.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 30.,
                        max_hp: 30.,
                        hp_regen: 1.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 2_000.,
                        projectile_weight: 6.,
                        projectile_collision_size: 15.,
                        projectile_hp_regen: -1.2,
                        projectile_hp: 12.,
                        reload_time: 2.4,
                        inaccuracy: 1.,
                        relative_position: (0.,-52.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 3500.,
                    rot_power: 50.,
                    texture: "infector".to_owned(),
                    ..Default::default()
                }, vec![
                ],
                1000.
            ));
        }


        {
            hash_set.insert("bomber".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 40.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 40.,
                        max_hp: 40.,
                        hp_regen: 1.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 3_000.,
                        projectile_weight: 30.,
                        projectile_collision_size: 32.,
                        projectile_hp_regen: -5.,
                        projectile_hp: 25.,
                        reload_time: 7.,
                        inaccuracy: 1.,
                        relative_position: (0.,-70.),
                        projectile_texture: "bomb".to_string(),
                        ..Default::default()
                    }],
                    power: 4500.,
                    rot_power: 80.,
                    texture: "bomber".to_owned(),
                    ..Default::default()                
                }, vec![
                    "trapbomber".to_string(),
                    "tribomber".to_string(),
                    "magnet bomber".to_string(),
                ],
                1000.
            ));

            hash_set.insert("magnet bomber".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 45.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 45.,
                        max_hp: 45.,
                        hp_regen: 1.1,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 3_000.,
                        projectile_weight: 30.,
                        projectile_collision_size: 32.,
                        projectile_hp_regen: -5.,
                        projectile_hp: 25.,
                        reload_time: 7.,
                        inaccuracy: 1.,
                        relative_position: (0.,-80.),
                        projectile_texture: "mbomb".to_string(),
                        ..Default::default()
                    }],
                    power: 4500.,
                    rot_power: 80.,
                    texture: "magnetbomber".to_owned(),
                    ..Default::default()                
                }, vec![
                    "tribomber".to_string(),
                    "trapbomber".to_string(),
                ],
                1000.
            ));
    
            hash_set.insert("tribomber".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 50.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 55.,
                        hp: 50.,
                        max_hp: 50.,
                        hp_regen: 1.1,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 28.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 9.,
                        inaccuracy: 1.,
                        relative_position: (-15.,-65.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: -10.,
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 28.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 9.,
                        inaccuracy: 1.,
                        relative_position: (0.,-70.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: 0.,
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 28.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 9.,
                        inaccuracy: 1.,
                        relative_position: (15.,-65.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: 10.,
                        ..Default::default()
                    }
                    ],
                    power: 4500.,
                    rot_power: 80.,
                    texture: "tribomber".to_owned(),
                    ..Default::default()                
                }, vec![
                ],
                1000.
            ));
        }

        {
            hash_set.insert("trapper".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 40.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 40.,
                        max_hp: 40.,
                        hp_regen: 1.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 20_000.,
                        projectile_weight: 200.,
                        projectile_collision_size: 6.,
                        projectile_hp_regen: -8.,
                        projectile_hp: 200.,
                        reload_time: 1.2,
                        inaccuracy: 0.,
                        relative_position: (0.,-65.),
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 40.,
                    texture: "trapper".to_owned(),
                    ..Default::default()                
                }, vec![
                    "trapbomber".to_string(),
                    "trapspawner".to_string(),
                    "barricade".to_string(),
                    ],
                1000.
            ));

            hash_set.insert("trapspawner".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 45.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 55.,
                        hp: 45.,
                        max_hp: 45.,
                        hp_regen: 1.1,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 16_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 4.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 50.,
                        reload_time: 1.2,
                        inaccuracy: 4.,
                        relative_position: (0.,0.),
                        relative_direction: -120.,
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 16_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 4.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 50.,
                        reload_time: 1.2,
                        inaccuracy: 4.,
                        relative_position: (0.,0.),
                        relative_direction: 120.,
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    },
                    // spawner turret
                    Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 12.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 6.,
                        reload_time: 1.8,
                        inaccuracy: 1.,
                        relative_position: (0.,-60.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 40.,
                    texture: "trapspawner".to_owned(),
                    ..Default::default()                
                }, vec![
                    ],
                1000.
            ));

            hash_set.insert("barricade".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 55.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 60.,
                        hp: 55.,
                        max_hp: 55.,
                        hp_regen: 1.2,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 16_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 4.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 50.,
                        reload_time: 0.3,
                        inaccuracy: 10.,
                        relative_position: (0.,-65.),
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 40.,
                    texture: "barricade".to_owned(),
                    ..Default::default()                
                }, vec![
                    ],
                1000.
            ));
    

            hash_set.insert("trapbomber".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 50.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 50.,
                        max_hp: 50.,
                        hp_regen: 1.2,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 10_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 16.,
                        projectile_hp_regen: -10.,
                        projectile_hp: 50.,
                        reload_time: 6.0,
                        inaccuracy: 1.,
                        relative_position: (0.,-80.),
                        projectile_texture: "trapbomb".to_string(),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 40.,
                    texture: "trapbomber".to_owned(),
                    ..Default::default()                
                }, vec![],
                1000.
            ));
        }


        {
            hash_set.insert("sniper".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 20.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 35.,
                        hp: 20.,
                        max_hp: 20.,
                        hp_regen: 1.2,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 32_000.,
                        projectile_weight: 25.,
                        projectile_collision_size: 15.,
                        projectile_hp_regen: -10.,
                        projectile_hp: 25.,
                        reload_time: 6.,
                        relative_position: (0.,-60.),
                        ..Default::default()
                    }],
                    power: 4000.,
                    rot_power: 30.,
                    texture: "sniper".to_owned(),
                    ..Default::default()                
                }, vec![
                    "hailstorm".to_string(),
                    "wide".to_string(),
                    "shotgun".to_string(),
                ],
                1000.
            ));
            
    
            hash_set.insert("wide".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 80.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 65.,
                        hp: 80.,
                        max_hp: 80.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 60_000.,
                        projectile_weight: 100.,
                        projectile_collision_size: 40.,
                        projectile_hp_regen: -25.,
                        projectile_hp: 100.,
                        reload_time: 4.0,
                        inaccuracy: 0.,
                        relative_position: (0.,-90.),
                        ..Default::default()
                    }],
                    power: 3000.,
                    rot_power: 50.,
                    texture: "wide".to_owned(),
                    ..Default::default()                
                }, vec![],
                1000.
            ));
    
    
            hash_set.insert("shotgun".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 40.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 50.,
                        hp: 40.,
                        max_hp: 40.,
                        hp_regen: 1.4,
                    },
                    turrets: vec![],
                    power: 3500.,
                    rot_power: 60.,
                    texture: "shotgun".to_owned(),
                    ..Default::default()                
                }, vec![
    
                ],
                1000.
            ));
    
            for a in 0..41 {
                let angle = (a as f64 - 20.) / 10.;
                hash_set.get_mut("shotgun").unwrap().0.turrets.push(Turret {
                    projectile_impulse: 8_000.,
                    projectile_weight: 8.,
                    projectile_collision_size: 5.,
                    projectile_hp_regen: -2.0,
                    projectile_hp: 2.,
                    reload_time: 6.0,
                    inaccuracy: 6.,
                    relative_position: (angle.to_radians().sin()*50.,-angle.to_radians().cos()*50.),
                    relative_direction: angle,
                    ..Default::default()
                })
            }
        }


        // Add more entries here using hash_set.insert() as needed

        hash_set
    };
}