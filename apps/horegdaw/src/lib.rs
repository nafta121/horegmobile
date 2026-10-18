//! HOREGDAW native Android entry point (`libhoregdaw.so` via `android.app.NativeActivity`).
//!
//! Runs the full pure-Rust DAW (`soundcraft_engine`, `soundcraft_dsp`, `soundcraft_mix`,
//! `soundcraft_playback` via Android AAudio, and `soundcraft_ui_egui::SoundApp`) natively on
//! Android using `eframe` + `wgpu` (Vulkan / OpenGL ES) and `winit`'s `android-native-activity`.
#![deny(unsafe_code, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_os = "android")]
mod config;
#[cfg(target_os = "android")]
mod logging;

#[cfg(target_os = "android")]
use soundcraft_ui_egui::{Services, SoundApp, UiState, dialogs};
#[cfg(target_os = "android")]
use std::path::PathBuf;
#[cfg(target_os = "android")]
use std::sync::{Arc, OnceLock};

#[cfg(target_os = "android")]
static ANDROID_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "android")]
struct AndroidHostApp(SoundApp, Option<&'static logging::AppLogger>);

#[cfg(target_os = "android")]
impl eframe::App for AndroidHostApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(logger) = self.1 {
            logger.report_audio_thread();
        }
        self.0.guarded(|app| app.logic(ctx));
        if self.0.quit_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if ctx.input(|i| i.viewport().close_requested()) && dialogs::ask_to_save(&mut self.0, "app.quit", &serde_json::json!({})) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        self.0.ui.ui_scale = self.0.extra.ui_scale;
        if self.0.take_recent_changed() {
            save_prefs(&self.0.ui);
        }
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.guarded(|app| app.ui(ui));
    }

    fn on_exit(&mut self) {
        self.0.close_autosave();
        save_prefs(&self.0.ui);
        if let Some(logger) = self.1 {
            logger.report_audio_thread();
        }
        self.0.player = None;
        soundcraft_engine::shutdown_plugin_hosts();
    }
}

#[cfg(target_os = "android")]
fn config_dir() -> Option<PathBuf> {
    ANDROID_DATA_DIR.get().cloned().or_else(config::config_dir)
}

#[cfg(target_os = "android")]
fn prefs_path() -> Option<PathBuf> {
    config_dir().map(|b| b.join("ui.json"))
}

#[cfg(target_os = "android")]
fn mobile_default_prefs() -> UiState {
    let mut ui = UiState::default();
    ui.mobile_layout = true;
    ui.show_tracks_list = false;
    ui.show_clip_list = false;
    ui.narrow_mix = true;
    ui.ui_scale = 1.0;
    ui
}

#[cfg(target_os = "android")]
fn load_prefs() -> Option<UiState> {
    let path = prefs_path()?;
    match config::load_ui(&path) {
        Ok(state) => state,
        Err(error) => {
            log::warn!("{error}");
            None
        }
    }
}

#[cfg(target_os = "android")]
fn save_prefs(ui: &UiState) {
    if let Some(path) = prefs_path()
        && let Err(error) = config::save_ui(&path, ui)
    {
        log::warn!("{error}");
    }
}

#[cfg(target_os = "android")]
fn services() -> Services {
    Services {
        pick_open: None,
        pick_save: Some(Box::new(|purpose: &str, default: &str| {
            let base = config_dir()?;
            let folder = match purpose {
                "file.bounce_mix" => base.join("Bounces"),
                _ => base.join("Sessions"),
            };
            let _ = std::fs::create_dir_all(&folder);
            let file_name = std::path::Path::new(default)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(if purpose == "file.bounce_mix" { "Bounce.wav" } else { "Session.scraft" });
            Some(folder.join(file_name).to_string_lossy().into_owned())
        })),
        pick_folder: Some(Box::new(|_start: &str| {
            let base = config_dir()?;
            let sessions = base.join("Sessions");
            let _ = std::fs::create_dir_all(&sessions);
            Some(sessions.to_string_lossy().into_owned())
        })),
    }
}

/// Native Android `NativeActivity` entry point invoked by the Android runtime when loading `libhoregdaw.so`.
#[cfg(target_os = "android")]
#[allow(unsafe_code)]
// SAFETY: `android_main` is the required C ABI entry symbol for `android-activity` (`android-native-activity`).
#[unsafe(no_mangle)]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let logger = logging::install();
    if let Some(data_dir) = app.external_data_path().or_else(|| app.internal_data_path()) {
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = ANDROID_DATA_DIR.set(data_dir);
    }
    if let Some(logger) = logger
        && let Some(dir) = config_dir()
    {
        let _ = logger.attach_dir(&dir.join(logging::LOG_DIR));
    }
    log::info!("HOREGDAW {} (android {})", env!("CARGO_PKG_VERSION"), std::env::consts::ARCH);

    let engine = soundcraft_engine::demo::demo_engine();
    let prefs = load_prefs().unwrap_or_else(mobile_default_prefs);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("HOREGDAW").with_app_id("com.horegdaw.app"),
        android_app: Some(app),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "HOREGDAW",
        options,
        Box::new(move |_cc| {
            let player = Some(soundcraft_playback::Player::with_config(Arc::new(engine.session().clone()), &prefs.audio));
            let mut sound_app = SoundApp::new(engine, player, services());
            sound_app.autosave_dir = prefs_path().and_then(|p| p.parent().map(|d| d.join("Autosave")));
            sound_app.leftovers = sound_app.autosave_dir.as_deref().map(soundcraft_engine::autosave::leftovers).unwrap_or_default();
            sound_app.preset_dir = prefs_path().and_then(|p| p.parent().map(|d| d.join("Presets")));
            sound_app.extra.ui_scale = prefs.ui_scale;
            sound_app.ui = prefs;
            Ok(Box::new(AndroidHostApp(sound_app, logger)))
        }),
    ) {
        log::error!("HOREGDAW native Android runner exited with error: {err}");
    }
}
