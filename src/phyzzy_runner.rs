use phyzzy_rs::{ self, V2D };
use eframe::{ egui::{ self, Color32, Pos2, Rect, Response, Sense, Slider, Vec2 }, epaint::CornerRadiusF32 };
use egui_plot::{ self, Line, LineStyle, Plot, PlotPoints };
use core::f64;
use std::time::Instant;

use crate::phyzzy_sim::PhyzzySimulator;

pub struct PhyzzyApp {
    pub phz: PhyzzySimulator,
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
        let two_pi = 2.0 * f64::consts::PI;
        Line::new(
            "wave",
            PlotPoints::from_parametric_callback(move |t|
            (0.5 * (1.0 + self.phz.model.wave_amplitude * (t + self.phz.model.angle).sin()), t),
            0.0..two_pi,
            64))
            .color(Color32::from_rgb(29, 179, 34))
            .style(LineStyle::Solid)
    }
}
impl eframe::App for PhyzzyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("sim_settings_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(180.0)
        .show_inside(ui, |ui| {
            ui.heading(&self.phz.model_meta.name);
            let creator_string = format!("by {}", self.phz.model_meta.creator);
            ui.label(creator_string);
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

            let plot = Plot::new("Wavebox")
                .allow_zoom(false)
                .allow_axis_zoom_drag(false)
                .allow_scroll(false)
                .allow_drag(false)
                .include_x(0.0)
                .include_x(1.0)
                .set_margin_fraction(Vec2::ZERO)
                .show(ui, |plot_ui| {
                    plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max([0.0, 0.0], [1.0, 6.28]));
                plot_ui.line(self.wave());
            });

            if plot.response.dragged() {
                if plot.response.drag_delta().x > 0.0 {
                    self.phz.model.wave_amplitude += 0.05;
                    self.phz.model.wave_amplitude = self.phz.model.wave_amplitude.clamp(0.0, 1.0);
                } else if plot.response.drag_delta().x < 0.0 {
                    self.phz.model.wave_amplitude -= 0.05;
                    self.phz.model.wave_amplitude = self.phz.model.wave_amplitude.clamp(0.0, 1.0);
                }
            }

        });
        egui::Panel::right("properties_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(200.0)
        .show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.add(Slider::new(&mut self.phz.model.wave_speed, 0.0..=30.0).text("w"));
                ui.add(Slider::new(&mut self.phz.world_cfg.gravity.y, 0.0..=-20.0).text("g"));
                ui.add(Slider::new(&mut self.phz.world_cfg.drag, 0.0..=30.0).text("d"));
            });

            if let Some(idx) = self.sel_idx {
                let f_mass_idx = format!("Selected: Mass {}", idx);
                ui.heading(f_mass_idx);
                let mass = self.phz.model.get_mass(idx);
                let f_mass_props = format!("mass: {}, pos: {:.3?}, vel: {:.3?}", mass.m, mass.p_i, mass.vel(self.phz.dt));
                ui.label(f_mass_props);
            }

            if let Some(idx) = self.mass_idx {
                let f_mass_idx = format!("Mass {}", idx);
                ui.heading(f_mass_idx);
                let mass = self.phz.model.get_mass(idx);
                let f_mass_props = format!("mass: {}, pos: {:.3?}, vel: {:.3?}", mass.m, mass.p_i, mass.vel(self.phz.dt));
                ui.label(f_mass_props);
            }
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get time passed.
            let t_elapsed = self.phz.t_now.elapsed();
            self.phz.last_frame = t_elapsed.as_secs_f64();
            self.phz.t_now = Instant::now();

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

            let response = ui.allocate_rect(centered_rect, Sense::click_and_drag());

            // User interaction.
            self.user_interact(&response);

            // Update model.
            let mut acc = self.phz.last_frame;
            while acc >= self.phz.dt {
                self.phz.model.step(self.phz.dt, &self.phz.world, &self.phz.world_cfg, self.phz.paused);
                acc -= self.phz.dt;
            }
            let alpha = acc / self.phz.dt;

            // Draw the background.
            ui.request_repaint();
            let no_radius = CornerRadiusF32::same(0.0);
            ui.painter().rect_filled(full_area, no_radius, Color32::from_gray(0));
            painter.rect_filled(self.phz.screen_rect, no_radius, Color32::from_gray(16));

            // Draw the model.
            self.phz.draw_model(&painter, alpha, self.mass_idx);
        });
    }
}

impl PhyzzyApp {
    pub fn user_interact(&mut self, response: &Response) {
        match response.hover_pos() {
            Some(pos) => {
                self.pointer_pos = Some(pos);
                let mut m_idx: Option<usize> = None;
                for (idx, mass) in self.phz.model.get_masses().iter().enumerate() {
                    let bound_rad = ((mass.r * self.phz.scaling) + 10.0) as f32;
                    let mass_pos = self.phz.world_to_panel(&mass.p_i);

                    if (pos - mass_pos).length() < bound_rad {
                        m_idx = Some(idx);
                        break;
                    }
                }

                self.mass_idx = m_idx;
                m_idx
            },
            None => {
                self.pointer_pos = None;
                self.mass_idx = None;
                None
            }
        };

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut m_idx: Option<usize> = None;

                for (idx, mass) in self.phz.model.get_masses().iter().enumerate() {
                    let bound_rad = ((mass.r * self.phz.scaling) + 10.0) as f32;
                    let mass_pos = self.phz.world_to_panel(&mass.p_i);

                    if (pos - mass_pos).length() < bound_rad {
                        m_idx = Some(idx);
                        break;
                    }
                }

                self.sel_idx = m_idx;
            }
        }

        match response.interact_pointer_pos() {
            Some(pos) => {
                self.pointer_interact_pos = Some(pos);

                // Find the mass that's being clicked on.
                let mut m_idx: Option<usize> = None;
                for (idx, mass) in self.phz.model.get_masses().iter().enumerate() {
                    let bound_rad = ((mass.r * self.phz.scaling) + 10.0) as f32;
                    let mass_pos = self.phz.world_to_panel(&mass.p_i);

                    if (pos - mass_pos).length() < bound_rad {
                        m_idx = Some(idx);
                        break;
                    }
                }

                // If a mass was found for interaction.
                if let Some(idx) = m_idx {
                    // If the mass was already being used for interaction.
                    match self.held_idx {
                        None => {
                            self.held_idx = m_idx;
                            // self.drag_vel = V2D::null();
                            self.phz.model.hold_mass(idx);
                        },
                        Some(_) => {},
                    }
                }
            },
            None => {
                self.pointer_interact_pos = None;
                if let Some(idx) = self.held_idx {
                    self.phz.model.release_mass(idx);
                    self.held_idx = None;
                    self.drag_vel = V2D::null();
                }
            }
        };

        self.pointer_drag_delta = response.drag_delta();

        if response.dragged() {
            if let (Some(idx), Some(cursor)) = (self.held_idx, response.interact_pointer_pos()) {
                let p_x = (cursor.x as f64 - self.phz.screen_rect.min.x as f64) / self.phz.scaling;
                let p_y = (self.phz.screen_rect.max.y as f64 - cursor.y as f64) / self.phz.scaling;
                let target = V2D::new(p_x, p_y);

                self.phz.model.set_mass_pos(idx, target);

                let drag_delta = response.drag_delta();
                let frame_vel = V2D::new(
                    drag_delta.x as f64 / self.phz.scaling,
                    -drag_delta.y as f64 / self.phz.scaling
                ) / self.phz.last_frame;

                // Since the drag velocity gets lost on release, hold on to last non-stopped drag vel.
                if !self.phz.paused {
                    if !response.drag_stopped() { self.drag_vel = frame_vel; }
                } else {
                    self.drag_vel = V2D::null();
                }
            }
        }

        if response.drag_stopped() {
            if let Some(idx) = self.held_idx {
                self.phz.model.set_mass_vel(idx, self.drag_vel, self.phz.dt);
                self.phz.model.release_mass(idx);
                self.held_idx = None;
                self.drag_vel = V2D::null();
            }
        }
    }
}



