use anyhow::Result;
use lofty::{
    Accessor, AudioFile, ItemKey, ItemValue, MimeType, PictureType, Tag, TagExt, TagItem,
    TaggedFile, TaggedFileExt,
};

#[derive(Debug)]
pub struct Picture {
    /// The picture's MIME type.
    pub mime_type: String,
    /// The image data.
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Metadata {
    pub title: Option<String>,
    pub duration_ms: Option<f64>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<u16>,
    pub track_total: Option<u16>,
    pub disc_number: Option<u16>,
    pub disc_total: Option<u16>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub picture: Option<Picture>,
    pub file_size: Option<u64>,
    pub replay_gain_album_gain: Option<f64>,
    pub replay_gain_album_peak: Option<f64>,
    pub replay_gain_track_gain: Option<f64>,
    pub replay_gain_track_peak: Option<f64>,
}

pub fn read_metadata(file: String) -> Result<Metadata> {
    let (tagged_file, tag) = get_tag_for_file(&file)?;
    let cover = tag
        .get_picture_type(PictureType::CoverFront)
        .or(tag.pictures().first());
    Ok(Metadata {
        title: tag.title().and_then(|s| Some(s.to_string())),
        duration_ms: Some(tagged_file.properties().duration().as_millis() as f64),
        album: tag.album().and_then(|s| Some(s.to_string())),
        album_artist: tag
            .get(&ItemKey::AlbumArtist)
            .and_then(|s| match s.value() {
                ItemValue::Text(t) => Some(t.to_string()),
                _ => None,
            }),
        artist: tag.artist().and_then(|s| Some(s.to_string())),
        track_number: tag.track().map(|f| f as u16),
        track_total: tag.track_total().map(|f| f as u16),
        disc_number: tag.disk().map(|f| f as u16),
        disc_total: tag.disk_total().map(|f| f as u16),
        year: tag.year().map(|f| f as i32),
        genre: tag.genre().and_then(|s| Some(s.to_string())),
        picture: (match cover {
            Some(cover) => Some(Picture {
                mime_type: cover.mime_type().to_string(),
                data: cover.data().to_vec(),
            }),
            None => None,
        }),
        file_size: Some(21231_u64),
        replay_gain_album_gain: get_f64_from_tag(&tag, ItemKey::ReplayGainAlbumGain),
        replay_gain_album_peak: get_f64_from_tag(&tag, ItemKey::ReplayGainAlbumPeak),
        replay_gain_track_gain: get_f64_from_tag(&tag, ItemKey::ReplayGainTrackGain),
        replay_gain_track_peak: get_f64_from_tag(&tag, ItemKey::ReplayGainTrackPeak),
    })
}

pub fn write_metadata(file: String, metadata: Metadata) -> Result<()> {
    let (_tagged_file, mut tag) = get_tag_for_file(&file)?;

    if metadata.title.is_some() {
        tag.set_title(metadata.title.unwrap());
    }
    if metadata.album.is_some() {
        tag.set_album(metadata.album.unwrap());
    }
    if metadata.album_artist.is_some() {
        tag.insert(TagItem::new(
            ItemKey::AlbumArtist,
            ItemValue::Text(metadata.album_artist.unwrap()),
        ));
    }
    if metadata.artist.is_some() {
        tag.set_artist(metadata.artist.unwrap());
    }
    if metadata.track_number.is_some() {
        tag.set_track(metadata.track_number.unwrap() as u32);
    }
    if metadata.track_total.is_some() {
        tag.set_track_total(metadata.track_total.unwrap() as u32);
    }
    if metadata.disc_number.is_some() {
        tag.set_disk(metadata.disc_number.unwrap() as u32);
    }
    if metadata.disc_total.is_some() {
        tag.set_disk_total(metadata.disc_total.unwrap() as u32);
    }
    if metadata.year.is_some() {
        tag.set_year(metadata.year.unwrap() as u32);
    }
    if metadata.genre.is_some() {
        tag.set_genre(metadata.genre.unwrap());
    }
    if metadata.picture.is_some() {
        let image = metadata.picture.unwrap();
        tag.push_picture(lofty::Picture::new_unchecked(
            PictureType::CoverFront,
            MimeType::from_str(&image.mime_type),
            None,
            image.data,
        ));
    }

    tag.save_to_path(&file)?;
    Ok(())
}

fn get_tag_for_file(file: &str) -> Result<(TaggedFile, Tag)> {
    let mut tagged_file = lofty::read_from_path(file)?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                eprintln!("WARN: No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().unwrap()
            }
        }
    }
    .to_owned();
    Ok((tagged_file, tag))
}

fn get_f64_from_tag(tag: &Tag, key: ItemKey) -> Option<f64> {
    tag.get(&key).and_then(|s| match s.value() {
        ItemValue::Text(t) => t.split(" ").next().and_then(|f| f.parse::<f64>().ok()),
        _ => None,
    })
}
