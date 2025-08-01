use std::path::PathBuf;

use zerocopy::{Immutable, IntoBytes};

pub struct MusicFile {
    pub path: PathBuf,
    pub soundtrack_index: u32,
    pub soundtrack_name: String,
    pub index: u32,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Codec {
    Wmav1,
    Wmav2,
}

impl ToString for Codec {
    fn to_string(&self) -> String {
        match self {
            Codec::Wmav1 => String::from("wmav1"),
            Codec::Wmav2 => String::from("wmav2"),
        }
    }
}

// https://xboxdevwiki.net/Soundtracks#ST.DB
#[derive(Debug, Immutable, IntoBytes)]
#[repr(C)]
pub struct Header {
    pub magic: i32,
    pub num_soundtracks: i32,
    pub next_soundtrack_id: i32,
    pub soundtrack_ids: [i32; 100],
    pub next_song_id: i32,
    pub padding: [char; 24],
}

#[derive(Debug, Immutable, IntoBytes)]
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

#[derive(Debug, Immutable, IntoBytes)]
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

impl Default for Header {
    #[inline]
    fn default() -> Header {
        Header {
            magic: 0,
            num_soundtracks: 0,
            next_soundtrack_id: 0,
            soundtrack_ids: [0; 100],
            next_song_id: 0,
            padding: [char::MIN; 24],
        }
    }
}

impl Default for Soundtrack {
    #[inline]
    fn default() -> Soundtrack {
        Soundtrack {
            magic: 0,
            id: 0,
            num_songs: 0,
            song_groups_ids: [0; 84],
            total_time_miliseconds: 0,
            name: [[0; 2]; 32],
            padding: [char::MIN; 24],
        }
    }
}

impl Default for Song {
    #[inline]
    fn default() -> Song {
        Song {
            magic: 0,
            soundtrack_id: 0,
            id: 0,
            ipadding: 0,
            song_id: [0; 6],
            song_time_miliseconds: [0; 6],
            song_name: [[0; 2]; 192],
            cpadding: [char::MIN; 16],
        }
    }
}
