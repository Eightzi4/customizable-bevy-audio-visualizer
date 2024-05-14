use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use crate::audio_data::{components::AudioData, SPECTRUM_DATA_LENGTH};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, scaling::scale_to_zero_to_one};
use ringbuffer::RingBuffer;
use crate::visualizer::*;
use super::components::*;


const FIXED_ARRAY_SIZE_1: usize = 1024; // I've decided to use large array instead of vector, may not be the right choice tho.
 
pub fn setup_audio_visualizer(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let window = window.get_single().unwrap();
    audio_visualizer_settings.column_width = window.width() / audio_visualizer_settings.column_count as f32;

    audio_visualizer_settings.highlight_color_material_handle = Some(materials.add(Color::rgb(1.0, 0.0, 0.0)));

    audio_visualizer_settings.normal_color_material_handle = Some(materials.add(Color::rgb(1.0, 1.0, 1.0)));

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
                AudioVisualizerCollumn
            ));
        }
    });
}

pub fn restructure_audio_visualizer(
    mut commands: Commands,
    mut audio_visulizer_container_query: Query<Entity, With<AudioVisualizerContainer>>,
    mut audio_visualizer_settings: ResMut<AudioVisualizerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let current_audio_visualizer = audio_visulizer_container_query.get_single_mut().unwrap();

    audio_visualizer_settings.column_count = 2_u32.pow(audio_visualizer_settings.column_count_power_of_two as u32) as usize;
    audio_visualizer_settings.angle_increment = 2.0 * PI / audio_visualizer_settings.column_count as f32;

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
                AudioVisualizerCollumn
            ));
        }
    });

    commands.entity(current_audio_visualizer).despawn_recursive();
}

pub fn tick_audio_visualizer_update_timer(
    time: Res<Time>,
    mut audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    audio_visualizer_update_timer.timer.tick(time.delta());
}

pub fn visualize_audio_spectrum(
    audio_data: NonSend<AudioData>,
    mut column_query: Query<(&mut Transform, &mut Handle<ColorMaterial>), With<AudioVisualizerCollumn>>,
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
            let mut combined_spectrums: [f32; FIXED_ARRAY_SIZE_1] = [0.0; FIXED_ARRAY_SIZE_1];
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
    mut column_query: Query<(&mut Transform, &mut Handle<ColorMaterial>), With<AudioVisualizerCollumn>>,
    audio_visualizer_settings: Res<AudioVisualizerSettings>,
    audio_visualizer_update_timer: ResMut<AudioVisualizerUpdateTimer>
) {
    if audio_visualizer_update_timer.timer.just_finished() {
        let spectrum_data = audio_data.latest_audio_data.lock().unwrap();
        
        if spectrum_data.len() == audio_visualizer_settings.spectrum_data_length {

            let spectrum_data_clone: [f32; SPECTRUM_DATA_LENGTH] = spectrum_data.iter().copied().collect::<Vec<f32>>().try_into().unwrap();
            drop(spectrum_data);

            let frequencies = samples_fft_to_spectrum(
                &spectrum_data_clone,
                audio_visualizer_settings.spectrum_data_length as u32 / 2,
                FrequencyLimit::Range(20.0, 1555.5),
                Some(&scale_to_zero_to_one),
            ).unwrap();

            audio_data.latest_average_frequency_value = frequencies.average().val();

            let mut combined_frequencies: [f32; FIXED_ARRAY_SIZE_1] = [0.0; FIXED_ARRAY_SIZE_1];
            let sum_range = frequencies.data().len() / audio_visualizer_settings.column_count;
            for i in 0..audio_visualizer_settings.column_count / audio_visualizer_settings.section_count {
                for j in i * sum_range..(i + 1) * sum_range {
                    combined_frequencies[i] += frequencies.data().get(j).unwrap().1.val();
                }
            }

            for (i, (mut transform, mut material)) in column_query.iter_mut().enumerate() {
                let clamped_index = i % (audio_visualizer_settings.column_count / audio_visualizer_settings.section_count);
            
                let new_height;
                if i < audio_visualizer_settings.column_count / audio_visualizer_settings.section_count {
                    let mut smooth_frequency = 0.0;
                    let mut index = (clamped_index as i32 - (audio_visualizer_settings.smoothing_range / 2) as i32 + audio_visualizer_settings.column_count as i32 / audio_visualizer_settings.section_count as i32) % (audio_visualizer_settings.column_count / audio_visualizer_settings.section_count) as i32;
                    for _ in 0..audio_visualizer_settings.smoothing_range {
                        smooth_frequency += combined_frequencies[index as usize];
                        index = (index + 1) % (audio_visualizer_settings.column_count / audio_visualizer_settings.section_count) as i32;
                    }
                    combined_frequencies[clamped_index] = smooth_frequency / audio_visualizer_settings.smoothing_range as f32;
                    new_height = combined_frequencies[clamped_index] / sum_range as f32 * audio_visualizer_settings.max_height / frequencies.max().1.val();
                } else {
                    let opposite_index = (i - audio_visualizer_settings.column_count / audio_visualizer_settings.section_count) % (audio_visualizer_settings.column_count / audio_visualizer_settings.section_count);
                    new_height = combined_frequencies[opposite_index] / sum_range as f32 * audio_visualizer_settings.max_height / frequencies.max().1.val();
                }
            
                if new_height.is_nan() {
                    transform.scale.y = 1.0;
                } else {
                    transform.scale.y = new_height.clamp(1.0, audio_visualizer_settings.max_height);
                }
            
                if combined_frequencies[clamped_index] / sum_range as f32 > frequencies.average().val() * 2.0 {
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

pub fn update_audio_visualizer_scale(
    audio_data: NonSend<AudioData>,
    mut audio_visulizer_container_query: Query<&mut Transform, With<AudioVisualizerCollumn>>,
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

            transform.translation.x = (x - current_x) / audio_visualizer_settings.scale_treshold + current_x;
            transform.translation.y = (y - current_y) / audio_visualizer_settings.scale_treshold + current_y;
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