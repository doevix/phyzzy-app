use phyzzy_rs::{ self, V2D };
use eframe::egui::{ self, MenuBar, Pos2, Sense, Slider, Vec2, Rect };
use rfd::FileDialog;
use std::{ time::Instant };

use crate::phyzzy_io::PhyzzyIO;
use crate::phyzzy_sim::PhyzzySimulator;
use crate::phyzzy_viewport::{ PhyzzyViewport };
use crate::phyzzy_wavebox::PhyzzyWavebox;

pub struct PhyzzyApp {
    pub phz: PhyzzySimulator,
    pub viewport: PhyzzyViewport,
    pub wavebox: PhyzzyWavebox,
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
            wavebox: PhyzzyWavebox::init(),
            pointer_pos: None,
            pointer_interact_pos: None,
            pointer_drag_delta: Vec2::new(0.0, 0.0),
            drag_vel: V2D::null(),
            mass_idx: None,
            held_idx: None,
            sel_idx: None,
        }
    }
}
impl eframe::App for PhyzzyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load").clicked() {
                        let files = FileDialog::new()
                            .add_filter("text", &["txt", "json"])
                            .add_filter("json", &["txt", "json"])
                            .pick_file();

                        if let Some(file) = files {
                            let filename = String::from(file.to_str().unwrap_or(""));
                            let new_model = PhyzzyIO::import(&filename, self.phz.dt);
                            match new_model {
                                Ok(model) => {
                                    let new_phz = PhyzzySimulator {
                                        last_frame: 0.0,
                                        dt: self.phz.dt,
                                        world: model.world,
                                        world_cfg: model.world_config,
                                        model: model.model,
                                        model_meta: model.meta,
                                        scaling: 0.0,
                                        t_now: Instant::now(),
                                        screen_rect: Rect {
                                            min: Pos2 { x: 0.0, y: 0.0 },
                                            max: Pos2 { x: 0.0, y: 0.0 }
                                        },
                                        paused: false,
                                    };
                                    self.phz = new_phz;
                                },
                                Err(_) => {}
                            }
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
            });
            ui.separator();
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
                ui.separator();
                ui.add(Slider::new(&mut self.phz.model.wave_speed, 0.0..=40.0).step_by(0.1).update_while_editing(false).text("w"));
                ui.add(Slider::new(&mut self.phz.world_cfg.gravity.y, 0.0..=-20.0).step_by(0.1).update_while_editing(false).text("g"));
                ui.add(Slider::new(&mut self.phz.world_cfg.drag, 0.0..=30.0).step_by(0.1).update_while_editing(false).text("d"));
            });
        });
        egui::Panel::bottom("info").show_inside(ui, |ui| {
            let str_hover = match  self.viewport.cursor_location {
                Some(coord) => format!("({:.3}, {:.3})", coord.x, coord.y),
                None => format!("(---, ---)"),
            };
            ui.label(str_hover);
        });
        egui::Panel::left("sim_settings_panel")
        .resizable(true)
        .min_size(150.0)
        .max_size(180.0)
        .show_inside(ui, |ui| {
            // Show the muscle waveform.
            self.wavebox.show(ui, &mut self.phz);
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
