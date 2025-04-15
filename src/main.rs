use csv::ReaderBuilder;
use eframe::egui;
use rand::rng;
use rand::seq::IndexedRandom;
use serde::Deserialize;

const WINE_DATA_CSV: &str = include_str!("../WineDataset.csv");

#[derive(Debug, Deserialize)]
struct WineRecord {
    #[serde(rename = "Grape")]
    grape: String,

    #[serde(rename = "Characteristics")]
    characteristics: String,
}

fn load_csv_data_from_str(data: &str) -> Result<Vec<WineRecord>, csv::Error> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(data.as_bytes());
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: WineRecord = result?;
        records.push(record);
    }
    Ok(records)
}


struct WineFermentationApp {
    wine_data: Vec<WineRecord>,
    grape_type: String,
    fermentation_days: String,
    container_type: String,
    sugar_content: String,
    temperature: String,
    climate: String,

    result_text: String,
}

impl WineFermentationApp {
    fn new(wine_data: Vec<WineRecord>) -> Self {
        Self {
            wine_data,
            grape_type: String::new(),
            fermentation_days: String::new(),
            container_type: String::new(),
            sugar_content: String::new(),
            temperature: String::new(),
            climate: String::new(),
            result_text: String::new(),
        }
    }

    fn simulate(&mut self) {
        let fermentation_days: i32 = self.fermentation_days.trim().parse().unwrap_or_default();
        let user_sugar_input: i32 = self.sugar_content.trim().parse().unwrap_or_default();
        let temperature: f64 = self.temperature.trim().parse().unwrap_or_default();

        let (sugar_mod, _acidity_mod, tannin_mod) = match self.climate.to_lowercase().as_str() {
            "cool" => (0.90, 1.10, 1.00),
            "moderate" => (1.00, 1.00, 1.00),
            "warm" => (1.10, 0.90, 1.10),
            _ => (1.00, 1.00, 1.00),
        };

        let sugar_content = (user_sugar_input as f64) * sugar_mod;

        let conversion_factor = 16.83;
        let potential_abv = sugar_content / conversion_factor;

        if temperature < 5.0 || temperature > 40.0 {
            self.result_text =
                "Fermentation failed: temperature out of range for yeast activity.".to_owned();
            return;
        }

        let ref_temp = 20.0;
        let k_ref = 0.20;
        let q10: f64 = 2.0;
        let k = k_ref * q10.powf((temperature - ref_temp) / 10.0);

        let mut fraction_fermented = 1.0 - (-k * (fermentation_days as f64)).exp();
        if fraction_fermented > 1.0 {
            fraction_fermented = 1.0;
        }

        let sugar_consumed = fraction_fermented * sugar_content;
        let mut actual_abv = sugar_consumed / conversion_factor;

        let max_abv = 15.0;
        if actual_abv > max_abv {
            actual_abv = max_abv;
            let sugar_consumed_capped = max_abv * conversion_factor;
            fraction_fermented = sugar_consumed_capped / sugar_content;
        }

        let residual_sugar = sugar_content - sugar_consumed;
        let sweetness_description = if residual_sugar > 35.0 {
            "extremely sweet"
        } else if residual_sugar > 20.0 {
            "noticeably sweet"
        } else if residual_sugar > 5.0 {
            "with just a subtle hint of sweetness"
        } else {
            "bone dry"
        };

        let body_description = if actual_abv > 12.0 {
            "full-bodied"
        } else if actual_abv >= 10.0 {
            "medium-bodied"
        } else {
            "light-bodied"
        };
        
        let alcohol_level = if actual_abv <= 1.0 {
            "extremely low"
        } else if actual_abv < 5.0 {
            "very low"
        } else if actual_abv < 10.0 {
            "low"
        } else if actual_abv < 13.5 {
            "moderate"
        } else if actual_abv < 15.0 {
            "high"
        } else if actual_abv < 20.0 {
            "very high"
        } else {
            "extremely high"
        };

        let tannin_base = match self.grape_type.to_lowercase().as_str() {
            "cabernet sauvignon" => "robust, high tannins",
            "merlot" => "smooth, moderate tannins",
            "pinot noir" => "delicate, low tannins",
            "syrah" | "shiraz" => "moderate tannins",
            "tempranillo" => "moderate tannins",
            "zinfandel" => "spicy, moderately high tannins",
            "sangiovese" => "moderate tannins",
            "chardonnay" => "minimal tannins",
            "sauvignon blanc" => "minimal tannins",
            "riesling" => "very minimal tannins",
            _ => "unknown tannin levels",
        };

        let tannin_level = if tannin_mod > 1.0 {
            format!("{} (slightly accentuated by the warm climate)", tannin_base)
        } else if tannin_mod < 1.0 {
            format!(
                "{} (somewhat less pronounced in the cool climate)",
                tannin_base
            )
        } else {
            tannin_base.to_string()
        };

        let acidity_description = match self.climate.to_lowercase().as_str() {
            "cool" => "high",
            "moderate" => "moderate",
            "warm" => "low",
            _ => "unknown",
        };

        let matches: Vec<&WineRecord> = self
            .wine_data
            .iter()
            .filter(|record| record.grape.eq_ignore_ascii_case(&self.grape_type))
            .collect();
        let grape_characteristics = if matches.is_empty() {
            "unknown flavor profile".to_owned()
        } else {
            let mut rng = rng();
            matches.choose(&mut rng).unwrap().characteristics.clone()
        };

        let container_lower = self.container_type.to_lowercase();
        let container_note = match container_lower.as_str() {
            "oak barrel" => "woody, oaky undertones",
            "steel tank" => "a pristine, clean character",
            "clay amphora" => "earthy nuances",
            _ => "a distinct vessel charm",
        };

        let climate = self.climate.to_lowercase();
        self.result_text = format!(
            "Your {} wine was fermented over {} days in a {} that adds {}. \
             The initial sugar level was {:.1} g/L (adjusted for a {} climate), which could have reached a potential of {:.1}% ABV.\n\n\
             Fermenting at {}°C, about {:.1}% of that potential was met, resulting in a final ABV of {:.1}%% and leaving behind a residual sugar of {:.1} g/L, making it {}.\n\n\
             The wine is {} in body, with {} tannins and {} acidity. It shows hints of {} in its flavor profile.\n\n\
             The alcohol content is classified as {}.\n\n\
             Enjoy your wine.",
            self.grape_type,
            fermentation_days,
            self.container_type,
            container_note,
            sugar_content,
            climate,
            potential_abv,
            temperature,
            fraction_fermented * 100.0,
            actual_abv,
            residual_sugar,
            sweetness_description,
            body_description,
            tannin_level,
            acidity_description,
            grape_characteristics.to_ascii_lowercase(),
            alcohol_level
        );
    }
}

impl eframe::App for WineFermentationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Wine Fermentation Simulator");

            ui.label("Grape Type:");
            egui::ComboBox::from_label("Select a Grape")
                .selected_text(&self.grape_type)
                .width(200.0)
                .show_ui(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for grape in &[
                                "Cabernet Sauvignon",
                                "Merlot",
                                "Pinot Noir",
                                "Chardonnay",
                                "Sauvignon Blanc",
                                "Riesling",
                                "Syrah",
                                "Shiraz",
                                "Zinfandel",
                                "Tempranillo",
                                "Sangiovese",
                            ] {
                                ui.selectable_value(
                                    &mut self.grape_type,
                                    grape.to_string(),
                                    *grape,
                                );
                            }
                        });
                });

            ui.label("Fermentation Days (Usually 5-21):");
            ui.text_edit_singleline(&mut self.fermentation_days);

            ui.label("Container Type:");
            egui::ComboBox::from_label("Select Container")
                .selected_text(&self.container_type)
                .width(200.0)
                .show_ui(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for container in &["Oak Barrel", "Steel Tank", "Clay Amphora"] {
                                ui.selectable_value(
                                    &mut self.container_type,
                                    container.to_string(),
                                    *container,
                                );
                            }
                        });
                });

            // 5) Climate dropdown:
            ui.label("Climate:");
            egui::ComboBox::from_label("Select a Climate")
                .selected_text(&self.climate)
                .width(200.0)
                .show_ui(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for climate_option in &["Cool", "Moderate", "Warm"] {
                                ui.selectable_value(
                                    &mut self.climate,
                                    climate_option.to_string(),
                                    *climate_option,
                                );
                            }
                        });
                });

            ui.label("Sugar Content (g/L) (Usually 180g-300g):");
            ui.text_edit_singleline(&mut self.sugar_content);

            ui.label("Temperature (°C) (Usually 10.0°C to 30.0°C):");
            ui.text_edit_singleline(&mut self.temperature);

            if ui.button("Simulate Wine Fermentation").clicked() {
                self.simulate();
            }

            ui.separator();
            ui.label("Results:");
            ui.text_edit_multiline(&mut self.result_text);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    let wine_data = match load_csv_data_from_str(WINE_DATA_CSV) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Could not load CSV: {}", err);
            Vec::new()
        }
    };

    eframe::run_native(
        "Wine Fermentation Simulator",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(WineFermentationApp::new(wine_data)))),
    )
}
