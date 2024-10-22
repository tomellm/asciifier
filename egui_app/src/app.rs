use std::path::PathBuf;

use asciifier::{
    asciifier::{Asciifier, ImageBuilder},
    error::AsciiError,
};
use eframe::App;
use egui::{
    Align, CentralPanel, Context, Frame, Image, Label, Layout, RichText, ScrollArea, SidePanel,
    Slider, Stroke, TextureHandle, TextureOptions, Ui, Vec2, Vec2b, Widget, Window,
};
use image::{DynamicImage, EncodableLayout, ImageBuffer, Rgb};

use crate::{file_selection::FileSelection, font_builder::FontBuilderControls};

pub struct AsciifierApp {
    builder: Option<ImageBuilder>,
    file_selection: FileSelection,
    texture_handle: Option<TextureHandle>,
    font_controls: FontBuilderControls,
    showing_error: Option<String>,
    zoom: f32,
}

impl App for AsciifierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_file_drag_and_drop(ctx);

        self.show_ascii_error(ctx);

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                let width_mult = if self.file_selection.showing() {
                    0.75
                } else {
                    1.
                };
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width() * width_mult);
                    ui.heading("Asciify Image");
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !self.file_selection.showing() && ui.button("Files").clicked() {
                                self.file_selection.set_showing(true);
                            };

                            ui.add(Slider::new(&mut self.zoom, 0.25..=10.).show_value(false));
                            ui.label(format!("Zoom: {:.0}%", self.zoom * 100.));
                        });
                    });
                });
                if ui.button("asciify image").clicked() {
                    if let Some(file) = self.file_selection.selected_image() {
                        match self.asciify_image(&file).cloned() {
                            Ok(image) => {
                                self.set_image(image.into(), ui);
                            }
                            Err(error) => {
                                self.showing_error = Some(format!("{error}"));
                            }
                        }
                    } else if self.file_selection.is_empty() {
                        self.showing_error =
                            Some("Please drop in some images to then asciify them.".into());
                    } else {
                        self.showing_error =
                            Some("Please click on one of the images to asciify one.".into());
                    }
                }
                self.font_controls.ui(ui);

                ScrollArea::both()
                    .max_width(ui.available_width() * width_mult)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if let Some(texture) = &self.texture_handle {
                                let image = Image::new(texture).fit_to_exact_size(
                                    [
                                        (ui.available_width() * width_mult) * self.zoom,
                                        ui.available_height() * self.zoom,
                                    ]
                                    .into(),
                                );
                                ui.add(image);
                            }
                        });
                    });
            });

            SidePanel::right("file viewer")
                .resizable(true)
                .min_width(ui.available_width() / 4.)
                .show_animated_inside(ui, self.file_selection.showing(), |ui| {
                    self.file_selection.ui(ui);
                });
        });
    }
}

impl AsciifierApp {
    fn set_image(&mut self, image: DynamicImage, ui: &mut Ui) {
        let color_image = match &image {
            DynamicImage::ImageRgb8(image) => egui::ColorImage::from_rgb(
                [image.width() as usize, image.height() as usize],
                image.as_bytes(),
            ),
            other => {
                let image = other.to_rgba8();
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_bytes(),
                )
            }
        };
        let handle =
            ui.ctx()
                .load_texture("asciified_image", color_image, TextureOptions::default());
        self.texture_handle = Some(handle);
    }

    fn show_ascii_error(&mut self, ctx: &Context) {
        let mut open = false;
        if let Some(error) = &self.showing_error {
            open = true;
            let half_screen = ctx.screen_rect() / 2.;
            Window::new("Error")
                .default_pos(half_screen.right_bottom())
                .anchor(egui::Align2([Align::Center, Align::Center]), Vec2::ZERO)
                .movable(true)
                .open(&mut open)
                .collapsible(false)
                .resizable(Vec2b::new(false, false))
                .show(ctx, |ui| {
                    let visuals = ui.style().noninteractive();
                    let text_color = visuals.text_color();

                    Frame::canvas(ui.style())
                        .fill(visuals.weak_bg_fill)
                        .inner_margin(ui.spacing().menu_margin * 6.)
                        .stroke(Stroke::NONE)
                        .show(ui, |ui| {
                            Label::new(RichText::new(error).color(text_color))
                                .selectable(false)
                                .ui(ui);
                        });
                });
        }
        if !open {
            self.showing_error = None;
        }
    }

    fn asciify_image(
        &mut self,
        file: &PathBuf,
    ) -> Result<&ImageBuffer<Rgb<u8>, Vec<u8>>, AsciiError> {
        let font_builder = self.font_controls.font_builder();
        let mut builder = Asciifier::load_image(file)?.font(|mut builder| {
            builder.copy(&font_builder);
            Ok(builder)
        })?;
        builder.convert()?;
        self.builder = Some(builder);
        Ok(self.builder.as_ref().unwrap().get_image().unwrap())
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
        use std::fmt::Write as _;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Dropping files:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.file_selection.add_files(i.raw.dropped_files.clone());
                self.file_selection.set_showing(true);
            }
        });
    }
}

impl Default for AsciifierApp {
    fn default() -> Self {
        Self {
            builder: None,
            file_selection: FileSelection::default(),
            texture_handle: None,
            font_controls: FontBuilderControls::default(),
            showing_error: None,
            zoom: 1.,
        }
    }
}
