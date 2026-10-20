use std::path::Path;

use anyhow::{Context, Result};
use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::read_from_path;
use lofty::tag::{Tag, TagType};

/// Writes or updates metadata tags (ID3v2 for mp3, VorbisComments for flac/ogg, Mp4Ilst for m4a)
/// and optionally embeds a front cover art image.
pub fn write_audio_tags(
    path: &Path,
    title: &str,
    artists: &[String],
    album: Option<&str>,
    year: Option<u32>,
    cover_data: Option<(&[u8], &str)>,
) -> Result<()> {
    let mut tagged_file = read_from_path(path)
        .with_context(|| format!("Failed to read audio file for tagging: {:?}", path))?;

    let preferred_tag_type = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp3") => TagType::Id3v2,
        Some("flac") | Some("ogg") | Some("opus") => TagType::VorbisComments,
        Some("m4a") => TagType::Mp4Ilst,
        _ => TagType::Id3v2,
    };

    let tag = if tagged_file.primary_tag().is_some() {
        tagged_file.primary_tag_mut().expect("primary tag exists")
    } else if tagged_file.first_tag().is_some() {
        tagged_file.first_tag_mut().expect("first tag exists")
    } else {
        tagged_file.insert_tag(Tag::new(preferred_tag_type));
        if tagged_file.tag_mut(preferred_tag_type).is_some() {
            tagged_file.tag_mut(preferred_tag_type).expect("tag exists")
        } else if tagged_file.primary_tag_mut().is_some() {
            tagged_file.primary_tag_mut().expect("primary tag exists")
        } else {
            tagged_file
                .first_tag_mut()
                .context("Failed to obtain tag handle after creation")?
        }
    };

    tag.set_title(title.to_string());
    tag.set_artist(artists.join(", "));

    if let Some(alb) = album {
        let trimmed = alb.trim();
        if !trimmed.is_empty() {
            tag.set_album(trimmed.to_string());
        }
    }

    if let Some(y) = year {
        tag.set_year(y);
    }

    if let Some((bytes, mime)) = cover_data {
        let mime_type = match mime.to_ascii_lowercase().as_str() {
            "image/png" | "png" => MimeType::Png,
            "image/jpeg" | "image/jpg" | "jpg" | "jpeg" => MimeType::Jpeg,
            other => MimeType::from_str(other),
        };
        let pic = Picture::new_unchecked(
            PictureType::CoverFront,
            Some(mime_type),
            None,
            bytes.to_vec(),
        );
        tag.remove_picture_type(PictureType::CoverFront);
        tag.push_picture(pic);
    }

    tagged_file
        .save_to_path(path, WriteOptions::default())
        .with_context(|| format!("Failed to save audio tags to {:?}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    fn create_minimal_wav_file(path: &Path) {
        let pcm_data = vec![0u8; 32];
        let subchunk2_size = pcm_data.len() as u32;
        let chunk_size = 36 + subchunk2_size;

        let mut data = Vec::with_capacity(44 + pcm_data.len());
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&chunk_size.to_le_bytes());
        data.extend_from_slice(b"WAVE");
        data.extend_from_slice(b"fmt ");
        data.extend_from_slice(&(16u32).to_le_bytes()); // PCM subchunk size
        data.extend_from_slice(&(1u16).to_le_bytes());  // AudioFormat = 1 (PCM)
        data.extend_from_slice(&(1u16).to_le_bytes());  // NumChannels = 1
        data.extend_from_slice(&(44100u32).to_le_bytes()); // SampleRate = 44100
        data.extend_from_slice(&(88200u32).to_le_bytes()); // ByteRate = 44100 * 1 * 16/8
        data.extend_from_slice(&(2u16).to_le_bytes());  // BlockAlign = 1 * 16/8
        data.extend_from_slice(&(16u16).to_le_bytes()); // BitsPerSample = 16
        data.extend_from_slice(b"data");
        data.extend_from_slice(&subchunk2_size.to_le_bytes());
        data.extend_from_slice(&pcm_data);

        let mut file = File::create(path).expect("failed to create dummy wav");
        file.write_all(&data).expect("failed to write dummy wav");
    }

    #[test]
    fn test_write_and_read_audio_tags() {
        let temp_dir = std::env::temp_dir().join(format!("mt_tagger_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let audio_path = temp_dir.join("test_sample.wav");

        create_minimal_wav_file(&audio_path);

        let artists = vec!["Pink Floyd".to_string(), "David Gilmour".to_string()];
        let fake_cover_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4\x00\x00\x00\nIDATx\x9cc`\x00\x00\x00\x02\x00\x01H\xaf\xa4q\x00\x00\x00\x00IEND\xaeB`\x82";

        let result = write_audio_tags(
            &audio_path,
            "Comfortably Numb",
            &artists,
            Some("The Wall"),
            Some(1979),
            Some((fake_cover_png, "image/png")),
        );

        assert!(result.is_ok(), "Tagging failed: {:?}", result.err());

        // Read back and verify with lofty
        let read_file = read_from_path(&audio_path).expect("failed to read tagged file");
        let tag = read_file
            .primary_tag()
            .or_else(|| read_file.first_tag())
            .expect("should have a tag");

        assert_eq!(tag.title().as_deref(), Some("Comfortably Numb"));
        assert_eq!(tag.artist().as_deref(), Some("Pink Floyd, David Gilmour"));
        assert_eq!(tag.album().as_deref(), Some("The Wall"));
        assert_eq!(tag.year(), Some(1979));
        assert_eq!(tag.pictures().len(), 1);
        assert_eq!(tag.pictures()[0].pic_type(), PictureType::CoverFront);
        assert_eq!(tag.pictures()[0].data(), fake_cover_png);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
