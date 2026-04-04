use color_eyre::Result;
use rodio::{MixerDeviceSink, Source};
use std::io::Cursor;

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
        let cursor = Cursor::new(song);
        let song = rodio::Decoder::new(cursor)?.buffered();
        self.total_duration = song
            .total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        self.player.append(song);
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
