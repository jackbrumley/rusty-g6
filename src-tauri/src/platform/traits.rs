use tauri::WebviewWindow;

pub trait AudioSetup: Send + Sync {
    fn setup_microphone(&self) -> Result<String, String>;
    fn get_microphone_status(&self) -> Result<String, String>;
}

pub trait WindowManagement: Send + Sync {
    fn setup_window(&self, window: &WebviewWindow);
}

pub trait PlatformBackend: AudioSetup + WindowManagement + Send + Sync {}

impl<T> PlatformBackend for T where T: AudioSetup + WindowManagement + Send + Sync {}
