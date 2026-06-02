use phyzzy_rs::{ self, V2D };
use eframe::egui::{ self, Color32, Pos2, Vec2, Slider, Sense };
use egui_plot::{ self, Line, LineStyle, Plot, PlotPoints };
use core::f64;
use std::{f64::consts::TAU, time::Instant};

use crate::phyzzy_sim::PhyzzySimulator;
use crate::phyzzy_viewport::{ PhyzzyViewport, PhyzzyObject };

pub struct PhyzzyApp {
    pub phz: PhyzzySimulator,
    pub viewport: PhyzzyViewport,
    pub pointer_pos: Option<Pos2>,
    pub pointer_interact_pos: Option<Pos2>,
    pub pointer_drag_delta: Vec2,
    pub drag_vel: V2D,
    pub mass_idx: Option<usize>,
    pub held_idx: Option<usize>,
    pub sel_idx: Option<usize>,
}


impl PhyzzyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, phz: PhyzzySimulator) -> Self {
        Self {
            phz,
            viewport: PhyzzyViewport::init(),
            pointer_pos: None,
            pointer_interact_pos: None,
            pointer_drag_delta: Vec2::new(0.0, 0.0),
            drag_vel: V2D::null(),
            mass_idx: None,
            held_idx: None,
            sel_idx: None,
        }
    }

    pub fn wave(&self) -> Line<'_> {
        Line::new(
            "wave",
            PlotPoints::from_parametric_callback(move |t|
            (0.5 * (1.0 + self.phz.model.wave_amplitude * (t + self.phz.model.angle).sin()), t),
            0.0..(TAU + 0.5), // The extra 0.5 is to ensure the wave line makes it to the other side of the plot.
            64))
            .color(Color32::from_rgb(29, 179, 34))
            .style(LineStyle::Solid)
    }
}
impl eframe::App for PhyzzyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Pause").clicked() {
                    self.phz.paused = !self.phz.paused;
                }
                if ui.button("Reverse").clicked() {
                    self.phz.model.toggle_wave_dir();
                }
                if ui.button("Toggle g").clicked() {
                    self.phz.model.toggle_g();
                }
            });
        });
        egui::Panel::left("sim_settings_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(180.0)
        .show_inside(ui, |ui| {
            // Show the muscle waveform.
            let plot = Plot::new("Wavebox")
                .allow_zoom(false)
                .allow_axis_zoom_drag(false)
                .allow_scroll(false)
                .allow_drag(false)
                .include_x(0.0)
                .include_x(1.0)
                .set_margin_fraction(Vec2::ZERO)
                .show(ui, |plot_ui| {
                    plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max([0.0, 0.0], [1.0, TAU]));
                plot_ui.line(self.wave());
            });

            // Adjust the wave amplitude by dragging along the x axis.
            if plot.response.dragged() {
                let x_delta = plot.response.drag_delta().x as f64;
                let p_width = plot.response.rect.width()as f64;
                self.phz.model.wave_amplitude = (self.phz.model.wave_amplitude + x_delta / p_width).clamp(0.0, 1.0);
            }

        });
        egui::Panel::right("properties_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(200.0)
        .show_inside(ui, |ui| {
            // Sliders for wave speed, gravity, drag
            ui.vertical(|ui| {
                ui.add(Slider::new(&mut self.phz.model.wave_speed, 0.0..=30.0).text("w"));
                ui.add(Slider::new(&mut self.phz.world_cfg.gravity.y, 0.0..=-20.0).text("g"));
                ui.add(Slider::new(&mut self.phz.world_cfg.drag, 0.0..=30.0).text("d"));
            });

            // Show properties of a selected mass.
            if let Some(idx) = &self.viewport.select_idx {
                match &idx {
                    PhyzzyObject::Mass(o_idx) => {
                        let f_mass_idx = format!("Selected: Mass {}", o_idx);
                        ui.heading(f_mass_idx);
                        let mass = self.phz.model.get_mass(*o_idx);
                        let f_mass_props = format!("mass: {}, pos: {:.3?}, vel: {:.3?}", mass.m, mass.p_i, mass.vel(self.phz.dt));
                        ui.label(f_mass_props);
                    },
                    PhyzzyObject::Spring(_o_idx) => {},

                }
            }

            let sel_check = format!("{:?}", self.viewport.select_idx);
            ui.label(sel_check);
            let drag_check = format!("{:?}", self.viewport.drag_idx);
            ui.label(drag_check);

            // Show properties of a hovered mass.
            if let Some(idx) = &self.viewport.hover_idx {
                match &idx {
                    PhyzzyObject::Mass(o_idx) => {
                        let f_mass_idx = format!("Mass {}", o_idx);
                        ui.heading(f_mass_idx);
                        let mass = self.phz.model.get_mass(*o_idx);
                        let f_mass_props = format!("mass: {}, pos: {:.3?}, vel: {:.3?}", mass.m, mass.p_i, mass.vel(self.phz.dt));
                        ui.label(f_mass_props);
                    },
                    PhyzzyObject::Spring(_o_idx) => {},
                }
            }
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get time passed.
            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.last_frame = t_elapsed.as_secs_f64();
            self.phz.t_now = Instant::now();

            // Setup painter.
            self.viewport.viewer_area(&ui, &self.phz);


            // User interaction.
            let response = ui.allocate_rect(self.viewport.centered_rect, Sense::click_and_drag());
            // self.user_interact(&response);
            self.viewport.user_hover(&response, &self.phz);
            self.viewport.user_single_interact(&response, &mut self.phz);

            // Update model.
            let mut acc = self.phz.last_frame;
            while acc >= self.phz.dt {
                self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, self.phz.paused);
                acc -= self.phz.dt;
            }
            let alpha = acc / self.phz.dt;

            // Draw the background.
            ui.request_repaint();

            // Draw the model.
            self.viewport.draw_view(&ui);
            self.viewport.draw_model(&ui, &self.phz, alpha);
            self.viewport.draw_interaction(&ui, &self.phz, alpha);
        });
    }
}
