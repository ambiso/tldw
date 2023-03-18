# tl;dw

## Demo

![tldw being executed on https://youtu.be/HeBP3MG-WHg and showing how tldw downloads the automatically generated subtitles and generates a summary of the video](demo.webp)

## Install

```
sudo pacman -S yt-dlp
cargo install --git https://github.com/ambiso/tldw.git
```

## Usage

You need to have `yt-dlp` installed and an `api_key.txt` in the current directory, that contains your OpenAI API key.

```
tldw <YOUTUBE URL>
```

