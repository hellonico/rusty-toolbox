use eframe::egui;
use eframe::egui::{Color32, FontId, Ui, Vec2, Widget};

pub struct RoundedLabel {
    pub text: String,
    pub text_color: Color32,
    pub background_color: Color32,
    pub rounding: f32,
    pub padding: Vec2,
    pub margin: Vec2, // New field for margins
}

impl RoundedLabel {
    pub fn new(
        text: &str,
        text_color: Color32,
        background_color: Color32,
        rounding: f32,
        padding: Vec2,
        margin: Vec2,
    ) -> Self {
        Self {
            text: text.to_string(),
            text_color,
            background_color,
            rounding,
            padding,
            margin,
        }
    }

    pub fn blue_bubble(text: &str) -> Self {
        Self::new(
            text,
            Color32::WHITE,
            Color32::from_rgb(100, 150, 250),
            20.0,
            Vec2::new(12.0, 6.0), // Padding
            Vec2::new(0.0, 50.0), // Right margin
        )
    }

    pub fn orange_bubble(text: &str) -> Self {
        Self::new(
            text,
            Color32::WHITE,
            Color32::from_rgb(255, 165, 0),
            20.0,
            Vec2::new(12.0, 6.0), // Padding
            Vec2::new(50.0, 0.0), // Left margin
        )
    }
}

impl Widget for &RoundedLabel {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let RoundedLabel {
            text,
            text_color,
            background_color,
            rounding,
            padding,
            margin,
        } = self;

        // Compute available width, accounting for margins
        let max_width = ui.available_width() - margin.x;

        // Layout the text with wrapping enabled within the available width
        let galley = ui.painter().layout(
            text.clone(),
            FontId::proportional(11.0),
            *text_color,
            max_width - padding.x * 2.0,
        );

        // Compute the desired size for the bubble, including the margins
        let desired_size = Vec2::new(max_width-100.0, galley.size().y + padding.y * 2.0);

        // Allocate space for the widget
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            // Adjust for horizontal margins
            let adjusted_rect = egui::Rect::from_min_size(
                rect.min + Vec2::new(margin.x * 0.5, 0.0),
                Vec2::new(max_width, rect.height()),
            );

            // Draw the rounded rectangle background
            ui.painter().rect_filled(
                adjusted_rect,
                egui::Rounding::same(*rounding),
                *background_color,
            );

            // Draw the wrapped text inside the bubble with padding
            let text_pos = adjusted_rect.min + *padding;
            ui.painter().galley(text_pos, galley,*text_color);
        }

        response
    }
}
