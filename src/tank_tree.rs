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
                        projectile_impulse: 1000.,
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
                        projectile_impulse: 1000.,
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
                    "machine".to_string(),
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
                        weight: 85.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 40.,
                        hp: 85.,
                        max_hp: 85.,
                        hp_regen: 2.,
                    },
                    turrets: vec![
                    Turret {
                        projectile_impulse: 1300.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 9.,
                        projectile_hp_regen: -1.5,
                        projectile_hp: 3.,
                        reload_time: 0.55,
                        inaccuracy: 1.,
                        relative_position: (-25.,-55.),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1100.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 9.,
                        projectile_hp_regen: -1.5,
                        projectile_hp: 3.,
                        reload_time: 0.55,
                        inaccuracy: 1.,
                        relative_position: (0.,-65.),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1300.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 9.,
                        projectile_hp_regen: -1.5,
                        projectile_hp: 3.,
                        reload_time: 0.55,
                        inaccuracy: 1.,
                        relative_position: (25.,-55.),
                        ..Default::default()
                    },],
                    power: 20000.,
                    rot_power: 400.,
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
                    power: 15000.,
                    rot_power: 400.,
                    texture: "cross".to_owned(),
                    ..Default::default()                
                }, vec![
                ],
                1000.
            ));


            hash_set.insert("machine".to_string(), (
                Tank {
                    physics: Physics {
                        x: 0.,
                        y: 0.,
                        xvel: 0.,
                        yvel: 0.,
                        weight: 160.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 70.,
                        hp: 160.,
                        max_hp: 160.,
                        hp_regen: 4.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 2_500.,
                        projectile_weight: 2.,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -2.,
                        projectile_hp: 3.,
                        reload_time: 0.15,
                        inaccuracy: 0.,
                        relative_position: (0.,-85.),
                        ..Default::default()
                    }],
                    power: 10000.,
                    rot_power: 120.,
                    texture: "machine".to_owned(),
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
                        weight: 75.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 40.,
                        hp: 75.,
                        max_hp: 75.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 12.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 6.,
                        reload_time: 0.8,
                        inaccuracy: 1.,
                        relative_position: (0.,-52.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 15000.,
                    rot_power: 450.,
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
                        weight: 80.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 80.,
                        max_hp: 80.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.4,
                        projectile_hp: 4.,
                        reload_time: 0.8,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: 0.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.4,
                        projectile_hp: 4.,
                        reload_time: 0.8,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: 120.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 2.5,
                        projectile_collision_size: 10.,
                        projectile_hp_regen: -0.4,
                        projectile_hp: 4.,
                        reload_time: 0.8,
                        inaccuracy: 1.,
                        relative_position: (0.,0.),
                        relative_direction: -120.,
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 5000.,
                    rot_power: 100.,
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
                        weight: 90.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 35.,
                        hp: 90.,
                        max_hp: 90.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 1_000.,
                        projectile_weight: 3.,
                        projectile_collision_size: 12.,
                        projectile_hp_regen: -0.5,
                        projectile_hp: 6.,
                        reload_time: 1.2,
                        inaccuracy: 1.,
                        relative_position: (0.,-52.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 10000.,
                    rot_power: 200.,
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
                        weight: 70.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 70.,
                        max_hp: 70.,
                        hp_regen: 2.,
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
                    power: 25000.,
                    rot_power: 600.,
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
                        weight: 80.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 80.,
                        max_hp: 80.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 0_000.,
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
                    power: 25000.,
                    rot_power: 600.,
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
                        weight: 80.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 55.,
                        hp: 80.,
                        max_hp: 80.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 24.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 7.,
                        inaccuracy: 1.,
                        relative_position: (-15.,-65.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: -10.,
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 24.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 7.,
                        inaccuracy: 1.,
                        relative_position: (0.,-70.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: 0.,
                        ..Default::default()
                    },
                    Turret {
                        projectile_impulse: 5_000.,
                        projectile_weight: 20.,
                        projectile_collision_size: 24.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 15.,
                        reload_time: 7.,
                        inaccuracy: 1.,
                        relative_position: (15.,-65.),
                        projectile_texture: "bomb".to_string(),
                        relative_direction: 10.,
                        ..Default::default()
                    }
                    ],
                    power: 25000.,
                    rot_power: 600.,
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
                        weight: 200.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 55.,
                        hp: 200.,
                        max_hp: 200.,
                        hp_regen: 4.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 20_000.,
                        projectile_weight: 100.,
                        projectile_collision_size: 5.,
                        projectile_hp_regen: -5.,
                        projectile_hp: 100.,
                        reload_time: 0.8,
                        inaccuracy: 0.,
                        relative_position: (0.,-65.),
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    }],
                    power: 40000.,
                    rot_power: 800.,
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
                        weight: 160.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 55.,
                        hp: 160.,
                        max_hp: 160.,
                        hp_regen: 4.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 16_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 4.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 50.,
                        reload_time: 1.0,
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
                        reload_time: 1.0,
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
                        reload_time: 1.0,
                        inaccuracy: 1.,
                        relative_position: (0.,-60.),
                        projectile_texture: "drone".to_string(),
                        ..Default::default()
                    }],
                    power: 12000.,
                    rot_power: 120.,
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
                        weight: 160.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 60.,
                        hp: 160.,
                        max_hp: 160.,
                        hp_regen: 4.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 16_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 4.,
                        projectile_hp_regen: -3.,
                        projectile_hp: 50.,
                        reload_time: 0.25,
                        inaccuracy: 10.,
                        relative_position: (0.,-65.),
                        projectile_texture: "trap".to_string(),
                        ..Default::default()
                    }],
                    power: 40000.,
                    rot_power: 800.,
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
                        weight: 70.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 45.,
                        hp: 70.,
                        max_hp: 70.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 10_000.,
                        projectile_weight: 50.,
                        projectile_collision_size: 12.,
                        projectile_hp_regen: -10.,
                        projectile_hp: 50.,
                        reload_time: 3.0,
                        inaccuracy: 1.,
                        relative_position: (0.,-80.),
                        projectile_texture: "trapbomb".to_string(),
                        ..Default::default()
                    }],
                    power: 35000.,
                    rot_power: 600.,
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
                    texture: "sniper".to_owned(),
                    ..Default::default()                
                }, vec![
                    "machine".to_string(),
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
                        weight: 160.,
                        rot: 0.,
                        rotvel: 0.,
                        collision_size: 60.,
                        hp: 160.,
                        max_hp: 160.,
                        hp_regen: 2.,
                    },
                    turrets: vec![Turret {
                        projectile_impulse: 75_000.,
                        projectile_weight: 100.,
                        projectile_collision_size: 40.,
                        projectile_hp_regen: -25.,
                        projectile_hp: 100.,
                        reload_time: 3.0,
                        inaccuracy: 0.,
                        relative_position: (0.,-90.),
                        ..Default::default()
                    }],
                    power: 12000.,
                    rot_power: 150.,
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
                    texture: "shotgun".to_owned(),
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
        }


        // Add more entries here using hash_set.insert() as needed

        hash_set
    };
}