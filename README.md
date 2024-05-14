# Customizable Bevy Audio Visualizer
**This in an audio visualizer written in Rust capable of visualizing audio frequencies and audio spectrum (kinda).**  
It features many options to customize it to your liking (as you can see in picture below).
  
- **Column Count** - how many columns is the wheel going to be made of, each column represents the average value of: 1530.5 frequencies / colum count (requires to be applied by clicking the button)
- **Column Width** - width for all columns (requires to be applied by clicking the button)
- **Max Height** - the maximum height a column can reach
- **Section Count** - divides the wheel into multiple sections
- **Radius** - radius of the wheel (when it's not affected by scaling)
- **Rotation Speed** - speed and direction of the wheel's rotation
- **Smoothing Range** - range in which columns affect the height of neighboring columns
- **Scale Strength** - how much does the wheel scale its radius on "beats"
- **Scale Threshold** - smooths the transitions between scaling
- **Color settings** - colors of columns/highlighted column/background (with HDR option)
- **FPS settings** - VSync/Show FPS (requires to be applied by clicking the button)
![avs](https://github.com/Eightzi4/customizable-bevy-audio-visualizer/assets/111708236/a693f107-d6db-4931-94a2-a2b23ce13f62)
