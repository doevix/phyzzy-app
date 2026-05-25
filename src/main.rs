pub mod phyzzy_io;

use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2, Painter, Rect};
use std::time::Instant;

use phyzzy_io::PhyzzyIO;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let filename = String::from("models/triangle.json");
    let init_dt = 0.001;

    let import_result = PhyzzyIO::import(&filename, init_dt);
    let phz = match import_result {
        Ok(phz_elements) => {
            PhyzzySimulator {
                last_frame: 60.0_f64.recip(),
                dt: init_dt,
                world: phz_elements.world,
                world_cfg: phz_elements.world_config,
                model: phz_elements.model,
                scaling: 100.0,
                view_sz: V2D::new(500.0, 500.0),
                t_now: Instant::now(),
                screen_rect: Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 { x: 500.0, y: 500.0 as f32 } },
                }
        },
        Err(_) => PhyzzySimulator::new(100.0, &V2D::new(500.0, 500.0)),
    };

    // Run the model
    let _ = eframe::run_native("Phyzzy", native_options, Box::new(|cc| Ok(Box::new(PhyzzyApp::new(cc, phz)))));
}

struct PhyzzySimulator {
    world: World,
    world_cfg: WorldConfig,
    model: Model,
    scaling: f64,
    view_sz: V2D,
    dt: f64,
    last_frame: f64,
    t_now: Instant,
    screen_rect: Rect,
}

impl PhyzzySimulator {
    fn new(scaling: f64, view_sz: &V2D) -> Self {
        Self {
            dt: 0.001,
            last_frame: 60.0_f64.recip(),
            scaling,
            view_sz: V2D::from(view_sz),
            world: World::new(),
            world_cfg: WorldConfig { gravity: V2D::new(0.0, -9.81), drag: 0.0 },
            model: Model::new(5.0, 1.0),
            t_now: Instant::now(),
            screen_rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: view_sz.x as f32, y: view_sz.y as f32 } },
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
    paused: bool
}


impl PhyzzyApp {
    fn new(_cc: &eframe::CreationContext<'_>, phz: PhyzzySimulator) -> Self {
        Self {
            phz,
            paused: false,
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
            let mystr = format!("dt = {disp_dt:.3}", disp_dt=self.phz.dt);
            ui.heading(mystr);
            if ui.button("Pause").clicked() {
                self.paused = !self.paused;
            }

        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let size_v = Vec2::new(self.phz.view_sz.x as f32, self.phz.view_sz.y as f32);
            let (response, painter) = ui.allocate_painter(size_v, Sense::hover());
            let scr_rect = response.rect;
            self.phz.screen_rect = scr_rect;

            let color = Color32::from_gray(128);
            let stroke = Stroke::new(1.0, color);

            if self.phz.world.bounds.len() > 0 {
                // Get boundary points.
                let left_side_x = 0.0;
                let right_side_x = self.phz.view_sz.x / self.phz.scaling;
                let bound_nrm = self.phz.world.bounds[0].nrm;
                let mb = -bound_nrm.x / bound_nrm.y;
                let pos_b = self.phz.world.bounds[0].pos;
                let y1 = pos_b.y - mb * (pos_b.x - left_side_x);
                let y2 = pos_b.y - mb * (pos_b.x - right_side_x);
                let p_1 = self.phz.world_to_panel(&V2D::new(left_side_x, y1));
                let p_2 = self.phz.world_to_panel(&V2D::new(right_side_x, y2));
                // Draw boundary.
                painter.line_segment([p_1, p_2], stroke);
            }

            ui.request_repaint();

            // Get time passed.
            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.last_frame = t_elapsed.as_secs_f64();
            self.phz.t_now = Instant::now();

            // Update for next frame.
            let mut acc = self.phz.last_frame;
            let mut sim_cycles = 0;
            while acc >= self.phz.dt {
                self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, self.paused);
                acc -= self.phz.dt;
                sim_cycles += 1;
            }
            let alpha = acc / self.phz.dt;

            self.phz.draw_model(&painter, alpha);

            let framerate = t_elapsed.as_secs_f64().recip();
            let dt_display = format!("Framerate: {framerate:.width$} Hz, Cycles: {sim_cycles}, Rect: {scr_rect}", width=3);
            ui.heading(dt_display);
        });
    }
}
