use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use crate::audio_data::{components::AudioData, SPECTRUM_DATA_LENGTH};
use spectrum_analyzer::{samples_fft_to_spectrum, scaling::scale_to_zero_to_one, windows::{hamming_window, hann_window}, FrequencyLimit};
use ringbuffer::RingBuffer;
use crate::visualizer::*;
use super::components::*;


const FIXED_ARRAY_SIZE: usize = 1024;
 
pub fn setup_audio_visualizer(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let window = window.get_single().unwrap();
    audio_visualizer_settings.column_width = window.width() / audio_visualizer_settings.column_count as f32;

    audio_visualizer_settings.normal_color_material_handle = Some(materials.add(audio_visualizer_settings.normal_primary_color));

    audio_visualizer_settings.highlight_color_material_handle = Some(materials.add(audio_visualizer_settings.highlight_primary_color));

    build_audio_visualizer(&mut commands, audio_visualizer_settings, meshes, materials);
}

pub fn restructure_audio_visualizer(
    mut commands: Commands,
    mut audio_visulizer_container_query: Query<Entity, With<AudioVisualizerContainer>>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>
) {
    let current_audio_visualizer = audio_visulizer_container_query.get_single_mut().unwrap();

    audio_visualizer_settings.column_count = 2_u32.pow(audio_visualizer_settings.column_count_power_of_two as u32) as usize;
    audio_visualizer_settings.angle_increment = 2.0 * PI / audio_visualizer_settings.column_count as f32;

    build_audio_visualizer(&mut commands, audio_visualizer_settings, meshes, materials);
    commands.entity(current_audio_visualizer).despawn_recursive();
}

fn build_audio_visualizer(
    commands: &mut Commands,
    audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        AudioVisualizerContainer
    )
    ).with_children(|parent| {
        for i in 0..audio_visualizer_settings.column_count {
            let angle = i as f32 * 2.0 * PI / audio_visualizer_settings.column_count as f32;
            let x = audio_visualizer_settings.radius * angle.cos();
            let y = audio_visualizer_settings.radius * angle.sin();
    
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Rectangle::new(audio_visualizer_settings.column_width, 1.0)).into(),
                    material: audio_visualizer_settings.normal_color_material_handle.clone().unwrap(),
                    transform: Transform::from_xyz(x, y, 0.0).with_rotation(Quat::from_rotation_z(angle + PI / 2.0)),
                    ..default()
                },
                AudioVisualizerColumn
            ));
        }
    });
}

pub fn tick_audio_visualizer_update_timer(
    time: Res<Time>,
    mut audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    audio_visualizer_update_timer.timer.tick(time.delta());
}

pub fn visualize_audio_spectrum(
    audio_data: NonSend<AudioData>,
    mut column_query: Query<(&mut Transform, &mut Handle<ColorMaterial>), With<AudioVisualizerColumn>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    audio_visualizer_settings: Res<AudioVisualizerSettings>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == audio_visualizer_settings.spectrum_data_length {
            let window = window_query.get_single().unwrap();

            let max_height = window.height();

            let mut highest_spectrum_value = 0.0;
            let mut combined_spectrums: [f32; FIXED_ARRAY_SIZE] = [0.0; FIXED_ARRAY_SIZE];
            let sum_range = spectrum_data.len() / audio_visualizer_settings.column_count;
            for i in 0..audio_visualizer_settings.column_count {
                for j in i * sum_range..(i + 1) * sum_range {
                    combined_spectrums[i] += spectrum_data[j];
                    if spectrum_data[i] > highest_spectrum_value {
                        highest_spectrum_value = spectrum_data[i];
                    }
                }
            }

            for (i, (mut transform, mut material)) in column_query.iter_mut().enumerate() {
                let new_height;
                let mut smooth_spectrum = 0.0;
                let mut index: i32 = (i as i32 - (audio_visualizer_settings.smoothing_range / 2) as i32 + audio_visualizer_settings.column_count as i32) % (audio_visualizer_settings.column_count) as i32;
                for _ in 0..audio_visualizer_settings.smoothing_range {
                    smooth_spectrum += combined_spectrums[index as usize];
                    index = (index + 1) % audio_visualizer_settings.column_count as i32;
                }
                combined_spectrums[i] = smooth_spectrum / audio_visualizer_settings.smoothing_range as f32;
                new_height = combined_spectrums[i] / sum_range as f32 * max_height / highest_spectrum_value;
                
                if new_height.is_nan() {
                    transform.scale.y = 1.0;
                } else {
                    transform.scale.y = new_height.clamp(1.0, audio_visualizer_settings.max_height);
                }

                if combined_spectrums[i] / sum_range as f32 > 0.0 {
                    *material = audio_visualizer_settings.highlight_color_material_handle.clone().unwrap()
                } else {
                    *material = audio_visualizer_settings.normal_color_material_handle.clone().unwrap()
                }
            }
        } else {
            println!("Not enough data to visualize!");
        }
    }
}

pub fn visualize_audio_frequency(
    mut audio_data: NonSendMut<AudioData>,
    mut column_query: Query<(&mut Transform, &mut Handle<ColorMaterial>), With<AudioVisualizerColumn>>,
    audio_visualizer_settings: Res<AudioVisualizerSettings>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == audio_visualizer_settings.spectrum_data_length {
            let mut spectrum_data_clone: [f32; SPECTRUM_DATA_LENGTH] = spectrum_data.iter().copied().collect::<Vec<f32>>().try_into().unwrap();
            drop(spectrum_data);

            match audio_visualizer_settings.window_function {
                WindowFunction::Hann => {
                    spectrum_data_clone = hann_window(&spectrum_data_clone).try_into().unwrap();
                },
                WindowFunction::Hamming => {
                    spectrum_data_clone = hamming_window(&spectrum_data_clone).try_into().unwrap();
                },
                _ => {}
            }

            let frequencies = samples_fft_to_spectrum(
                &spectrum_data_clone,
                audio_visualizer_settings.sampling_rate,
                FrequencyLimit::Range(audio_visualizer_settings.lower_frequency_limit, audio_visualizer_settings.upper_frequency_limit),
                Some(&scale_to_zero_to_one),
            ).unwrap();

            audio_data.latest_average_frequency_value = frequencies.average().val();

            let frequencies_data = frequencies.data();
            let frequencies_len = frequencies_data.len();
            let column_count = audio_visualizer_settings.column_count;
            let mut combined_frequencies: [f32; FIXED_ARRAY_SIZE] = [0.0; FIXED_ARRAY_SIZE];

            if frequencies_len >= column_count {
                let sum_range = frequencies_len / column_count;
                for (i, freq) in combined_frequencies.iter_mut().enumerate().take(column_count) {
                    let start_index = i * sum_range;
                    let end_index = if i == column_count - 1 {
                        frequencies_len
                    } else {
                        (i + 1) * sum_range
                    };
                    *freq = frequencies_data[start_index..end_index].iter().map(|(_, val)| val.val()).sum::<f32>() / sum_range as f32;
                }
            } else {
                let columns_per_frequency = column_count / frequencies_len;
                let extra_columns = column_count % frequencies_len;
                let mut current_column = 0;

                for (i, (_, freq)) in frequencies_data.iter().enumerate() {
                    let count = if i < extra_columns { columns_per_frequency + 1 } else { columns_per_frequency };
                    for _ in 0..count {
                        combined_frequencies[current_column] = freq.val();
                        current_column += 1;
                    }
                }
            }

            let section_count = audio_visualizer_settings.section_count;
            let max_height = audio_visualizer_settings.max_height;
            let smoothing_range = audio_visualizer_settings.smoothing_range as i32;
            let half_smoothing_range = smoothing_range / 2;
            let highlight_color = audio_visualizer_settings.highlight_color_material_handle.clone().unwrap();
            let normal_color = audio_visualizer_settings.normal_color_material_handle.clone().unwrap();
            let max_frequency_val = frequencies.max().1.val();
            let highlighted_frequency_threshold = frequencies.average().val() * 2.0;

            for (i, (mut transform, mut material)) in column_query.iter_mut().enumerate() {
                let clamped_index = i % (column_count / section_count);

                let new_height = if i < column_count / section_count {
                    let mut smooth_frequency = 0.0;
                    let mut index = (clamped_index as i32 - half_smoothing_range + column_count as i32 / section_count as i32) % (column_count / section_count) as i32;
                    for _ in 0..smoothing_range {
                        smooth_frequency += combined_frequencies[index as usize];
                        index = (index + 1) % (column_count / section_count) as i32;
                    }
                    combined_frequencies[clamped_index] = smooth_frequency / smoothing_range as f32;
                    combined_frequencies[clamped_index] * max_height / max_frequency_val
                } else {
                    let opposite_index = (i - column_count / section_count) % (column_count / section_count);
                    combined_frequencies[opposite_index] * max_height / max_frequency_val
                };

                transform.scale.y = if new_height.is_nan() { 1.0 } else { new_height.clamp(1.0, max_height) };

                *material = if combined_frequencies[clamped_index] > highlighted_frequency_threshold {
                    highlight_color.clone()
                } else {
                    normal_color.clone()
                };
            }
        } else {
            println!("Not enough data to visualize!");
        }
    }
}

pub fn update_audio_visualizer_scale(
    audio_data: NonSend<AudioData>,
    mut audio_visulizer_container_query: Query<&mut Transform, With<AudioVisualizerColumn>>,
    audio_visualizer_settings: Res<AudioVisualizerSettings>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let radius_scaler = audio_data.latest_average_frequency_value * audio_visualizer_settings.scale_strenght;
        for (i, mut transform) in audio_visulizer_container_query.iter_mut().enumerate() {
            let current_x = transform.translation.x;
            let current_y = transform.translation.y;
            let angle = i as f32 * audio_visualizer_settings.angle_increment;
            let x = (audio_visualizer_settings.radius + radius_scaler) * angle.cos();
            let y = (audio_visualizer_settings.radius + radius_scaler) * angle.sin();

            transform.translation.x = (x - current_x) / audio_visualizer_settings.scale_threshold + current_x;
            transform.translation.y = (y - current_y) / audio_visualizer_settings.scale_threshold + current_y;
        }
    }
}

pub fn update_audio_visualizer_rotation(
    mut audio_visulizer_container_query: Query<&mut Transform, With<AudioVisualizerContainer>>,
    audio_visulizer_settings: Res<AudioVisualizerSettings>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() && audio_visulizer_settings.rotation_speed != 0.0 {
        let mut audio_visualizer_container = audio_visulizer_container_query.get_single_mut().unwrap();
        audio_visualizer_container.rotation *= Quat::from_rotation_z(-audio_visulizer_settings.rotation_speed);
    }
}

pub fn center_audio_visualizer(
    mut audio_visulizer_container_query: Query<&mut Transform, With<AudioVisualizerContainer>>
) {
    let mut audio_visualizer_container = audio_visulizer_container_query.get_single_mut().unwrap();
    audio_visualizer_container.translation = Vec3::new(0.0, 0.0, 0.0);
}

pub fn update_color_transition(
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let normal_material = materials.get_mut(audio_visualizer_settings.normal_color_material_handle.clone().unwrap()).unwrap();
        if audio_visualizer_settings.normal_color_transition_enabled {
            if audio_visualizer_settings.normal_color_transition_progress > 1.0 && audio_visualizer_settings.normal_color_transition_speed > 0.0 || audio_visualizer_settings.normal_color_transition_progress < 0.0 && audio_visualizer_settings.normal_color_transition_speed < 0.0 {
                audio_visualizer_settings.normal_color_transition_speed *= -1.0;
            }
            audio_visualizer_settings.normal_color_transition_progress += audio_visualizer_settings.normal_color_transition_speed;
            let updated_color = lerp_color(audio_visualizer_settings.normal_primary_color, audio_visualizer_settings.normal_secondary_color, audio_visualizer_settings.normal_color_transition_progress);
            let hdr_multiplier = audio_visualizer_settings.normal_primary_color_hdr_multiplier + (audio_visualizer_settings.normal_secondary_color_hdr_multiplier - audio_visualizer_settings.normal_primary_color_hdr_multiplier) * audio_visualizer_settings.normal_color_transition_progress;
            let mut updated_color_rgba = updated_color.as_rgba_f32();
            updated_color_rgba.iter_mut().take(3).for_each(|c| *c *= hdr_multiplier);
            normal_material.color = Color::rgba_from_array(updated_color_rgba);
        } else {
            let mut updated_color_rgba = audio_visualizer_settings.normal_primary_color.as_rgba_f32();
            updated_color_rgba.iter_mut().take(3).for_each(|c| *c *= audio_visualizer_settings.normal_primary_color_hdr_multiplier);
            normal_material.color = Color::rgba_from_array(updated_color_rgba);
        }

        let highlight_material = materials.get_mut(audio_visualizer_settings.highlight_color_material_handle.clone().unwrap()).unwrap();
        if audio_visualizer_settings.highlight_color_transition_enabled {
            if audio_visualizer_settings.highlight_color_transition_progress > 1.0 && audio_visualizer_settings.highlight_color_transition_speed > 0.0 || audio_visualizer_settings.highlight_color_transition_progress < 0.0 && audio_visualizer_settings.highlight_color_transition_speed < 0.0 {
                audio_visualizer_settings.highlight_color_transition_speed *= -1.0;
            }
            audio_visualizer_settings.highlight_color_transition_progress += audio_visualizer_settings.highlight_color_transition_speed;
            let updated_color = lerp_color(audio_visualizer_settings.highlight_primary_color, audio_visualizer_settings.highlight_secondary_color, audio_visualizer_settings.highlight_color_transition_progress);
            let hdr_multiplier = audio_visualizer_settings.highlight_primary_color_hdr_multiplier + (audio_visualizer_settings.highlight_secondary_color_hdr_multiplier - audio_visualizer_settings.highlight_primary_color_hdr_multiplier) * audio_visualizer_settings.highlight_color_transition_progress;
            let mut updated_color_rgba = updated_color.as_rgba_f32();
            updated_color_rgba.iter_mut().take(3).for_each(|c| *c *= hdr_multiplier);
            highlight_material.color = Color::rgba_from_array(updated_color_rgba);
        } else {
            let mut updated_color_rgba = audio_visualizer_settings.highlight_primary_color.as_rgba_f32();
            updated_color_rgba.iter_mut().take(3).for_each(|c| *c *= audio_visualizer_settings.highlight_primary_color_hdr_multiplier);
            highlight_material.color = Color::rgba_from_array(updated_color_rgba);
        }
    }
}

fn lerp_color(
    color1: Color, 
    color2: Color, 
    t: f32
) -> Color {
    let mut result = [0.0; 4];
    result[0] = color1.r() + (color2.r() - color1.r()) * t;
    result[1] = color1.g() + (color2.g() - color1.g()) * t;
    result[2] = color1.b() + (color2.b() - color1.b()) * t;
    result[3] = color1.a() + (color2.a() - color1.a()) * t;
    Color::rgba(result[0], result[1], result[2], result[3])
}