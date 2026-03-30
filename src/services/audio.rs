use color_eyre::Result;
use std::io::Cursor;

pub struct PlaybackService {
    sink: rodio::MixerDeviceSink,
    player: rodio::Player,
}

impl PlaybackService {
    pub fn new() -> Result<PlaybackService> {
        let sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(sink.mixer());
        Ok(PlaybackService { sink, player })
    }

    pub async fn play_new(&self, song: Vec<u8>) -> Result<()> {
        self.player.stop();
        let cursor = Cursor::new(song);
        let song = rodio::Decoder::new(cursor)?;
        self.player.append(song);
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

    pub fn stop(&self) -> Result<()> {
        self.player.stop();
        Ok(())
    }

    pub fn set_vol(&self, vol: f64) -> Result<()> {
        self.player.set_volume(vol as f32);
        Ok(())
    }
}
