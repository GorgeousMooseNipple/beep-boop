## Beep-boop
Toy synthesizer written in Rust.  
I was just curious how to make this thing produce sounds. Also this was my first non "Hello, world!"-type project in Rust.  
It works on Ubuntu 20 and its the one and only platform I truly tested it on.

## Content
* [Dependencies](#dependencies)
* [Interface](#interface)
* [Controls](#controls)
* [Demo](#demo)
* [TODO](#todo)  

## Dependencies
* Beep-boop uses [Portaudio-rs][portaudio-rs] to produce sounds
* [Druid][druid] for that magnificent look
* [Rand][rand] to generate random numbers for phase purposes
* and [Num-traits][num-traits] to define sample formats

## Interface
![Beep-boop UI](../media/images/beep-boop-default-ui.png?raw=true)  

Beep-boop has two identical **oscillators** with five waveforms each:
* Sine
* Triangle
* Saw
* Square
* Pulse with 25% width

Both oscillators have volume slider, transpose control which changes pitch in semitones and tune control to change pitch in cents.  
There are up to 7 unison voices. If current unison count for oscillator is more than 1, tune control starts to act as a spread control, affecting fine tuning of each unison differently relative to base pitch.  
So if you have 5 unisons with tune control at 10 and play middle C, 1 voice is going to be right on the middle C frequency (around 261.63 Hz), 2 unisons 5 cents apart from that (one 5 cents up and the other 5 cents down) and other 2 unisons 2.5 cents apart from middle C.  

For each oscillator you can pick one of the two **ADSR-envelopes**.  
Envelopes have log scale sliders for standard attack, decay, sustain and release controls. Values for sustain are in 0.0-1.0 range and for the other parameters it's from 1 ms to 3000 ms. With _Ctrl+click_ those values can be reset to default.

Of course there is general output volume slider on top-right. And that's it.

## Controls
It can be played only with keyboard and uses piano-like layout where 'z' key is binded to C piano key, 's' key is C#, 'x' is D and so on ending on 'm' key which represents B. It's range is only one octave, but you can switch octaves up and down using left and right arrow keys.

Application can be closed by pressing Escape.

## Demo
Very unprofessional demo recorded on a microphone directly from my speakers. Sorry about quality.  
https://user-images.githubusercontent.com/36864010/163676890-1f525295-d085-4942-bb2b-0abf3f6bddbd.mp4


## TODO:

* Probably rework all the internals responsible for producing sounds and optimise it
* Make limited pool of voices available
* Switch between mono and poly modes
* Some form of panning, maybe stereo spread for unisons
* Would be great to implement basic LP/HP filters


[portaudio-rs]: https://github.com/mvdnes/portaudio-rs
[druid]: https://github.com/linebender/druid
[rand]: https://github.com/rust-random/rand
[num-traits]: https://github.com/rust-num/num-traits
