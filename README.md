# See Slug
*Web server to stream or download videos*

- [Install See Slug](#install) on your server

- Stream or download videos in the web interface


## Web Interface

### Browse Page

![](res/screenshots/videos.webp "Video selection page")

Keyboard shortcuts:
* Arrows: Move up, down, left, and right
* Space/Enter: Click on the video/collection link
* Letter: Go to the first video starting with that character

---

### Watch Page

![](res/screenshots/video.webp "Video play page")

Keyboard shortcuts:
* C: Captions
* F: Fullscreen
* P: Picture in Picture
* Space: Play/pause
* Left/Right: Go back/forward 5 seconds


## Build

1. Install [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) and [rustup](https://rustup.rs)

2. Clone and build
	```sh
	git clone https://github.com/RobbyCBennett/SeeSlug.git seeslug
	cd seeslug
	cargo build --release
	```


## Install

1. Get the executable
	- [Download the server executable from GitHub](https://github.com/RobbyCBennett/SeeSlug/releases/latest) or
	- [Build the server executable](#build)

2. Gather your movies and shows into one place

3. Configure your system to run the executable as a startup service



## Naming and Organization
- All videos, posters, subtitles, and subfolders must be under 1 folder
- Videos are sorted alphabetically, so consider putting them in collections and prepending a number
- To show a poster, give it the basename of the video or subfolder like `VIDEO.png`
- To get a subtitle, give it the basename of the video like `VIDEO.vtt` where English is assumed
- Specify the subtitle language with `VIDEO.LANG.vtt` (to add support for other languages, edit `src/languages.rs`)
	- `ar`: Arabic
	- `bn`: Bengali
	- `en`: English
	- `es`: Spanish
	- `fr`: French
	- `hi`: Hindi
	- `pt`: Portuguese
	- `ru`: Russian
	- `ur`: Urdu
	- `zh`: Chinese
- Subtitles can be enabled by default with `VIDEO`.default.vtt or `VIDEO`.default.`LANG`.vtt
- Example file structure:
	- Inception.mp4
	- Inception.png
	- Inception.vtt
	- Indiana Jones.jpg
	- Indiana Jones
		- 1: Raiders of the Lost Ark.default.vtt
		- 1: Raiders of the Lost Ark.mp4
		- 1: Raiders of the Lost Ark.webp
		- 2: Indiana Jones and the Temple of Doom.en.vtt
		- 2: Indiana Jones and the Temple of Doom.mp4
		- 2: Indiana Jones and the Temple of Doom.png
		- 3: Indiana Jones and the Last Crusade.default.en.vtt
		- 3: Indiana Jones and the Last Crusade.es.vtt
		- 3: Indiana Jones and the Last Crusade.mp4
		- 3: Indiana Jones and the Last Crusade.png


## Optional Command Line Arguments

Configuration:
- `--folder`: Folder which contains the posters, subtitles, and videos *(string)*
- `--port`: TCP port to listen to *(integer from 0 to 65535)*

Other:
- `--help` or `-h`: Display the help text
- `--version` or `-v`: Display the version text


## Supported Formats

### Video File Extensions
- .mp4

### Poster File Extensions
- .jpeg
- .jpg
- .png
- .webp

### Subtitle File Extensions
- .vtt

### [Audio and Video in Chromium](https://www.chromium.org/audio-video)

### [Audio and Video in Firefox](https://support.mozilla.org/en-US/kb/html5-audio-and-video-firefox)

### Recommended Settings in [HandBrake](https://github.com/HandBrake/HandBrake/releases)

- Summary:
  - Format: MP4, Web Optimized
- Dimensions:
  - Orientation and Cropping:
    - Cropping: Automatic
- Video:
  - Video Encoder:
    - AV1 (NVENC), AV1 (VNC), or AV1 (QuickSync) if your GPU supports it or...
    - H.265 (NVENC), H.265 (VNC), or H.265 (QuickSync) if your web browser supports it or...
    - H.264 (NVENC), H.264 (VNC), or H.264 (QuickSync) if your GPU supports it or...
    - H.264 (x264)
  - Encoder Preset: The slowest
  - Quality: The highest
- Audio:
  - Codec: FLAC 16-bit
  - Mixdown: The highest amount of channels (or anything but Mono)
