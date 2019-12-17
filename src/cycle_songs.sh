# TODO: don't download and move flies unessecarily if
# downloading beforehand
MUSDIR='./music'

############## Cycle mp3 files and download next track ##############

rm $"$MUSDIR/prev.mp3"
mv $"$MUSDIR/playing.mp3" $"$MUSDIR/prev.mp3"
mv $"$MUSDIR/next.mp3" $"$MUSDIR/playing.mp3"

# download next.mp3
echo $1
youtube-dl -o $"$MUSDIR/next.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 $"https://www.youtube.com/playlist?list=$1" >/dev/null
