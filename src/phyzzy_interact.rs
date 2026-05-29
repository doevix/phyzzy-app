use eframe::egui::Response;
use phyzzy_rs::{Model, V2D};

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

    pub fn user_hover(response: &Response, model: &Model) {

    }
    pub fn user_select(response: &Response, model: &Model) {

    }
}
