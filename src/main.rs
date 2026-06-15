pub mod phyzzy_io;
pub mod phyzzy_app;
pub mod phyzzy_sim;
pub mod phyzzy_viewport;
pub mod phyzzy_wavebox;
pub mod phyzzy_menu;

use eframe::egui::{ Pos2, Rect };
use std::time::Instant;

use phyzzy_io::PhyzzyIO;
use phyzzy_app::PhyzzyApp;
use phyzzy_sim::PhyzzySimulator;


const FIXED_DT: f64 = 0.001;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let filename = String::from("models/shapes.json");

    let import_result = PhyzzyIO::import(&filename, FIXED_DT);
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
                    max: Pos2 { x: 0.0, y: 0.0 }
                },
                paused: false,
            }
        },
        Err(_) => PhyzzySimulator::new(),
    };

    // Run the model
    let _ = eframe::run_native("Phyzzy", native_options, Box::new(|cc| Ok(Box::new(PhyzzyApp::new(cc, phz)))));
}


