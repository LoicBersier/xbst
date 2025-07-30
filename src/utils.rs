use std::path::PathBuf;

use zerocopy::TryFromBytes;

pub struct MusicFile {
    pub path: PathBuf,
    pub soundtrack_index: u32,
    pub soundtrack_name: String,
    pub index: u32,
}

// https://xboxdevwiki.net/Soundtracks#ST.DB

#[derive(Debug, TryFromBytes)]
#[repr(C)]
pub struct Header {
    pub magic: i32,
    pub num_soundtracks: i32,
    pub next_soundtrack_id: i32,
    pub soundtrack_ids: [i32; 100],
    pub next_song_id: i32,
    pub padding: [char; 24],
}

#[derive(Debug, TryFromBytes)]
#[repr(C)]
pub struct Soundtrack {
    pub magic: i32,
    pub id: i32,
    pub num_songs: u32,
    pub song_groups_ids: [i32; 84],
    pub total_time_miliseconds: i32,
    pub name: [[u8; 2]; 32],
    pub padding: [char; 24],
}

#[derive(Debug, TryFromBytes)]
#[repr(C)]
pub struct Song {
    pub magic: i32,
    pub soundtrack_id: i32,
    pub id: i32,
    pub ipadding: i32,
    pub song_id: [i32; 6],
    pub song_time_miliseconds: [i32; 6],
    pub song_name: [[u8; 2]; 192],
    pub cpadding: [char; 16],
}
