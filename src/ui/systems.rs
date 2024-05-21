use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui::{self, widgets, Color32, RichText}};
use crate::{visualizer::components::{AudioVisualizerRestructureEvent, AudioVisualizerSettings, VisualilzerType, WindowFunction}, AdvancedSettings, AdvancedSettingsChangeEvent};

pub fn settings_ui(
    mut contexts: EguiContexts,
    mut audio_visualizer_restructure_event_writer: EventWriter<AudioVisualizerRestructureEvent>,
    mut advanced_settings_change_event_writer: EventWriter<AdvancedSettingsChangeEvent>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut advanced_settings: ResMut<AdvancedSettings>,
    mut clear_color: ResMut<ClearColor>,
    visualizer_type_state: Res<State<VisualilzerType>>,
    mut visualizer_type_next_state: ResMut<NextState<VisualilzerType>>
) {
    egui::Window::new("Audio Visualizer Settings")
        .resizable(true)
        .vscroll(true)
        .max_width(265.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(RichText::new("Changes to values that are red need to be manually applied!").color(egui::Color32::RED).heading());
            ui.add(widgets::Separator::default());

            let mut visualizer_type_selection = visualizer_type_state.get();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Select visualizer type:").color(egui::Color32::YELLOW));
                egui::ComboBox::from_id_source("visualizer-type")
                    .selected_text(format!("{:?}", visualizer_type_selection))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut visualizer_type_selection, &VisualilzerType::FrequencyVisualizer, "Frequency Visualizer");
                        ui.selectable_value(&mut visualizer_type_selection, &VisualilzerType::SpectrumVisualizer, "Spectrum Visualizer");
                    });
            });
            visualizer_type_next_state.set(visualizer_type_selection.clone());
            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Frequencies Settings").color(egui::Color32::YELLOW).heading());
            ui.label("Lower Frequency Limit:");
            let upper_freq_limit = audio_visualizer_settings.upper_frequency_limit - audio_visualizer_settings.column_count_power_of_two as f32;
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.lower_frequency_limit, 0.0..=upper_freq_limit));
            
            ui.label("Upper Frequency Limit:");
            let lower_freq_limit = audio_visualizer_settings.lower_frequency_limit + audio_visualizer_settings.column_count_power_of_two as f32;
            let upper_freq_limit_max = (audio_visualizer_settings.sampling_rate / 2) as f32;
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.upper_frequency_limit, lower_freq_limit..=upper_freq_limit_max));
            
            ui.label("Sampling rate:");
            let upper_freq_limit = (audio_visualizer_settings.upper_frequency_limit * 2.0 + 1.0) as u32;
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.sampling_rate, upper_freq_limit..=44100));

            let window_selection = audio_visualizer_settings.window_function;
            ui.horizontal(|ui| {
                ui.label("Select window function:");
                egui::ComboBox::from_id_source("window-selection")
                    .selected_text(format!("{:?}", window_selection))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut audio_visualizer_settings.window_function, WindowFunction::None, "None");
                        ui.selectable_value(&mut audio_visualizer_settings.window_function, WindowFunction::Hann, "Hann Window");
                        ui.selectable_value(&mut audio_visualizer_settings.window_function, WindowFunction::Hamming, "Hamming Window");
                    });
            });
            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Wheel Settings").color(egui::Color32::YELLOW).heading());
            ui.label("Radius");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.radius, 0.0..=1000.0));

            ui.label(RichText::new("Column Count (2 to the power of):").color(egui::Color32::RED));
            ui.horizontal(|ui| {
                ui.add(widgets::Slider::new(&mut audio_visualizer_settings.column_count_power_of_two, 5..=10));
                ui.label(format!("= {}", 2_u32.pow(audio_visualizer_settings.column_count_power_of_two as u32) as usize));
            });

            ui.label(RichText::new("Column Width:").color(egui::Color32::RED));
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.column_width, 1.0..=20.0));

            ui.label("Max Height");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.max_height, 2.0..=1000.0));

            ui.label("Section Count");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.section_count, 1..=10));

            ui.label("Rotation Speed");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.rotation_speed, -0.500..=0.500));

            ui.label("Scale Strength");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.scale_strenght, 0.0..=20000.0));

            ui.label("Scale Threshold");
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.scale_threshold, 0.55..=10.0));

            ui.label("Smoothing Range");
            let half_of_column_count = audio_visualizer_settings.column_count / 2;
            ui.add(widgets::Slider::new(&mut audio_visualizer_settings.smoothing_range, 1..=half_of_column_count));

            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Color Settings").color(egui::Color32::YELLOW).heading());

            ui.label(RichText::new("Normal Color").strong());
            (audio_visualizer_settings.normal_primary_color, 
            audio_visualizer_settings.normal_secondary_color, 
            audio_visualizer_settings.normal_primary_color_hdr_multiplier,
            audio_visualizer_settings.normal_secondary_color_hdr_multiplier,
            audio_visualizer_settings.normal_color_transition_enabled,
            audio_visualizer_settings.normal_color_transition_speed) = update_color_material(
                ui,
                audio_visualizer_settings.normal_primary_color,
                audio_visualizer_settings.normal_secondary_color,
                audio_visualizer_settings.normal_primary_color_hdr_multiplier,
                audio_visualizer_settings.normal_secondary_color_hdr_multiplier,
                audio_visualizer_settings.normal_color_transition_enabled,
                audio_visualizer_settings.normal_color_transition_speed
            );

            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Highlight Color").strong());
            ui.vertical(|ui| {
                (audio_visualizer_settings.highlight_primary_color, 
                audio_visualizer_settings.highlight_secondary_color, 
                audio_visualizer_settings.highlight_primary_color_hdr_multiplier,
                audio_visualizer_settings.highlight_secondary_color_hdr_multiplier,
                audio_visualizer_settings.highlight_color_transition_enabled,
                audio_visualizer_settings.highlight_color_transition_speed) = update_color_material(
                    ui,
                    audio_visualizer_settings.highlight_primary_color,
                    audio_visualizer_settings.highlight_secondary_color,
                    audio_visualizer_settings.highlight_primary_color_hdr_multiplier,
                    audio_visualizer_settings.highlight_secondary_color_hdr_multiplier,
                    audio_visualizer_settings.highlight_color_transition_enabled,
                    audio_visualizer_settings.highlight_color_transition_speed
                );
            });

            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Background Color").strong());
            let mut clear_color_rgba = clear_color.as_rgba_f32();
            ui.color_edit_button_rgba_unmultiplied(&mut clear_color_rgba);
            *clear_color = ClearColor(Color::rgba_from_array(clear_color_rgba));

            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Advanced Settings").color(egui::Color32::YELLOW).heading());

            ui.checkbox(&mut advanced_settings.show_fps, RichText::new("Show FPS").color(Color32::RED));
            ui.checkbox(&mut advanced_settings.vsync, RichText::new("VSync").color(Color32::RED));

            ui.horizontal(|ui| {
                ui.label(RichText::new("Apply changes:").color(egui::Color32::RED));
                if ui.button("Apply").clicked() {
                    advanced_settings_change_event_writer.send(AdvancedSettingsChangeEvent);
                    audio_visualizer_restructure_event_writer.send(AudioVisualizerRestructureEvent);
                }
            });
        });
}

fn update_color_material(
    ui: &mut egui::Ui,
    mut primary_color: Color,
    mut secondary_color: Color,
    mut primary_color_hdr_multiplier: f32,
    mut secondary_color_hdr_multiplier: f32,
    mut transition_enabled: bool,
    mut transition_speed: f32
) -> (Color, Color, f32, f32, bool, f32) {
    ui.horizontal(|ui| {
        ui.label("Primary:");
        let mut primary_color_rgba = primary_color.as_rgba_f32();
        ui.color_edit_button_rgba_unmultiplied(&mut primary_color_rgba);
        primary_color = Color::rgba_from_array(primary_color_rgba);
    });
    ui.label("HDR Multiplier:");
    ui.add(widgets::Slider::new(&mut primary_color_hdr_multiplier, 1.0..=10.0));

    ui.checkbox(&mut transition_enabled, "Enable Transition");

    ui.add_enabled_ui(transition_enabled, |ui|{
        ui.horizontal(|ui| {
            ui.label("Secondary:");
            let mut secondary_color_rgba = secondary_color.as_rgba_f32();
            ui.color_edit_button_rgba_unmultiplied(&mut secondary_color_rgba);
            secondary_color = Color::rgba_from_array(secondary_color_rgba);
        });
        ui.label("HDR Multiplier:");
        ui.add(widgets::Slider::new(&mut secondary_color_hdr_multiplier, 1.0..=10.0));
        ui.label("Transition Speed:");
        let mut was_negative = false;
        if transition_speed < 0.0 {
            transition_speed *= -1.0;
            was_negative = true;
        }
        ui.add(widgets::Slider::new(&mut transition_speed, 0.0001..=0.5));
        if was_negative {
            transition_speed *= -1.0;
        }
    });

    (primary_color, secondary_color, primary_color_hdr_multiplier, secondary_color_hdr_multiplier, transition_enabled, transition_speed)
}