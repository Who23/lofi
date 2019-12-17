# Downloads the mp3 files if they are not already present
# It also replaces playing.mp3 and next.mp3 if they are not from the same playlist
# the user requested, by checking the last_playlist file.
# Run at the start of the program to make sure all mp3s are correct and there.

MUSDIR='./music/'

################### Download mp3 files if deleted ################### 
last_playlist=$"cat ./last_playlist"

# download prev.mp3 if it does not exist
if  ! [ -f $"$MUSDIR/prev.mp3" ] || [ ]
then
    youtube-dl -o $"$MUSDIR/prev.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 $"https://www.youtube.com/playlist?list=$1" >/dev/null
fi

# download playing.mp3 if it does not exist
if ! [ -f $"$MUSDIR/playing.mp3" ]
then
    youtube-dl -o $"$MUSDIR/playing.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 $"https://www.youtube.com/playlist?list=$1" >/dev/null
fi

# download next.mp3 if it does not exist
if ! [ -f $"$MUSDIR/next.mp3" ]
then
    youtube-dl -o $"$MUSDIR/next.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 $"https://www.youtube.com/playlist?list=$1" >/dev/null
fi
