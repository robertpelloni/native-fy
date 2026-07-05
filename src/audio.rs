
pub struct AudioEngine {
    // Basic stub because headless environment has trouble compiling rodio with cpal/alsa linking properly
}

impl AudioEngine {
    pub fn new() -> Option<Self> {
        Some(Self {})
    }

    pub fn play(&self, _id: String, _url: String) {
        println!("Audio: Playing {} from {}", _id, _url);
    }

    pub fn stop(&self, _id: &str) {
        println!("Audio: Stopping {}", _id);
    }
}
