use eframe::egui::Response;
use phyzzy_rs::V2D;

use crate::phyzzy_sim::PhyzzySimulator;

#[derive(PartialEq, Eq)]
pub enum PhyzzyObject {
    Mass(usize),
    Spring(usize),
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
                    Some(idx) => Some(PhyzzyObject::Mass(idx)),
                    None => None,
                }
            },
            None => { None },
        };
    }
    pub fn user_single_select(&mut self, response: &Response, phz: &PhyzzySimulator) {
        let sel_idx = match response.interact_pointer_pos() {
            // Detect if user is holding down the mouse over a mass.
            Some(interact_coord) => {
                let m_idx = phz.model.get_masses().iter().position(|mass| {
                    let mass_panel_pos = phz.world_to_panel(&mass.p_i);
                    let rad_detect = (mass.r * phz.scaling + 5.0) as f32;
                    (interact_coord - mass_panel_pos).length() <= rad_detect
                });
                match m_idx {
                    Some(idx) => Some(PhyzzyObject::Mass (idx)),
                    None => None,
                }
            },
            None => None,
        };

        // Before assigning it selected, check if the user isn't already dragging the same mass.

        // When there's a mass already selected.
        if self.selection.len() > 0 {
            match sel_idx {
                // When the user is holding a mass.
                Some(idx) => match idx {
                    PhyzzyObject::Mass (o_idx) => {
                        let contains = self.selection.contains(&PhyzzyObject::Mass(o_idx));
                        // Not selected, or a different mass is selected.
                        if !contains && !response.dragged(){
                            self.selection.clear();
                            self.selection.push(PhyzzyObject::Mass(o_idx));
                        }
                    },
                    PhyzzyObject::Spring (_o_idx) => {},
                },
                // When the user clicked on empty space.
                None => {
                    self.selection.clear();
                }
            }

        // When there's nothing selected yet.
        } else {
            match sel_idx {
                // when the user clicked an object
                Some(idx) => {
                    match idx {
                        PhyzzyObject::Mass (idx) => {
                            self.selection.push(PhyzzyObject::Mass (idx));
                        },
                        PhyzzyObject::Spring (_idx) => {},
                    }
                },
                None => {}
            }
        }

    }
    pub fn user_drag(&mut self, response: &Response, phz: &PhyzzySimulator) {

    }
}
