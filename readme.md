# XBST

I wanted a simple utility to sync my music library to something I can have on my original Xbox but I didn't find any easy to use, crossplatform and straight forward utility so I made this.

## Prerequisits

- Windows or Linux (maybe macOS, untested)
- ffmpeg and ffprobe in your PATH

If you are on Windows you can easily install ffmpeg and ffprobe by opening a terminal and doing `winget install ffmpeg`.

If you are on Linux, well, you're probably a nerd and already have it, if not, open a terminal and do `sudo apt install ffmpeg` on debian based distributions.

## Usage

Drag and drop your music folder onto xbst

The input music folder needs the following structure:
```
 📁 Music
 ├📁 Soundtrack 1
 |├ 💾 Music 1
 |├ 💾 Music 2
 |└ 💾 Music 3
 └📁 Soundtrack 2
  └ 💾 Music 1
 ```

Directory/File names must not exceed 31~ characters.

If you encounter issues with song not playing you could try to change the codec to wmav1 or lower the bitrate.

```
Usage: xbst [OPTIONS] [INPUT] [OUTPUT]

Arguments:
  [INPUT]   Input folder of your musics [default: ./music]
  [OUTPUT]  Output folder for the database and converted musics [default: ./output]

Options:
  -b, --bitrate <BITRATE>  Bitrate for the output [default: 128]
  -c, --codec <CODEC>      Codec to use for conversion [default: wmav2] [possible values: wmav1, wmav2]
  -h, --help               Print help
  -V, --version            Print version
```

## Known issues

- The progress bar on soundtrack other than the first one doesn't progress.
- Some files, once converted, are quieter than usual?
- When using wmav1, some audio files might sounds absolute ass.
- Untested with a large library, probably has issues?
- Code is poo poo :(