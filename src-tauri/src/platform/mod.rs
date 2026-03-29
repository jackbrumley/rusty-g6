pub mod linux;
pub mod traits;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::initialize;

#[cfg(target_os = "windows")]
pub use windows::initialize;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn initialize() -> std::sync::Arc<dyn traits::PlatformBackend> {
    unimplemented!("Platform initialization not implemented for this OS yet");
}
