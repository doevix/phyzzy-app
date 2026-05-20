use phyzzy_rs::{self, Boundary, Mass, Model, Spring, V2D, World, WorldConfig, Loader};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2, Painter};
use std::fs;
use std::time::Instant;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let mut phz = PhyzzySimulator::new(60.0_f64.recip(), 100.0, &V2D::new(500.0, 500.0));

    // Super crude model loader, make it better later.
    let model_json = fs::read_to_string("triangle.json").unwrap();
    let model_proto = Loader::load_from_json_str(&model_json);

    match model_proto {
        Ok(loaded_model) => {
            println!("{:?}", loaded_model);

            phz.world_cfg.drag = loaded_model.world_config.drag;
            phz.world_cfg.gravity = V2D::new(loaded_model.world_config.gravity.x, loaded_model.world_config.gravity.y);

            for mass in loaded_model.model.masses {
                let pos = V2D::new(mass.pos.x, mass.pos.y);
                let vel = V2D::new(mass.vel.x, mass.vel.y);
                let pos_prv = pos - vel * phz.dt;
                let loaded_mass = Mass::load(mass.mass, mass.radius, &pos, &pos_prv);
                phz.model.new_mass(loaded_mass);
            }

            for spring in loaded_model.model.springs {
                let loaded_spring = Spring::new(spring.restlength, spring.springing, spring.dampening, spring.m_a, spring.m_b);
                phz.model.new_spring(loaded_spring).unwrap();
            }

            for bound in loaded_model.world.bounds {
                let pos = V2D::new(bound.pos.x, bound.pos.y);
                let nrm = V2D::new(bound.nrm.x, bound.nrm.y);
                let loaded_bound = Boundary::new(pos, nrm, bound.refl, bound.mu_s, bound.mu_k);
                phz.world.bounds.push(loaded_bound);
            }
        },
        Err(e) => panic!("Could not parse JSON to file: {e:?}"),
    }

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
            model: Model::new(0.0, 0.0),
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
        egui::Panel::left("Sim controls").resizable(true).show_inside(ui, |ui| {
            ui.heading("sim controls");
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let size_v = Vec2::new(self.phz.view_sz.x as f32, self.phz.view_sz.y as f32);
            let (response, painter) = ui.allocate_painter(size_v, Sense::hover());
            let _ = response.rect;
            let color = Color32::from_gray(128);
            let stroke = Stroke::new(1.0, color);

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

            // Draw model.
            ui.request_repaint();
            self.phz.draw_model(&painter);

            // Update for next frame.
            self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg);

            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.dt = t_elapsed.as_secs_f64();
            let dt_display = format!("dt = {:?}", t_elapsed);
            ui.heading(dt_display);
            self.phz.t_now = Instant::now();
        });
    }
}
