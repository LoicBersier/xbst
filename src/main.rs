use std::{
    ffi::OsStr,
    fs::{self, read_dir, File},
    io::{stdout, Write},
    path::PathBuf,
    process::Command,
    string::FromUtf8Error,
};

mod utils;

use clap::Parser;
use deunicode::AsciiChars;
use thiserror::Error;
use zerocopy::IntoBytes;

use crate::utils::{Header, MusicFile, Song, Soundtrack};

#[derive(Error, Debug)]
enum Errors {
    #[error("Couldn't find your input folder. {}", .0.kind())]
    UnknownFolder(#[source] std::io::Error),
    #[error(transparent)]
    UnknownIO(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8(#[from] FromUtf8Error),
    #[error("Skill issue on the programmer part ngl, report this to dev pls")]
    SkillIssue(),
    #[error("Didn't find any file to convert, is your input folder structured correctly?")]
    NoFileToConvert(),

    #[error("You are missing ffprobe in your PATH")]
    MissingFfprobe(#[source] std::io::Error),
    #[error("You are missing ffmpeg in your PATH")]
    MissingFfmpeg(#[source] std::io::Error),
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Input folder of your musics
    #[arg(default_value = "./music")]
    input: String,
    /// Output folder for the database and converted musics
    #[arg(default_value = "./output")]
    output: String,
    /// Bitrate for the output
    #[arg(short, long, default_value_t = 128)]
    bitrate: i16,
}

fn main() {
    let args = Args::parse();

    println!(
        "
⠀⠀⠀⠂⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠈⠲⣥⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⠖⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠘⠿⣿⣷⣦⣀⡀⠀⠀⢀⣠⣴⣾⣿⠟⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣿⣿⣿⣿⣿⣿⣿⣿⡟⠁⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢼⣿⣿⣿⣿⣿⣷⡋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢀⢾⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢠⣺⣿⣽⣿⠟⠁⠈⠻⣿⣷⣾⣆⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢀⢴⣾⣿⡿⠈⠀⠀⠀⠀⠀⠀⠑⢿⣿⣷⡄⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣠⣾⣿⠟⠉⠀⠀⠀XBST⠀⠀⠀⠈⠻⣿⣦⡀⠀⠀⠀⠀
⠀⠀⢠⣪⠟⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠿⣔⠀⠀⠀
⢀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠢⡀
"
    );

    match process(&args) {
        Ok(_) => (),
        Err(e) => eprintln!("\r\x1B[K\x1b[0;31m{}\x1b[0;20m", e),
    }
}

fn process(args: &Args) -> Result<(), Errors> {
    let mut soundtrack_count: i32 = 0;
    let mut songs_count: u32 = 0;
    let mut total_songs_count: u32 = 0;
    let mut total_song_groups_count: i32 = 0;
    let mut song_time_miliseconds: [i32; 6] = [0; 6];
    let mut song_group_id: i32;

    let mut soundtracks: Vec<Soundtrack> = Default::default();
    let mut songs: Vec<Song> = Default::default();
    let mut sound_groups_ids: Vec<i32> = Vec::with_capacity(84);

    let mut files_to_convert: Vec<MusicFile> = Vec::new();

    let input_path = PathBuf::from(&args.input);
    let music_directory = read_dir(input_path).map_err(Errors::UnknownFolder)?;

    // Loop through each folders for the soundtrack struct
    for (i, soundtrack_dirs) in music_directory.enumerate() {
        let soundtrack = soundtrack_dirs.map_err(Errors::UnknownFolder)?.path();

        // Ignore non folders for soundtracks
        if !soundtrack.is_dir() {
            continue;
        }

        soundtrack_count += 1;
        song_group_id = 0;

        let soundtrack_name_str = soundtrack
            .file_name()
            .map_or(OsStr::new("Unknown soundtrack"), |f| f)
            .to_string_lossy()
            .trim()
            .ascii_chars()
            .to_string();

        // Convert the folder name into 2 bytes
        let mut soundtrack_name = soundtrack_name_str
            .bytes()
            .map(|b| [b, 0])
            .collect::<Vec<[u8; 2]>>();
        // Max value of 32
        soundtrack_name.resize(32, [0; 2]);

        let mut song_name: [[u8; 2]; 192] = [[0; 2]; 192];
        let files = read_dir(soundtrack)
            .map_err(Errors::UnknownIO)?
            .collect::<Vec<_>>();

        // Loop through each files in chunk of 6 (max songs allowed in a song group)
        files.chunks(6).for_each(|song_files| {
            let mut song_id: [i32; 6] = [0; 6];

            song_group_id += 1;
            sound_groups_ids.push(total_song_groups_count);
            total_song_groups_count += 1;

            let mut char_count = 0;
            song_time_miliseconds = [0; 6];

            for (g, f) in song_files.iter().enumerate() {
                let song = f.as_ref().unwrap();
                let song_path = song.path();

                // Ignore non files for song groups
                if !song_path.is_file() {
                    continue;
                }

                song_id[g] = total_songs_count as i32;
                song_time_miliseconds[g] = match get_duration(song_path) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("\x1b[0;31mFailed to get duration: {}\x1b[0;20m", e);
                        0
                    }
                };

                songs_count += 1;
                total_songs_count += 1;

                let filepath = song.path();
                let filename = filepath
                    .file_stem()
                    .map_or(OsStr::new("Unknown track"), |f| f)
                    .to_string_lossy()
                    .ascii_chars()
                    .to_string();

                let mut name = filename.trim().bytes().collect::<Vec<u8>>();
                name.resize(32, 0);

                for b in name.iter() {
                    song_name[char_count] = [*b, 0];
                    char_count += 1;
                }

                files_to_convert.push(MusicFile {
                    path: song.path(),
                    soundtrack_name: soundtrack_name_str.clone(),
                    soundtrack_index: 0,
                    index: total_songs_count - 1,
                });
            }

            let s = Song {
                magic: 200819,
                id: song_group_id - 1,
                ipadding: 0,
                soundtrack_id: i as i32,
                song_id,
                song_time_miliseconds,
                song_name,
                cpadding: [char::MIN; 16],
            };

            songs.push(s);

            song_name = [[0; 2]; 192];
        });

        let mut total_time_miliseconds: i32 = 0;

        for s in &songs {
            if s.soundtrack_id == i as i32 {
                total_time_miliseconds += s.song_time_miliseconds.iter().sum::<i32>()
            }
        }

        sound_groups_ids.resize(84, 0);
        let song_groups_ids: [i32; 84] = sound_groups_ids
            .clone()
            .try_into()
            .map_err(|_| Errors::SkillIssue())?;

        let st = Soundtrack {
            magic: 136049,
            id: i as i32,
            num_songs: songs_count,
            song_groups_ids: song_groups_ids,
            total_time_miliseconds,
            name: soundtrack_name
                .try_into()
                .map_err(|_| Errors::SkillIssue())?,
            padding: [char::MIN; 24],
        };

        soundtracks.push(st);
        sound_groups_ids = Vec::with_capacity(84);
        songs_count = 0;
    }

    let mut soundtrack_ids: [i32; 100] = [0; 100];
    for i in 0..soundtrack_count {
        soundtrack_ids[i as usize] = i;
    }

    let header = Header {
        magic: 0001,
        num_soundtracks: soundtrack_count,
        next_soundtrack_id: soundtrack_count + 1,
        soundtrack_ids,
        next_song_id: (songs_count as i32),
        padding: [char::MIN; 24],
    };

    if files_to_convert.len() == 0 {
        return Err(Errors::NoFileToConvert());
    }

    write_database(&args.output, header, soundtracks, songs)?;

    for f in files_to_convert {
        let percentage: f64 = ((f.index + 1) as f64 / total_songs_count as f64) * 100.0;
        print!(
            "{}{}\r{:3}% [{}{}] {:3}/{}",
            "\x1B[1A",
            "\x1B[K",
            percentage as usize,
            {
                let mut bar = "=".repeat(percentage as usize / 3);
                if percentage < 100.0 {
                    bar += ">"
                }
                bar
            },
            " ".repeat(100 / 3 - percentage as usize / 3),
            f.index + 1,
            total_songs_count
        );

        print!(
            "{}\r{}Processing {} - {}",
            "\x1B[1B",
            "\x1B[K",
            f.soundtrack_name,
            f.path
                .file_stem()
                .map_or(OsStr::new("Unknown track"), |f| f)
                .to_string_lossy()
        );

        stdout().flush().map_err(Errors::UnknownIO)?;

        convert_to_wma(
            f.path,
            &args.output,
            args.bitrate,
            f.soundtrack_index as usize,
            f.index as usize,
        )?;
    }

    print!("\x1B[1A\x1B[K\r\x1B[K Done.");

    Ok(())
}

fn get_duration(path: PathBuf) -> Result<i32, Errors> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path.into_os_string())
        .output()
        .map_err(Errors::MissingFfprobe)?;

    let binding = String::from_utf8(output.stdout).map_err(Errors::FromUtf8)?;
    let stdout = binding.trim();

    Ok((stdout.parse::<f32>().unwrap() * 1000.0) as i32)
}

fn convert_to_wma(
    input: PathBuf,
    output: &String,
    bitrate: i16,
    soundtrack_index: usize,
    song_index: usize,
) -> Result<(), Errors> {
    let binding = input.into_os_string();
    let input = binding.to_str().unwrap();

    fs::create_dir_all(format!("{}/{:0>4}", output, soundtrack_index))
        .map_err(Errors::UnknownIO)?;

    Command::new("ffmpeg")
        .args([
            "-i",
            input,
            "-acodec",
            "wmav1",
            "-ac",
            "2",
            "-ar",
            "44100",
            "-b:a",
            &format!("{}k", bitrate),
            "-map_metadata",
            "-1",
            "-map",
            "0:a",
            "-y",
            &format!(
                "{}/{:0>4}/{:0>8x}.wma",
                output, soundtrack_index, song_index
            ),
        ])
        .output()
        .map_err(Errors::MissingFfmpeg)?;

    Ok(())
}

fn write_database(
    output: &String,
    header: Header,
    soundtracks: Vec<Soundtrack>,
    songs: Vec<Song>,
) -> Result<(), Errors> {
    fs::create_dir_all(format!("{}/", &output)).map_err(Errors::UnknownIO)?;
    let mut database = File::create(format!("{}/ST.DB", &output)).map_err(Errors::UnknownIO)?;

    database.write_all(header.as_bytes())?;

    database.write_all(soundtracks.as_bytes())?;

    for _ in 0..100 - &soundtracks.len() {
        database.write_all(&[0 as u8; 512])?;
    }

    database.write_all(songs.as_bytes())?;

    Ok(())
}
