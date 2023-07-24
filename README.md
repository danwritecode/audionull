# Audionull

## This project is in progress, I will add more parts as I slowly build up to the final product.

### Part 1 - Learn Audio
My goal for this part is to just work through and gain an understanding of how audio works. From Fourier Transform to simply reading an audio signal from the serial port of an arduino.


#### Visualize Audio
This part of the project (source code in "visualize_audio" folder) was to figure out how to read the serial input from the arduino (or any microcontroller) and how do do things with it. For now I decided that I just wanted to visualize this data. So I used macroquad, a game engine in Rust, to quickly and easily visualize the output data that I was receiving. 

Here is the audio visualizer in action:
![audio](https://github.com/danwritecode/audionull/blob/master/visualize_audio/arduino_audio.gif)


