use eframe::{egui::{ Color32, Pos2, Response, Stroke, Ui, Vec2 }, epaint::{ CornerRadiusF32, Rect }};
use phyzzy_rs::V2D;

use crate::phyzzy_sim::PhyzzySimulator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhyzzyObject {
    Mass(usize),
    Spring(usize),
}


pub struct PhyzzyViewport {
    pub scale: f32,
    pub centered_rect: Rect,
    pub full_area: Rect,
    pub frame_alpha: f32,
    pub drag_vel: V2D,
    pub hover_idx: Option<PhyzzyObject>,
    pub drag_idx: Option<PhyzzyObject>,
    pub select_idx: Option<PhyzzyObject>,
}

impl PhyzzyViewport{
    pub fn init() -> Self {
        Self {
            scale: 0.0,
            centered_rect: Rect::ZERO,
            full_area: Rect::ZERO,
            frame_alpha: 0.0,
            drag_vel: V2D::null(),
            hover_idx: None,
            drag_idx: None,
            select_idx: None,
        }
    }

    pub fn area_to_rect(&self, phz: &PhyzzySimulator, rect: Rect) -> (Vec2, f32) {
        let rect_sz = rect.size();
        let world_sz = Vec2::new(phz.world.area_sz.x as f32, phz.world.area_sz.y as f32);

        if world_sz.x > world_sz.y {
            let scale = rect_sz.x / world_sz.x;
            // Clamp vertical size if it gets bigger than the window's.
            if world_sz.y * scale > rect_sz.y {
                let s = rect_sz.y / world_sz.y;
                return (Vec2::new(world_sz.x * s, rect_sz.y), s);
            }

            (Vec2::new(rect_sz.x, world_sz.y * scale), scale)
        } else {
            let scale = rect_sz.y / world_sz.y;
            // Clamp horizontal size if it gets bigger than the window's
            if world_sz.x * scale > rect_sz.x {
                let s = rect_sz.x / world_sz.x;
                return (Vec2::new(rect_sz.x, world_sz.y * s), s);
            }

            (Vec2::new(world_sz.x * scale, rect_sz.y), scale)
        }
    }

    // Gets the workable area the model will be seen in.
    pub fn viewer_area(&mut self, ui: &Ui, phz: &PhyzzySimulator) {
        self.full_area = ui.max_rect();
        let (scaled_area, scale) = self.area_to_rect(&phz, self.full_area);
        self.scale = scale;

        let center_offset = Vec2::new(
            (self.full_area.width() - scaled_area.x) / 2.0,
            (self.full_area.height() - scaled_area.y) / 2.0,
        );
        let centered_min = self.full_area.min + center_offset;
        self.centered_rect = Rect::from_min_size(centered_min, scaled_area);

    }

    // Converts the internal world coordinate to a drawable coordinate in the viewport.
    pub fn world_to_panel(&self, phz_coord: &V2D) -> Pos2 {
        // Function arranged for clarity on transformation matrix being used.
        let panel_coord = phz_coord.tf_fit(
            self.scale as f64,                   self.centered_rect.max.y as f64,
            self.centered_rect.min.x as f64, -self.scale as f64);

        Pos2::new(panel_coord.x as f32, panel_coord.y as f32)
    }

    // In charge of drawing the model and viewport across the entire given area.
    pub fn draw_view(&self, ui: &Ui) {
        let outer_color = Color32::from_gray(0);
        let centered_color = Color32::from_gray(16);
        let no_radius = CornerRadiusF32::same(0.0);

        ui.painter().rect_filled(self.full_area, no_radius, outer_color);
        ui.painter().rect_filled(self.centered_rect, no_radius, centered_color);
    }
    pub fn draw_model(&self, ui: &Ui, phz: &PhyzzySimulator, alpha: f64) {
        let painter = ui.painter_at(self.centered_rect);

        // Draw springs first
        let spring_color = Color32::from_gray(255);
        let stroke = Stroke::new(1.0, spring_color);
        for spring in phz.model.get_springs() {
            let mass_a = phz.model.get_mass(spring.get_ma());
            let mass_b = phz.model.get_mass(spring.get_mb());

            // Ignoring approximation on pause prevents jitter.
            let pos_a = if !phz.paused && !mass_a.fixed {
                self.world_to_panel(&mass_a.approx_pos(alpha))
            } else {
                self.world_to_panel(&mass_a.p_i)
            };
            let pos_b = if !phz.paused && !mass_b.fixed {
                self.world_to_panel(&mass_b.approx_pos(alpha))
            } else {
                self.world_to_panel(&mass_b.p_i)
            };

            painter.line_segment([pos_a, pos_b], stroke);
        }

        // Draw masses
        let mass_color = Color32::from_rgb(29, 179, 34);

        for mass in phz.model.get_masses() {

            // Ignoring approximation on pause prevents jitter.
            let pos = if !phz.paused && !mass.fixed {
                let aprox_render = mass.approx_pos(alpha);
                self.world_to_panel(&aprox_render)

            } else {
                self.world_to_panel(&mass.p_i)
            };

            let rad = mass.r as f32 * self.scale;

            painter.circle_filled(pos, rad, mass_color);
        }
    }

    // Draw highlights and user interaction.
    pub fn draw_interaction(&self, ui: &Ui, phz: &PhyzzySimulator, alpha: f64) {
        let painter = ui.painter_at(self.centered_rect);
        if let Some(phz_idx) = &self.hover_idx {
            match phz_idx {
                PhyzzyObject::Mass(idx) => {
                    let mass = phz.model.get_mass(*idx);
                    let pos = if !phz.paused && !mass.fixed{
                        self.world_to_panel(&mass.approx_pos(alpha))
                    } else {
                        self.world_to_panel(&mass.p_i)
                    };
                    let rad = phz.model.get_mass(*idx).r as f32 * self.scale + 5.0;
                    painter.circle_stroke(pos, rad, Stroke::new(1.0, Color32::from_gray(255)));
                },
                PhyzzyObject::Spring(_idx) => {}
            }
        }

    }

    pub fn user_hover(&mut self, response: &Response, phz: &PhyzzySimulator) {
        self.hover_idx = match response.hover_pos() {
            Some(hover_coord) => {
                let m_idx = phz.model.get_masses().iter().position(|mass| {
                    let mass_panel_pos = self.world_to_panel(&mass.p_i);
                    let rad_detect = ((mass.r * self.scale as f64) + 10.0) as f32;
                    (hover_coord - mass_panel_pos).length() < rad_detect
                });

                match m_idx {
                    Some(idx) => Some(PhyzzyObject::Mass(idx)),
                    None => None,
                }
            },
            None => { None },
        };
    }
    pub fn user_single_interact(&mut self, response: &Response, phz: &mut PhyzzySimulator ) {
        match response.interact_pointer_pos() {
            // Pointer held down
            Some(interact_coord) => {
                // Check if pointer is held down on a mass.
                let mass_idx = phz.model.get_masses().iter().position(|mass| {
                    let mass_panel_pos = self.world_to_panel(&mass.p_i);
                    let detection_rad = ((mass.r * self.scale as f64) + 10.0) as f32;
                    (mass_panel_pos - interact_coord).length() < detection_rad
                });

                self.select_idx = if let Some(idx) = mass_idx {
                    Some(PhyzzyObject::Mass(idx))

                } else { None };

                if response.drag_started() {
                    self.drag_idx = self.select_idx;
                    if let Some(obj) = self.drag_idx {
                        match obj {
                            PhyzzyObject::Mass(idx) => {
                                phz.model.hold_mass(idx);
                            },
                            PhyzzyObject::Spring(_idx) => {},
                        }
                    }
                }
                if response.drag_stopped() {
                    if let Some(obj) = self.drag_idx {
                        match obj {
                            PhyzzyObject::Mass(idx) => {
                                phz.model.release_mass(idx);
                                phz.model.set_mass_vel(idx, self.drag_vel, phz.dt);
                            },
                            PhyzzyObject::Spring(_idx) => {},
                        }
                    }
                    self.drag_idx = None;
                }
                if let Some(o_idx) = &self.drag_idx && response.dragged() {
                    match o_idx {
                        PhyzzyObject::Mass(idx) => {
                            let drag_delta = response.drag_delta();
                            let vel = V2D::new(drag_delta.x as f64 / self.scale as f64,
                                               -drag_delta.y as f64 / self.scale as f64) / phz.last_frame;

                            let m_new_pos = phz.model.get_mass(*idx).p_i + vel * phz.last_frame;
                            phz.model.set_mass_pos(*idx, m_new_pos);
                            self.drag_vel = if !phz.paused { vel } else { V2D::null() };
                        },
                        PhyzzyObject::Spring(_idx) => {},
                    }
                }
            },
            None => {}
        }
    }
}
