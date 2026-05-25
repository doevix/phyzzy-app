pub mod phyzzy_io;

use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::{egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2}, epaint::CornerRadiusF32};
use std::time::Instant;

use phyzzy_io::PhyzzyIO;

use crate::phyzzy_io::PhyzzyMeta;

const FIXED_DT: f64 = 0.001;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let filename = String::from("models/blob_thing.json");
    let init_dt = 0.001;

    let import_result = PhyzzyIO::import(&filename, init_dt);
    let phz = match import_result {
        Ok(phz_elements) => {
            PhyzzySimulator {
                last_frame: 0.0,
                dt: FIXED_DT,
                world: phz_elements.world,
                world_cfg: phz_elements.world_config,
                model: phz_elements.model,
                model_meta: phz_elements.meta,
                scaling: 0.0,
                t_now: Instant::now(),
                screen_rect: Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 { x: 0.0, y: 0.0 as f32 }
                },
                paused: false,
            }
        },
        Err(_) => PhyzzySimulator::new(),
    };
    // Run the model
    let _ = eframe::run_native("Phyzzy", native_options, Box::new(|cc| Ok(Box::new(PhyzzyApp::new(cc, phz)))));
}

struct PhyzzySimulator {
    model_meta: PhyzzyMeta,
    world: World,
    world_cfg: WorldConfig,
    model: Model,
    scaling: f64,
    dt: f64,
    last_frame: f64,
    t_now: Instant,
    screen_rect: Rect,
    paused: bool,
}

impl PhyzzySimulator {
    fn new() -> Self {
        Self {
            dt: FIXED_DT,
            last_frame: 0.0,
            scaling: 0.0,
            world: World::new(&V2D::new(8.0, 5.0)),
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

    // transforms vector to window coordinates. Requires conversion to Vec2
    fn tf_coord(&self, phz_coord: &V2D) -> V2D {
        phz_coord.tf_fit(self.scaling, self.screen_rect.max.y as f64, self.screen_rect.min.x as f64, -self.scaling)
    }

    fn world_to_panel(&self, phz_coord: &V2D) -> Pos2{
        let tf = self.tf_coord(phz_coord);
        Pos2::new(tf.x as f32, tf.y as f32)
    }

    // Set the area. Changes the scale.
    fn area_to_rect(&self, rect: Rect) -> (Vec2, f32) {
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

    fn draw_model(&self, painter: &Painter, alpha: f64) {
        let color = Color32::from_gray(128);
        let stroke = Stroke::new(1.0, color);

        // Draw springs
        for spring in self.model.get_springs() {
            let p_a = self.world_to_panel(&self.model.get_mass(spring.get_ma()).p_i);
            let p_b = self.world_to_panel(&self.model.get_mass(spring.get_mb()).p_i);

            painter.line_segment([p_a, p_b], stroke);
        }

        // Draw masses
        let mass_color = Color32::from_hex("#1DB322").unwrap();
        for mass in self.model.get_masses() {

            // Final frame interpolation. Reference: https://www.gafferongames.com/post/fix_your_timestep/
            let p_render = mass.p_i * alpha + mass.p_o * (1.0 - alpha);
            let pos = self.world_to_panel(&p_render);

            let rad = (mass.r * self.scaling)  as f32;
            painter.circle_filled(pos, rad, mass_color);
        }
    }

    fn _draw_world(&self, _painter: &Painter, _world: &World) {
        let color = Color32::from_gray(128);
        let _stroke = Stroke::new(1.0, color);


    }

}

struct PhyzzyApp {
    phz: PhyzzySimulator,
}


impl PhyzzyApp {
    fn new(_cc: &eframe::CreationContext<'_>, phz: PhyzzySimulator) -> Self {
        Self {
            phz,
        }
    }
}


impl eframe::App for PhyzzyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("options_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(500.0)
        .show_inside(ui, |ui| {
            let model_stats = format!("model: {name} by {creator} dt = {disp_dt:.3} [s], scaling: {scale:.3} [px/m]",
                                      disp_dt=self.phz.dt, scale=self.phz.scaling, name=self.phz.model_meta.name, creator=self.phz.model_meta.creator);
            ui.heading(model_stats);
            if ui.button("Pause").clicked() {
                self.phz.paused = !self.phz.paused;
            }
            if ui.button("change wave dir").clicked() {
                self.phz.model.wave_speed *= -1.0;
            }

        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get time passed.
            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.last_frame = t_elapsed.as_secs_f64();
            self.phz.t_now = Instant::now();

            // Update model.
            let mut acc = self.phz.last_frame;
            while acc >= self.phz.dt {
                self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, self.phz.paused);
                acc -= self.phz.dt;
            }
            let alpha = acc / self.phz.dt;

            // Setup painter.
            let full_area = ui.max_rect();

            let (scaled_area, scale) = self.phz.area_to_rect(full_area);
            self.phz.scaling = scale as f64;

            // Center the viewer.
            let center_offset = Vec2::new(
                (full_area.width() - scaled_area.x) * 0.5,
                (full_area.height() - scaled_area.y) * 0.5,
            );
            let centered_min = full_area.min + center_offset;
            let centered_rect = Rect::from_min_size(centered_min, scaled_area);
            let painter = ui.painter_at(centered_rect);
            self.phz.screen_rect = centered_rect;

            // Draw the background.
            ui.request_repaint();
            ui.painter().rect_filled(full_area, CornerRadiusF32::same(0.0), Color32::from_gray(0));
            painter.rect_filled(self.phz.screen_rect, CornerRadiusF32::same(0.0), Color32::from_gray(16));

            // Draw the model.
            self.phz.draw_model(&painter, alpha);
        });
    }
}
