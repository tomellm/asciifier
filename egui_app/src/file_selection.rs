use std::{collections::HashMap, path::PathBuf};

use egui::{
    Align, Color32, DroppedFile, Frame, Label, Layout, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Widget
};
use uuid::Uuid;

#[derive(Default)]
pub struct FileSelection {
    showing: bool,
    files: HashMap<Uuid, DroppedFile>,
    selected_image: Option<(Uuid, PathBuf)>,
}

impl FileSelection {
    pub fn add_files(&mut self, files: impl IntoIterator<Item = DroppedFile>) {
        self.files
            .extend(files.into_iter().map(|file| (Uuid::new_v4(), file)));
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                self.showing = false;
            }
            ui.vertical_centered(|ui| {
                if !self.files.is_empty() {
                    ui.label("Dropped Files");
                } else {
                    ui.label("Drop a File to start Asciifing");
                }
            });
        });
        ui.add_space(10.);
        ScrollArea::vertical().show(ui, |ui| {
            self.files.retain(|uuid, file| {
                Self::display_file(&mut self.selected_image, uuid, file, ui)
            });
        });
    }

    pub fn selected_image(&self) -> Option<PathBuf> {
        self.selected_image.as_ref().map(|(_, path)| path.clone())
    }

    pub fn display_file(
        selected: &mut Option<(Uuid, PathBuf)>,
        uuid: &Uuid,
        file: &DroppedFile,
        ui: &mut Ui,
    ) -> bool {
        let mut retain_file = true;
        let is_selected = selected
            .as_ref()
            .map(|(sel_uuid, _)| sel_uuid.eq(uuid))
            .unwrap_or(false);

        let response = ui
            .scope_builder(
                UiBuilder::new()
                    .id_salt(format!("file_display_{}", uuid))
                    .sense(Sense::click()),
                |ui| {
                    ui.set_max_width(ui.available_width());
                    ui.set_max_height(ui.available_width());

                    let response = ui.response();
                    let visuals = ui.style().interact(&response);
                    let text_color = visuals.text_color();

                    let mut gamma_multiply = 0.3f32;
                    let mut stroke = visuals.bg_stroke;
                    if is_selected {
                        gamma_multiply = 1f32;
                        stroke = Stroke::new(2., Color32::from_rgb(150, 150, 150));
                    }

                    Frame::canvas(ui.style())
                        .fill(visuals.bg_fill.gamma_multiply(gamma_multiply))
                        .stroke(stroke)
                        .inner_margin(ui.spacing().menu_margin)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(path) = &file.path {
                                    let path = path.to_str().unwrap();
                                    ui.image(format!("file://{path}"));
                                } else {
                                    Label::new(
                                        RichText::new("The path of this file is broken")
                                            .color(text_color),
                                    )
                                    .selectable(false)
                                    .ui(ui);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("remove").clicked() {
                                        retain_file = false;
                                    }
                                });
                            });
                        });
                },
            )
            .response;

        if response.clicked() {
            *selected = file.path.clone().map(|path| (*uuid, path));
        }
        retain_file
    }

    pub fn showing(&self) -> bool {
        self.showing
    }

    pub fn set_showing(&mut self, showing: bool) {
        self.showing = showing;
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
