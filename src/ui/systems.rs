use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui::{self, widgets, RichText}};
use crate::{visualizer::components::{AudioVisualizerRestructureEvent, AudioVisualizerSettings, VisualilzerType}, AdvancedSettings, AdvancedSettingsChangeEvent};

pub fn settings_ui(
    mut contexts: EguiContexts,
    mut audio_visualizer_restructure_event_writer: EventWriter<AudioVisualizerRestructureEvent>,
    mut advanced_settings_change_event_writer: EventWriter<AdvancedSettingsChangeEvent>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut advanced_settings: ResMut<AdvancedSettings>,
    mut clear_color: ResMut<ClearColor>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visualizer_type_state: Res<State<VisualilzerType>>,
    mut visualizer_type_next_state: ResMut<NextState<VisualilzerType>>
) {
    egui::Window::new("Audio Visualizer Settings")
        .resizable(true)
        .vscroll(true)
        .max_width(265.0)
        .show(contexts.ctx_mut(), |ui| {

            let mut visualizer_type_selection = visualizer_type_state.get();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Select visualizer type:").color(egui::Color32::YELLOW));
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", visualizer_type_selection))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut visualizer_type_selection, &VisualilzerType::FrequencyVisualizer, "Frequency Visualizer");
                        ui.selectable_value(&mut visualizer_type_selection, &VisualilzerType::SpectrumVisualizer, "Spectrum Visualizer");
                    });
            });

            visualizer_type_next_state.set(visualizer_type_selection.clone());

            ui.add(widgets::Separator::default());

            ui.label("Column Count (2 to the power of):");
            ui.horizontal(|ui| {
                ui.add(widgets::Slider::new(
                    &mut audio_visualizer_settings.column_count_power_of_two,
                    5..=10,
                ));
                ui.label(format!("= {}", 2_u32.pow(audio_visualizer_settings.column_count_power_of_two as u32) as usize));
            });

            ui.label("Column Width");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.column_width,
                1.0..=20.0,
            ));

            ui.horizontal(|ui| {
                ui.label(RichText::new("Apply changes made above:").color(egui::Color32::RED));
                if ui.button("Apply").clicked() {
                    audio_visualizer_restructure_event_writer.send(AudioVisualizerRestructureEvent);
                }
            });

            ui.add(widgets::Separator::default());

            ui.label("Max Height");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.max_height,
                2.0..=2000.0,
            ));

            ui.label("Section Count");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.section_count,
                1..=10,
            ));

            ui.label("Radius");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.radius,
                0.0..=1000.0,
            ));

            ui.label("Rotation Speed");
            ui.add(
                widgets::Slider::new(
                    &mut audio_visualizer_settings.rotation_speed,
                    -0.500..=0.500
                )
            );

            ui.label("Smoothing Range");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.smoothing_range,
                1..=20,
            ));

            ui.label("Scale Strength");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.scale_strenght,
                0.0..=20000.0,
            ));

            ui.label("Scale Treshold");
            ui.add(widgets::Slider::new(
                &mut audio_visualizer_settings.scale_treshold,
                0.55..=10.0,
            ));

            ui.add(widgets::Separator::default());

            ui.label("Normal Color");
            let mut color_material_rgba = materials.get(audio_visualizer_settings.normal_color_material_handle.clone().unwrap().id()).unwrap().color.as_rgba_f32();
            color_material_rgba.iter_mut().for_each(|c| *c /= audio_visualizer_settings.normal_color_hdr_multiplier);
            color_material_rgba[3] *= audio_visualizer_settings.normal_color_hdr_multiplier;
            ui.color_edit_button_rgba_unmultiplied(&mut color_material_rgba);

            ui.label("HDR Color Multiplier");
            ui.add(
                widgets::Slider::new(
                    &mut audio_visualizer_settings.normal_color_hdr_multiplier,
                    1.0..=10.0
                )
            );
            materials.get_mut(audio_visualizer_settings.normal_color_material_handle.clone().unwrap().id()).unwrap().color = Color::rgba(color_material_rgba[0] * audio_visualizer_settings.normal_color_hdr_multiplier, color_material_rgba[1] * audio_visualizer_settings.normal_color_hdr_multiplier, color_material_rgba[2] * audio_visualizer_settings.normal_color_hdr_multiplier, color_material_rgba[3]);
            
            ui.add(widgets::Separator::default());

            ui.label("Highlight Color");
            color_material_rgba = materials.get(audio_visualizer_settings.highlight_color_material_handle.clone().unwrap().id()).unwrap().color.as_rgba_f32();
            color_material_rgba.iter_mut().for_each(|c| *c /= audio_visualizer_settings.highlight_color_hdr_multiplier);
            color_material_rgba[3] *= audio_visualizer_settings.highlight_color_hdr_multiplier;
            ui.color_edit_button_rgba_unmultiplied(&mut color_material_rgba);

            ui.label("HDR Color Multiplier");
            ui.add(
                widgets::Slider::new(
                    &mut audio_visualizer_settings.highlight_color_hdr_multiplier,
                    1.0..=10.0
                )
            );
            materials.get_mut(audio_visualizer_settings.highlight_color_material_handle.clone().unwrap().id()).unwrap().color = Color::rgba(color_material_rgba[0] * audio_visualizer_settings.highlight_color_hdr_multiplier, color_material_rgba[1] * audio_visualizer_settings.highlight_color_hdr_multiplier, color_material_rgba[2] * audio_visualizer_settings.highlight_color_hdr_multiplier, color_material_rgba[3]);
        
            ui.add(widgets::Separator::default());

            ui.label(RichText::new("Advanced Settings").heading());

            ui.checkbox(&mut advanced_settings.show_fps, "Show FPS");

            ui.checkbox(&mut advanced_settings.vsync, "VSync");

            ui.horizontal(|ui| {
                ui.label(RichText::new("Apply changes made above:").color(egui::Color32::RED));
                if ui.button("Apply").clicked() {
                    advanced_settings_change_event_writer.send(AdvancedSettingsChangeEvent);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Background Color");
                let mut clear_color_rgba = clear_color.as_rgba_f32();
                ui.color_edit_button_rgba_unmultiplied(&mut clear_color_rgba);
                *clear_color = ClearColor(Color::rgba(clear_color_rgba[0], clear_color_rgba[1], clear_color_rgba[2], clear_color_rgba[3]));
            });
        });
}