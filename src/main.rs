pub mod model_io;

use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2, Painter};
use std::time::Instant;

use model_io::ModelIO;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let init_dt = 60.0_f64.recip();
    let filename = String::from("models/triangle.json");

    let import_result = ModelIO::import(&filename, init_dt);
    let phz = match import_result {
        Ok(phz_elements) => {
            PhyzzySimulator {
                world: phz_elements.world,
                world_cfg: phz_elements.world_config,
                model: phz_elements.model,
                scaling: 100.0,
                view_sz: V2D::new(500.0, 500.0),
                dt: init_dt,
                t_now: Instant::now(),
            }
        },
        Err(_) => panic!("Could not load model."),
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
    t_now: Instant,
}

impl PhyzzySimulator {
    fn new(dt: f64, scaling: f64, view_sz: &V2D) -> Self {
        Self {
            dt,
            scaling,
            view_sz: V2D::from(view_sz),
            world: World::new(),
            world_cfg: WorldConfig { gravity: V2D::new(0.0, -9.81), drag: 0.0 },
            model: Model::new(5.0, 1.0),
            t_now: Instant::now(),
        }
    }

    // transforms vector to window coordinates. Requires conversion to Vec2
    fn tf_coord(&self, phz_coord: &V2D) -> V2D {
        phz_coord.tf_fit(self.scaling, self.view_sz.x, 0.0, -self.scaling)
    }

    fn world_to_panel(&self, phz_coord: &V2D) -> Pos2{
        let tf = self.tf_coord(phz_coord);
        Pos2::new(tf.x as f32, tf.y as f32)
    }

    fn draw_model(&self, painter: &Painter) {
        let color = Color32::from_gray(128);
        let stroke = Stroke::new(1.0, color);

        // Draw springs
        for spring in self.model.get_springs() {
            let p_a = self.world_to_panel(&self.model.get_mass(spring.get_ma()).p_i);
            let p_b = self.world_to_panel(&self.model.get_mass(spring.get_mb()).p_i);

            painter.line_segment([p_a, p_b], stroke);
        }

        // Draw masses
        for mass in self.model.get_masses() {
            let pos = self.world_to_panel(&mass.p_i);
            let rad = (mass.r * self.scaling)  as f32;
            painter.circle_filled(pos, rad, color);
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
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let size_v = Vec2::new(self.phz.view_sz.x as f32, self.phz.view_sz.y as f32);
            let (response, painter) = ui.allocate_painter(size_v, Sense::hover());
            let _ = response.rect;
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

            // Draw model.
            ui.request_repaint();
            self.phz.draw_model(&painter);

            // Update for next frame.
            self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, false);

            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.dt = t_elapsed.as_secs_f64();
            let framerate = t_elapsed.as_secs_f64().recip();
            let dt_display = format!("Framerate: {framerate:.width$} Hz", width=3);
            ui.heading(dt_display);
            self.phz.t_now = Instant::now();
        });
    }
}
