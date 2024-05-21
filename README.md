# Customizable Bevy Audio Visualizer
**This is an audio visualizer written in Rust using Bevy, CPAL, audio-visualizer, and a few more libraries (all dependencies are in Cargo.toml).**
**It is capable of visualizing audio frequencies and audio spectrum (kind of) from the audio playing from your default output device.**  
It features many options to customize it to your liking (as you can see in the picture below).

- **Visualizer Type** - choose either Frequency Visualizer or Spectrum Visualizer (visualization of the spectrum is kind of useless)

## Frequencies Settings
- **Lower Frequency Limit** - bottom limit of the range of captured frequencies
- **Upper Frequency Limit** - top limit of the range of captured frequencies
- **Sampling Rate** - affects the frequency resolution: sampling_rate / (4096) e.g. 44100 / 4096 = 10.766 Hz, more samples => better accuracy/frequency resolution (sampling_rate must be > 2 * upper_frequency_limit)
- **Window Function** - apply either none, Hann, or Hamming window function to audio samples

## Wheel Settings
- **Radius** - radius of the wheel (when it's not affected by scaling)
- **Column Count** - how many columns the wheel is going to be made of (requires to be applied by clicking the button, affects performance the most!)
- **Column Width** - width of all columns (requires to be applied by clicking the button)
- **Max Height** - the maximum height a column can reach
- **Section Count** - divides the wheel into multiple sections
- **Rotation Speed** - speed and direction of the wheel's rotation
- **Scale Strength** - how much the wheel scales its radius on "beats"
- **Scale Threshold** - smooths the transitions between scaling
- **Smoothing Range** - range in which columns affect the height of neighboring columns

## Color Settings
- **Normal Color** - color of all columns that are not highlighted, slider to enable HDR colors. If transition is enabled, the color of columns will transition forward and backward between primary and secondary color; speed can be adjusted by the slider
- **Highlighted Color** - color of all columns that are highlighted (a column gets highlighted when its value is > 2 * average_column_value), slider to enable HDR colors. If transition is enabled, the color of columns will transition forward and backward between primary and secondary color; speed can be adjusted by the slider
- **Background Color** - color of the background, the darker the color, the better the visibility of HDR colors

## Advanced Settings
- **Show FPS** - shows current FPS of the application (requires to be applied by clicking the button, the FPS of the audio visualizer is capped at 32 FPS)
- ** VSync** - synchronizes the FPS with the refresh rate of the monitor (requires to be applied by clicking the button)
  
![avs](https://github.com/Eightzi4/customizable-bevy-audio-visualizer/assets/111708236/a693f107-d6db-4931-94a2-a2b23ce13f62)
