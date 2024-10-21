use asciifier::{
    asciifier::FontBuilder,
    chars::font_handler::{CharAlignment, CharDistributionType, CharacterBackground},
};
use egui::{ComboBox, DragValue, Ui};

const DEFAULT_CHARS: &str =
    "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM∇∕∑∏∇∆∃∫∬∮≋⊋⊂⊃⊞⊟⊠⊪⊩∸∷∶∶∵∴∾⊢⊯⊮⊭⊬⊫⊪⊩⊨⊧⊦⊥⊤⊣⊡";

pub struct FontBuilderControls {
    font_builder: FontBuilder,
    chars: String,
}

impl FontBuilderControls {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Select characters to use");
            ui.text_edit_singleline(&mut self.chars);
            if ui.button("set default").clicked() {
                self.chars = DEFAULT_CHARS.into();
            }
        });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.font_builder.font_height).range(10..=100000));
            ComboBox::new("char_alignment_control", "Char Alignment")
                .selected_text(format!("{:?}", self.font_builder.alignment))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.font_builder.alignment,
                        CharAlignment::Left,
                        "Left",
                    );
                    ui.selectable_value(
                        &mut self.font_builder.alignment,
                        CharAlignment::Center,
                        "Center",
                    );
                    ui.selectable_value(
                        &mut self.font_builder.alignment,
                        CharAlignment::Right,
                        "Right",
                    );
                });
            ComboBox::new("char_distribution", "Char Lum Distribution")
                .selected_text(format!("{:?}", self.font_builder.distribution))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.font_builder.distribution,
                        CharDistributionType::Exact,
                        "Exact",
                    );
                    ui.selectable_value(
                        &mut self.font_builder.distribution,
                        CharDistributionType::ExactAdjustedBlacks,
                        "Adjust Blacks",
                    );
                    ui.selectable_value(
                        &mut self.font_builder.distribution,
                        CharDistributionType::ExactAdjustedWhites,
                        "Adjust Whites",
                    );
                });
            ComboBox::new("char_background", "Char Background")
                .selected_text(format!("{:?}", self.font_builder.background))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.font_builder.background,
                        CharacterBackground::White,
                        "White",
                    );
                    ui.selectable_value(
                        &mut self.font_builder.background,
                        CharacterBackground::Black,
                        "Black",
                    );
                });
        });
    }

    pub fn font_builder(&mut self) -> FontBuilder {
        self.font_builder.set_chars(self.chars.clone()).clone()
    }
}

impl Default for FontBuilderControls {
    fn default() -> Self {
        Self {
            font_builder: FontBuilder::new().unwrap(),
            chars: DEFAULT_CHARS.into(),
        }
    }
}
