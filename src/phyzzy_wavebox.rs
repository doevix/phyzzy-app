use std::f64::consts::TAU;

use eframe::egui::{ Color32, Ui, Vec2 };
use egui_plot::{ Plot, PlotBounds, PlotPoints, LineStyle, Line };

use crate::phyzzy_sim::PhyzzySimulator;


pub struct PhyzzyWavebox {
    pub wave_color: Color32,
}

impl PhyzzyWavebox {
    pub fn init() -> Self {
        return Self {
            wave_color: Color32::from_rgb(29, 179, 34),
        }
    }

    pub fn wave(&self, phz: &PhyzzySimulator) -> Line<'_> {
        Line::new(
            "wave",
            PlotPoints::from_parametric_callback(move |t|
            (0.5 * (1.0 + phz.model.wave_amplitude * (t + phz.model.angle).sin()), t),
            0.0..(TAU + 0.5), // The extra 0.5 is to ensure the wave line makes it to the other side of the plot.
            64))
            .color(self.wave_color)
            .style(LineStyle::Solid)
    }

    pub fn show(&self, ui: &mut Ui, phz: &mut PhyzzySimulator) {
        let plot = Plot::new("Wavebox")
            .allow_zoom(false)
            .allow_axis_zoom_drag(false)
            .allow_scroll(false)
            .allow_drag(false)
            .include_x(0.0)
            .include_x(1.0)
            .set_margin_fraction(Vec2::ZERO)
            .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, 0.0], [1.0, TAU]));
            plot_ui.line(self.wave(phz));
        });

        // Adjust the wave amplitude by dragging along the x axis.
        if plot.response.dragged() {
            let x_delta = plot.response.drag_delta().x as f64;
            let p_width = plot.response.rect.width()as f64;
            phz.model.wave_amplitude = (phz.model.wave_amplitude + x_delta / p_width).clamp(0.0, 1.0);
        }
    }
}
