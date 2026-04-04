use color_eyre::eyre::{Result, eyre};
use rodio::{MixerDeviceSink, Source};
use std::io::Cursor;
use std::num::{NonZeroU16, NonZeroU32};
use std::time::Duration;
use symphonia_adapter_fdk_aac::AacDecoder;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, CodecRegistry, Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct PlaybackService {
    _sink: MixerDeviceSink,
    player: rodio::Player,
    total_duration: f64,
}

impl PlaybackService {
    pub fn new() -> Result<PlaybackService> {
        let _sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(_sink.mixer());
        Ok(PlaybackService {
            _sink,
            player,
            total_duration: 0.0,
        })
    }

    pub async fn play_new(&mut self, song: Vec<u8>) -> Result<()> {
        self.player.stop();
        self.player.clear();

        let source = SymphoniaSource::new(song)?;

        self.total_duration = source.duration().unwrap_or(0.0);

        self.player.append(source);
        self.play()?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.player.pause();
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.player.play();
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        !self.player.is_paused()
    }

    pub fn position(&self) -> Option<f64> {
        if self.player.len() == 0 {
            return None;
        }
        Some(self.player.get_pos().as_secs_f64())
    }

    pub fn get_end(&self) -> f64 {
        self.total_duration
    }

    pub fn stop(&self) -> Result<()> {
        self.player.stop();
        Ok(())
    }

    pub fn set_vol(&self, vol: f64) -> Result<()> {
        self.player.set_volume(vol as f32);
        Ok(())
    }
}

struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_buf: Option<SampleBuffer<f32>>,
    sample_idx: usize,
    channels: u16,
    sample_rate: u32,
    duration: Option<f64>,
}

impl SymphoniaSource {
    pub fn new(song: Vec<u8>) -> Result<Self> {
        let cursor = Cursor::new(song);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        let probe = symphonia::default::get_probe();
        let mut codec_registry = CodecRegistry::new();
        codec_registry.register_all::<AacDecoder>();

        let format = probe
            .format(
                &Hint::new(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| eyre!("Failed to probe format: {:?}", e))?
            .format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| eyre!("No audio track found in file"))?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track
            .codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(2);

        let duration = if let Some(n_frames) = track.codec_params.n_frames {
            if let Some(tb) = track.codec_params.time_base {
                let time = tb.calc_time(n_frames);
                Some(time.seconds as f64 + time.frac)
            } else {
                Some(n_frames as f64 / sample_rate as f64)
            }
        } else {
            None
        };

        let decoder = codec_registry
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| eyre!("Failed to create decoder: {:?}", e))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_buf: None,
            sample_idx: 0,
            channels,
            sample_rate,
            duration,
        })
    }

    pub fn duration(&self) -> Option<f64> {
        self.duration
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(buf) = &self.sample_buf
            && self.sample_idx < buf.len()
        {
            let sample = buf.samples()[self.sample_idx];
            self.sample_idx += 1;
            return Some(sample);
        }

        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(_) => return None,
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf) => {
                    self.sample_rate = audio_buf.spec().rate;
                    self.channels = audio_buf.spec().channels.count() as u16;

                    let mut sample_buf =
                        SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec());
                    sample_buf.copy_interleaved_ref(audio_buf);

                    if !sample_buf.is_empty() {
                        let sample = sample_buf.samples()[0];
                        self.sample_idx = 1;
                        self.sample_buf = Some(sample_buf);
                        return Some(sample);
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => return None,
            }
        }
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        self.sample_buf
            .as_ref()
            .map(|buf| buf.len().saturating_sub(self.sample_idx))
    }

    fn channels(&self) -> NonZeroU16 {
        NonZeroU16::new(self.channels).unwrap()
    }

    fn sample_rate(&self) -> NonZeroU32 {
        NonZeroU32::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration.map(Duration::from_secs_f64)
    }
}
