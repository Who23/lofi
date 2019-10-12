# TODO: don't download and move flies unessecarily if
# downloading beforehand
MUSDIR='./music/'

################### Download mp3 files if deleted ################### 

# download prev.mp3 if it does not exist
if ! [ -f $"$MUSDIR/prev.mp3" ]
then
    youtube-dl -o $"$MUSDIR/prev.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 "https://www.youtube.com/playlist?list=PL6NdkXsPL07KiewBDpJC1dFvxEubnNOp1" >/dev/null
fi

# download playing.mp3 if it does not exist
if ! [ -f $"$MUSDIR/playing.mp3" ]
then
    youtube-dl -o $"$MUSDIR/playing.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 "https://www.youtube.com/playlist?list=PL6NdkXsPL07KiewBDpJC1dFvxEubnNOp1" >/dev/null
fi

# download next.mp3 if it does not exist
if ! [ -f $"$MUSDIR/next.mp3" ]
then
    youtube-dl -o $"$MUSDIR/next.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 "https://www.youtube.com/playlist?list=PL6NdkXsPL07KiewBDpJC1dFvxEubnNOp1" >/dev/null
fi

############## Cycle mp3 files and download next track ##############

rm $"$MUSDIR/prev.mp3"
mv $"$MUSDIR/playing.mp3" $"$MUSDIR/prev.mp3"
mv $"$MUSDIR/next.mp3" $"$MUSDIR/playing.mp3"

# download next.mp3
youtube-dl -o $"$MUSDIR/next.%(ext)s" --max-downloads 1 --yes-playlist --playlist-random -x --audio-format mp3 "https://www.youtube.com/playlist?list=PL6NdkXsPL07KiewBDpJC1dFvxEubnNOp1" >/dev/null
