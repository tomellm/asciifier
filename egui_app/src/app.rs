use std::path::PathBuf;

use asciifier::{
    asciifier::{Asciifier, ImageBuilder},
    error::AsciiError,
};
use eframe::App;
use egui::{
    Align, CentralPanel, Image, Layout, Pos2, Rect, Response, ScrollArea, SidePanel, TextureHandle,
    TextureOptions, Ui,
};
use image::{DynamicImage, EncodableLayout, ImageBuffer, Rgb};

use crate::{file_selection::FileSelection, font_builder::FontBuilderControls};

#[derive(Default)]
pub struct AsciifierApp {
    builder: Option<ImageBuilder>,
    file_selection: FileSelection,
    texture_handle: Option<TextureHandle>,
    font_controls: FontBuilderControls,
}

impl App for AsciifierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_file_drag_and_drop(ctx);
        CentralPanel::default().show(ctx, |ui| {
            
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Asciify Image");
                    if !self.file_selection.showing() {
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Files").clicked() {
                                    self.file_selection.set_showing(true);
                                }
                            });
                        });
                    };
                });
                if ui.button("asciify image").clicked() {
                    if let Some(file) = self.file_selection.selected_image() {
                        let image = self.asciify_image(&file).unwrap().clone();
                        self.set_image(image.into(), ui);
                    }
                }
                self.font_controls.ui(ui);

                ScrollArea::both().show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        if let Some(texture) = &self.texture_handle {
                            let width_mult = if self.file_selection.showing() { 0.75 } else { 1. };
                            let image = Image::new(texture).max_size(
                                [ui.available_width() * width_mult, ui.available_height()].into(),
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
