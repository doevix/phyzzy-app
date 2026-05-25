pub mod phyzzy_io;

use phyzzy_rs::{ self, Model, V2D, World, WorldConfig };
use eframe::{egui::{self, Color32, Painter, Pos2, Rect, Sense, Stroke, Vec2}, epaint::CornerRadiusF32};
use std::time::Instant;

use phyzzy_io::PhyzzyIO;

const FIXED_DT: f64 = 0.001;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let filename = String::from("models/triangle.json");
    let init_dt = 0.001;

    let import_result = PhyzzyIO::import(&filename, init_dt);
    let phz = match import_result {
        Ok(phz_elements) => {
            PhyzzySimulator {
                last_frame: 60.0_f64.recip(),
                dt: FIXED_DT,
                world: phz_elements.world,
                world_cfg: phz_elements.world_config,
                model: phz_elements.model,
                scaling: 100.0,
                world_area: V2D::new(8.0, 5.0),
                t_now: Instant::now(),
                screen_rect: Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 { x: 500.0, y: 500.0 as f32 } },
                }
        },
        Err(_) => PhyzzySimulator::new(100.0, &V2D::new(5.0, 5.0)),
    };

    // Run the model
    let _ = eframe::run_native("Phyzzy", native_options, Box::new(|cc| Ok(Box::new(PhyzzyApp::new(cc, phz)))));
}

struct PhyzzySimulator {
    world: World,
    world_cfg: WorldConfig,
    model: Model,
    scaling: f64,
    world_area: V2D,
    dt: f64,
    last_frame: f64,
    t_now: Instant,
    screen_rect: Rect,
}

impl PhyzzySimulator {
    fn new(scaling: f64, world_area: &V2D) -> Self {
        Self {
            dt: FIXED_DT,
            last_frame: 60.0_f64.recip(),
            scaling,
            world_area: V2D::from(world_area),
            world: World::new(),
            world_cfg: WorldConfig { gravity: V2D::new(0.0, -9.81), drag: 0.0 },
            model: Model::new(5.0, 1.0),
            t_now: Instant::now(),
            screen_rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: (scaling * world_area.x) as f32, y: (scaling * world_area.y) as f32 } },
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

    // Set the scaling to the size the given rect allows.
    fn set_scale_to_rect(&mut self, rect: Rect) {
        let biggest_world_size = if self.world_area.x > self.world_area.y { self.world_area.x } else { self.world_area.y };

        let rect_size = rect.max - rect.min;
        let smallest_rect_size = if rect_size.x < rect_size.y { rect_size.x } else { rect_size.y };

        self.scaling = smallest_rect_size as f64 / biggest_world_size;
    }

    // Set the area.
    fn area_to_rect(&mut self, rect: Rect) -> Vec2 {
        let rect_sz = rect.size();
        let world_sz = Vec2::new(self.world_area.x as f32, self.world_area.y as f32);

        if self.world_area.x > self.world_area.y {
            let scale = rect_sz.x / world_sz.x;
            // Clamp vertical size if it gets bigger than the window's.
            if world_sz.y * scale > rect_sz.y {
                let scale = rect_sz.y / world_sz.y;
                self.scaling = scale as f64;
                return Vec2::new(world_sz.x * scale, rect_sz.y);
            }

            self.scaling = scale as f64;
            Vec2::new(rect_sz.x, world_sz.y * scale)
        } else {
            let scale = rect_sz.y / self.world_area.y as f32;
            // Clamp horizontal size if it gets bigger than the window's.
            if world_sz.x * scale > rect_sz.x {
                let scale = rect_sz.x / world_sz.x;
                self.scaling = scale as f64;
                return Vec2::new(rect_sz.x, world_sz.y * scale);
            }

            self.scaling = scale as f64;
            Vec2::new(self.world_area.x as f32 * scale, rect_sz.y)

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
            // Get time passed.
            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.last_frame = t_elapsed.as_secs_f64();
            self.phz.t_now = Instant::now();

            // Setup painter.
            let view_area = ui.max_rect();
            let scaled_area = self.phz.area_to_rect(view_area);
            let (response, painter) = ui.allocate_painter(scaled_area, Sense::hover());
            let scr_rect = response.rect;
            self.phz.screen_rect = scr_rect;

            ui.request_repaint();

            // Update for next frame.
            let mut acc = self.phz.last_frame;
            let mut sim_cycles = 0;
            while acc >= self.phz.dt {
                self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, self.paused);
                acc -= self.phz.dt;
                sim_cycles += 1;
            }
            let alpha = acc / self.phz.dt;

            painter.rect_filled(view_area, CornerRadiusF32::same(0.0), Color32::from_gray(16));
            self.phz.draw_model(&painter, alpha);

            let framerate = t_elapsed.as_secs_f64().recip();
            let dt_display = format!("Framerate: {framerate:.width$} Hz, Cycles: {sim_cycles}, Rect: {scr_rect}, Max Rect: {view_area}", width=3);
            ui.heading(dt_display);
        });
    }
}
