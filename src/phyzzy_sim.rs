use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::egui::{ Pos2, Rect, };
use std::time::Instant;

use crate::phyzzy_io::PhyzzyMeta;

pub struct PhyzzySimulator {
    pub model_meta: PhyzzyMeta,
    pub world: World,
    pub world_cfg: WorldConfig,
    pub model: Model,
    pub scaling: f64,
    pub dt: f64,
    pub last_frame: f64,
    pub t_now: Instant,
    pub screen_rect: Rect,
    pub paused: bool,
}

impl PhyzzySimulator {
    pub fn new() -> Self {
        Self {
            dt: 0.001,
            last_frame: 0.0,
            scaling: 0.0,
            world: World::new(&V2D::new(12.0, 7.5)),
            world_cfg: WorldConfig { gravity: V2D::new(0.0, -9.81), drag: 0.0 },
            model: Model::new(5.0, 1.0),
            model_meta: PhyzzyMeta {
                name: "No name".to_string(),
                creator: "anonymous".to_string(),
                created: "no date".to_string(),
            },
            t_now: Instant::now(),
            screen_rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: 0.0, y: 0.0 },
            },
            paused: false,
        }
    }
}
