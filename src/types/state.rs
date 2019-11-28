pub struct State {
    pub is_playing: bool,
    pub at_playing_song: bool,  // are we at playing.mp3 or prev.mp3
    pub can_skip: bool,         // so that we cannot skip while next.mp3 downloads
    pub volume: f32,
}
