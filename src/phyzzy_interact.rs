use eframe::egui::Response;
use phyzzy_rs::V2D;

use crate::phyzzy_sim::PhyzzySimulator;

pub enum PhyzzyObject {
    Mass { idx: usize },
    Spring { idx: usize },
}

pub struct PhyzzyInteract {
    pub drag_vel: V2D,
    pub selection: Vec<PhyzzyObject>,
    pub hover_idx: Option<PhyzzyObject>,
}

impl PhyzzyInteract {
    pub fn init() -> Self {
        Self {
            drag_vel: V2D::null(),
            selection: Vec::new(),
            hover_idx: None,
        }
    }

    pub fn user_hover(&mut self, response: &Response, phz: &PhyzzySimulator) {
        self.hover_idx = match response.hover_pos() {
            Some(hover_coord) => {
                let m_idx = phz.model.get_masses().iter().position(|mass| {
                    let mass_panel_pos = phz.world_to_panel(&mass.p_i);
                    let rad_detect = (mass.r * phz.scaling + 5.0) as f32;
                    (hover_coord - mass_panel_pos).length() <= rad_detect
                });

                match m_idx {
                    Some(idx) => Some(PhyzzyObject::Mass { idx }),
                    None => None,
                }
            },
            None => { None },
        };
    }
    pub fn user_select(&mut self, response: &Response, phz: &PhyzzySimulator) {

    }
    pub fn user_drag(&mut self, response: &Response, phz: &PhyzzySimulator) {

    }
}
