use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::egui::{ Color32, Painter, Pos2, Rect, Stroke, Vec2, };
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

    // Transforms vector to window coordinates. Requires conversion to Vec2
    pub fn tf_coord(&self, phz_coord: &V2D) -> V2D {
        // Function arranged for clarity on transformation matrix being used.
        phz_coord.tf_fit(
            self.scaling,                   self.screen_rect.max.y as f64,
            self.screen_rect.min.x as f64, -self.scaling
        )
    }

    pub fn world_to_panel(&self, phz_coord: &V2D) -> Pos2{
        let tf = self.tf_coord(phz_coord);
        Pos2::new(tf.x as f32, tf.y as f32)
    }

    // Set the area. Changes the scale.
    pub fn area_to_rect(&self, rect: Rect) -> (Vec2, f32) {
        let rect_sz = rect.size();
        let world_sz = Vec2::new(self.world.area_sz.x as f32, self.world.area_sz.y as f32);

        if world_sz.x > world_sz.y {
            let scale = rect_sz.x / world_sz.x;
            // Clamp vertical size if it gets bigger than the window's.
            if world_sz.y * scale > rect_sz.y {
                let s = rect_sz.y / world_sz.y;
                return (Vec2::new(world_sz.x * s, rect_sz.y), s);
            }

            (Vec2::new(rect_sz.x, world_sz.y * scale), scale)
        } else {
            let scale = rect_sz.y / self.world.area_sz.y as f32;
            // Clamp horizontal size if it gets bigger than the window's.
            if world_sz.x * scale > rect_sz.x {
                let s = rect_sz.x / world_sz.x;
                return (Vec2::new(rect_sz.x, world_sz.y * s), s);
            }

            (Vec2::new(world_sz.x * scale, rect_sz.y), scale)

        }

    }


    // TODO: Decide whether to move this over to phyzzy_runner
    pub fn draw_model(&self, painter: &Painter, alpha: f64, hover_idx: Option<usize>) {
        let color = Color32::from_gray(128);
        let stroke = Stroke::new(1.0, color);

        // Draw springs
        for spring in self.model.get_springs() {
            let (p_a, p_b) = if !self.paused {
                let mass_a = self.model.get_mass(spring.get_ma());
                let mass_b = self.model.get_mass(spring.get_mb());

                let p_render_a = mass_a.p_i * alpha + mass_a.p_o * (1.0 - alpha);
                let p_render_b = mass_b.p_i * alpha + mass_b.p_o * (1.0 - alpha);

                (self.world_to_panel(&p_render_a), self.world_to_panel(&p_render_b))
            } else {
                let pos_a = self.model.get_mass(spring.get_ma()).p_i;
                let pos_b = self.model.get_mass(spring.get_mb()).p_i;
                (self.world_to_panel(&pos_a), self.world_to_panel(&pos_b))
            };

            painter.line_segment([p_a, p_b], stroke);
        }

        // Draw masses
        let mass_color = Color32::from_rgb(29, 179, 34);
        for (idx, mass) in self.model.get_masses().iter().enumerate() {

            // Final frame interpolation. Reference: https://www.gafferongames.com/post/fix_your_timestep/
            let p_render = mass.p_i * alpha + mass.p_o * (1.0 - alpha);
            let pos = if !self.paused { self.world_to_panel(&p_render) } else { self.world_to_panel(&mass.p_i) };

            let rad = (mass.r * self.scaling)  as f32;

            painter.circle_filled(pos, rad, mass_color);

            match hover_idx {
                None => {},
                Some(h_idx) => {
                    if idx == h_idx {
                        let highlight_color = Color32::from_gray(255);
                        let highlight_stroke = Stroke::new(1.0, highlight_color);
                        let highlight_rad = 5.0;
                        painter.circle_stroke(pos, rad + highlight_rad, highlight_stroke);
                    }
                },
            }
        }
    }

    pub fn _draw_world(&self, _painter: &Painter, _world: &World) {
        let color = Color32::from_gray(128);
        let _stroke = Stroke::new(1.0, color);


    }

}
