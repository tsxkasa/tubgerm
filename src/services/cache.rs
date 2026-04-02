use std::collections::HashMap;

use submarine::data::{AlbumWithSongsId3, Child, PlaylistWithSongs};

#[derive(Default, Debug, Clone)]
pub struct LibaryCache {
    pub playlist_cache: HashMap<String, PlaylistWithSongs>,
    pub album_cache: HashMap<String, AlbumWithSongsId3>,
    pub liked_cache: HashMap<String, Child>,
}
