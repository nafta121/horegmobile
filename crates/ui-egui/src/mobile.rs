//! Cubasis 3–inspired mobile touch UI/UX for HOREGDAW.
//!
//! Designed specifically for phone and tablet screens:
//! 1. **Top Transport & Mode Bar (`44px`)**: Media/Session drawer, MIDI Keys toggle, Mixer toggle,
//!    `Bars.Beats.Sub` readout, large touch transport (`|<`, `>|`, Loop, Play, Record, Metronome),
//!    Tempo readout, and FX / Setup / Help buttons.
//! 2. **Contextual Tool & Action Strip (`42px`)**: One-tap touch buttons with icon + caption:
//!    `SEL`, `SPLIT`, `GLUE`, `ERASE`, `DRAW`, `MUTE`, `TRANSP`, `QUANT`, `1/16`, `STRETCH`,
//!    `UNDO`, `REDO`, `COPY`, `PASTE`, and `# GRID`.
//! 3. **Left Vertical Inspector Rail (`42px`) + Collapsible Inspector Drawer**:
//!    Rounded-square tab icons (`#7c8894` inactive, `#3ea6eb` active) for Track Overview, Routing,
//!    Insert FX, Sends, Automation, Notes/Instrument, Channel Strip Fader, Track Color, and Full Menu.
//! 4. **Compact Numbered Track Headers (`128px`) + Full-Bleed Colored Clip Arranger**:
//!    High-contrast numbered color badges (`1`, `2`, `3`, …), silver-white selected track plate,
//!    clean bar ruler (`1`, `3`, `5`, `7`, …), and full-bleed colored clips with bold labels.

use crate::dialogs::Dialog;
use crate::theme::{bold, rgb};
use crate::{MainWindow, SoundApp};
use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;
use soundcraft_model::{AutoParam, ClipContent, Route, Tool, Track, TrackId};
use soundcraft_time::{GridValue, NoteValue, Samples};

const TOP_BAR_H: f32 = 44.0;
const SUB_BAR_H: f32 = 42.0;
const RAIL_W: f32 = 42.0;
const DRAWER_W: f32 = 188.0;
const MEDIA_DRAWER_W: f32 = 220.0;
const TRACK_HEADER_W: f32 = 128.0;
const NUM_BADGE_W: f32 = 34.0;
const RULER_H: f32 = 26.0;

/// Cubasis 3–inspired vibrant rainbow track palette (pink, magenta, purple, indigo, blue, cyan,
/// teal, green, lime, yellow, amber, orange, coral, red).
const CUBASIS_PALETTE: [[u8; 3]; 14] = [
    [224, 92, 138],  // 1: Rose Pink
    [214, 94, 178],  // 2: Magenta
    [164, 102, 212], // 3: Purple
    [118, 108, 216], // 4: Indigo
    [90, 128, 220],  // 5: Royal Blue
    [76, 162, 224],  // 6: Sky Blue
    [68, 192, 206],  // 7: Cyan
    [68, 196, 152],  // 8: Seafoam Green
    [88, 196, 104],  // 9: Emerald Green
    [146, 204, 76],  // 10: Lime Green
    [212, 204, 72],  // 11: Sunny Yellow
    [232, 166, 68],  // 12: Warm Amber
    [232, 122, 68],  // 13: Vivid Orange
    [228, 86, 86],   // 14: Coral Red
];

fn cubasis_track_color(index: usize) -> Color32 {
    let c = CUBASIS_PALETTE.get(index % CUBASIS_PALETTE.len()).copied().unwrap_or([76, 162, 224]);
    rgb(c)
}

/// Entry point for the Cubasis 3–style mobile layout.
pub fn show(app: &mut SoundApp, ui: &mut Ui) {
    crate::edit_window::drive_drag(app, ui.ctx());

    let bg_dark = Color32::from_rgb(24, 28, 33);
    let top_bg = Color32::from_rgb(39, 47, 55);
    let sub_bg = Color32::from_rgb(24, 28, 33);

    // 1. Top Transport & Mode Bar
    egui::Panel::top("mobile_top_bar")
        .exact_size(TOP_BAR_H)
        .frame(egui::Frame::NONE.fill(top_bg))
        .show(ui, |ui| top_transport_bar(app, ui));

    // 2. Contextual Action & Tool Sub-Bar
    egui::Panel::top("mobile_sub_bar")
        .exact_size(SUB_BAR_H)
        .frame(egui::Frame::NONE.fill(sub_bg))
        .show(ui, |ui| action_sub_bar(app, ui));

    // 3. Optional bottom MIDI Editor / Piano Roll drawer when Keys mode is active
    if app.ui.show_midi_editor {
        let avail_h = ui.available_height();
        let midi_h = (avail_h * 0.44).clamp(130.0, 260.0);
        egui::Panel::bottom("mobile_midi_drawer")
            .exact_size(midi_h)
            .frame(egui::Frame::NONE.fill(bg_dark))
            .show(ui, |ui| {
                crate::midi_editor::show(app, ui);
            });
    }

    // 4. Main Workspace: Left Inspector Rail + Optional Drawer + Arranger or Touch Mixer
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg_dark))
        .show(ui, |ui| {
            let full = ui.max_rect();

            // Left Vertical Inspector Rail (42px)
            let rail_rect = Rect::from_min_max(full.min, pos2(full.min.x + RAIL_W, full.max.y));
            let mut rail_ui = ui.new_child(egui::UiBuilder::new().max_rect(rail_rect));
            inspector_rail(app, &mut rail_ui, rail_rect);

            let mut content_x = rail_rect.max.x;

            // Optional collapsible Inspector Drawer (188px)
            if app.ui.mobile_inspector_open {
                let drawer_rect = Rect::from_min_max(pos2(content_x, full.min.y), pos2((content_x + DRAWER_W).min(full.max.x - 120.0), full.max.y));
                let mut drawer_ui = ui.new_child(egui::UiBuilder::new().max_rect(drawer_rect));
                inspector_drawer(app, &mut drawer_ui, drawer_rect);
                content_x = drawer_rect.max.x;
            }

            // Optional collapsible Media / Session Drawer (220px)
            if app.ui.mobile_media_open {
                let media_rect = Rect::from_min_max(pos2(content_x, full.min.y), pos2((content_x + MEDIA_DRAWER_W).min(full.max.x - 120.0), full.max.y));
                let mut media_ui = ui.new_child(egui::UiBuilder::new().max_rect(media_rect));
                media_drawer(app, &mut media_ui, media_rect);
                content_x = media_rect.max.x;
            }

            // Remaining area: Arranger Timeline or Touch Mixer
            let body_rect = Rect::from_min_max(pos2(content_x, full.min.y), full.max);
            let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body_rect));
            match app.ui.window {
                MainWindow::Edit => arranger_view(app, &mut body_ui, body_rect),
                MainWindow::Mix => mobile_mixer_view(app, &mut body_ui, body_rect),
            }
        });
}

/// Row 1: Cubasis 3–style Top Transport, Bar.Beat Counter, Tempo, and View Switchers.
fn top_transport_bar(app: &mut SoundApp, ui: &mut Ui) {
    let r = ui.max_rect();
    let sky = Color32::from_rgb(62, 166, 235);
    let icon_fg = Color32::from_rgb(228, 234, 240);
    let dim_fg = Color32::from_rgb(162, 174, 186);
    let sep_col = Color32::from_rgb(24, 29, 35);

    ui.painter().hline(r.x_range(), r.max.y - 0.5, Stroke::new(1.0, sep_col));

    let mut x = r.min.x + 6.0;
    let btn_h = r.height() - 6.0;
    let cy = r.center().y;

    // --- Left Mode Buttons: Media, Keys, Mixer ---
    // 1. Media / Project button (with HOREGDAW icon)
    let media_r = Rect::from_center_size(pos2(x + 22.0, cy), vec2(40.0, btn_h));
    if top_mode_button(ui, media_r, app.ui.mobile_media_open, sky, "MediaBay & Project").clicked() {
        app.ui.mobile_media_open = !app.ui.mobile_media_open;
    }
    crate::icons::draw(ui.painter(), Rect::from_center_size(media_r.center(), vec2(24.0, 24.0)), "horegdaw", Color32::WHITE);
    x = media_r.max.x + 4.0;

    // 2. Keys / MIDI Editor toggle
    let keys_r = Rect::from_center_size(pos2(x + 22.0, cy), vec2(40.0, btn_h));
    if top_mode_button(ui, keys_r, app.ui.show_midi_editor, sky, "MIDI Piano Roll & Keyboard").clicked() {
        let _ = app.run("window.midi_editor", json!({}));
    }
    draw_piano_keys_icon(ui.painter(), keys_r.center(), if app.ui.show_midi_editor { sky } else { icon_fg });
    x = keys_r.max.x + 4.0;

    // 3. Mixer view toggle
    let is_mix = app.ui.window == MainWindow::Mix;
    let mix_r = Rect::from_center_size(pos2(x + 22.0, cy), vec2(40.0, btn_h));
    if top_mode_button(ui, mix_r, is_mix, sky, "Toggle Touch Mixer / Arranger").clicked() {
        app.ui.window = if is_mix { MainWindow::Edit } else { MainWindow::Mix };
    }
    draw_faders_icon(ui.painter(), mix_r.center(), if is_mix { sky } else { icon_fg });
    x = mix_r.max.x + 12.0;

    ui.painter().vline(x, (r.min.y + 6.0)..=(r.max.y - 6.0), Stroke::new(1.0, sep_col));
    x += 10.0;

    // --- Position Readout (Bars.Beats.Subbeats, e.g. "45.4.1" or "1.1.1") ---
    let pos_samples = app.position();
    let bbt_text = format_bars_beats_compact(app.engine.session(), pos_samples);
    let pos_r = Rect::from_min_size(pos2(x, r.min.y + 4.0), vec2(82.0, btn_h));
    let pos_resp = ui.interact(pos_r, ui.id().with("mob_pos_counter"), Sense::click());
    ui.painter().text(pos_r.center(), Align2::CENTER_CENTER, &bbt_text, bold(18.0), Color32::WHITE);
    if pos_resp.clicked() {
        let _ = app.run("transport.rtz", json!({}));
    }
    pos_resp.on_hover_text("Playhead Position (Bar.Beat.Tick) — Tap to return to 1.1.1");
    x = pos_r.max.x + 8.0;

    ui.painter().vline(x, (r.min.y + 6.0)..=(r.max.y - 6.0), Stroke::new(1.0, sep_col));
    x += 8.0;

    // --- Center Transport Controls: Prev |<, Next >|, Loop, Play |>, Record O, Metronome ---
    let trans_w = 42.0;
    // Prev (Return to Zero)
    let prev_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, prev_r, false, sky, "Return to Start (1.1.1)").clicked() {
        let _ = app.run("transport.rtz", json!({}));
    }
    draw_transport_prev(ui.painter(), prev_r.center(), icon_fg);
    x += trans_w + 2.0;

    // Next (Go to End)
    let next_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, next_r, false, sky, "Go to Session End").clicked() {
        let _ = app.run("transport.go_to_end", json!({}));
    }
    draw_transport_next(ui.painter(), next_r.center(), icon_fg);
    x += trans_w + 2.0;

    // Loop / Cycle
    let loop_on = app.engine.session().edit.loop_playback;
    let loop_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, loop_r, loop_on, sky, "Loop Playback").clicked() {
        let _ = app.run("options.loop_playback", json!({}));
    }
    draw_transport_loop(ui.painter(), loop_r.center(), if loop_on { sky } else { icon_fg });
    x += trans_w + 2.0;

    // Play / Stop
    let playing = app.is_playing();
    let play_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, play_r, playing, sky, "Play / Stop").clicked() {
        let cmd = if playing { "transport.stop" } else { "transport.play" };
        let _ = app.run(cmd, json!({}));
    }
    draw_transport_play(ui.painter(), play_r.center(), if playing { sky } else { icon_fg });
    x += trans_w + 2.0;

    // Record
    let rec_on = app.engine.transport.recording;
    let rec_col = Color32::from_rgb(239, 68, 68);
    let rec_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, rec_r, rec_on, rec_col, "Record").clicked() {
        let any_armed = app.engine.session().tracks.iter().any(|t| t.mixer.record_arm);
        if !any_armed {
            let target = app
                .engine
                .session()
                .edit
                .selected_tracks
                .first()
                .copied()
                .or_else(|| app.engine.session().tracks.first().map(|t| t.id));
            if let Some(tid) = target {
                let _ = app.run("mix.rec_arm", json!({"track": tid.0}));
            }
        }
        let _ = app.run("transport.record", json!({}));
    }
    ui.painter().circle_filled(rec_r.center(), 7.5, if rec_on { rec_col } else { icon_fg });
    x += trans_w + 2.0;

    // Metronome / Click
    let click_on = app.engine.session().edit.click;
    let click_r = Rect::from_center_size(pos2(x + trans_w * 0.5, cy), vec2(trans_w, btn_h));
    if top_mode_button(ui, click_r, click_on, sky, "Metronome Click").clicked() {
        let _ = app.run("options.click", json!({}));
    }
    draw_metronome_icon(ui.painter(), click_r.center(), if click_on { sky } else { icon_fg });
    x += trans_w + 8.0;

    ui.painter().vline(x, (r.min.y + 6.0)..=(r.max.y - 6.0), Stroke::new(1.0, sep_col));
    x += 8.0;

    // --- Tempo Readout ("120.0" / "104.0") ---
    let bpm = {
        let s = app.engine.session();
        let tick = s.tempo.samples_to_ticks(pos_samples.max(0), s.sample_rate);
        s.tempo.tempo_at_tick(tick)
    };
    let bpm_r = Rect::from_min_size(pos2(x, r.min.y + 4.0), vec2(74.0, btn_h));
    let bpm_resp = ui.interact(bpm_r, ui.id().with("mob_bpm_readout"), Sense::click_and_drag());
    ui.painter().text(bpm_r.center(), Align2::CENTER_CENTER, format!("{bpm:.1}"), bold(18.0), Color32::WHITE);
    if bpm_resp.dragged() {
        let dy = -bpm_resp.drag_delta().y * 0.25;
        if dy.abs() > 0.01 {
            let new_bpm = (bpm + f64::from(dy)).clamp(30.0, 300.0);
            let _ = app.run("event.tempo", json!({"bpm": new_bpm, "at": 0}));
        }
    }
    egui::Popup::from_toggle_button_response(&bpm_resp)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(170.0);
            ui.label(egui::RichText::new("TEMPO (BPM)").strong());
            ui.horizontal(|ui| {
                if ui.button(" -5 ").clicked() {
                    let _ = app.run("event.tempo", json!({"bpm": (bpm - 5.0).clamp(30.0, 300.0), "at": 0}));
                }
                if ui.button(" -1 ").clicked() {
                    let _ = app.run("event.tempo", json!({"bpm": (bpm - 1.0).clamp(30.0, 300.0), "at": 0}));
                }
                if ui.button(" +1 ").clicked() {
                    let _ = app.run("event.tempo", json!({"bpm": (bpm + 1.0).clamp(30.0, 300.0), "at": 0}));
                }
                if ui.button(" +5 ").clicked() {
                    let _ = app.run("event.tempo", json!({"bpm": (bpm + 5.0).clamp(30.0, 300.0), "at": 0}));
                }
            });
            ui.horizontal(|ui| {
                for preset in [90.0, 104.0, 120.0, 128.0, 140.0] {
                    if ui.selectable_label((bpm - preset).abs() < 0.1, format!("{preset:.0}")).clicked() {
                        let _ = app.run("event.tempo", json!({"bpm": preset, "at": 0}));
                        ui.close();
                    }
                }
            });
        });

    // --- Right Utility Buttons: AudioSuite/FX, Setup Gear, Help `?` ---
    let mut rx = r.max.x - 6.0;

    // Help / About `?`
    let help_r = Rect::from_center_size(pos2(rx - 19.0, cy), vec2(36.0, btn_h));
    if top_mode_button(ui, help_r, app.ui.show_about, sky, "About HOREGDAW").clicked() {
        let _ = app.run("app.about", json!({}));
    }
    ui.painter().text(help_r.center(), Align2::CENTER_CENTER, "?", bold(18.0), dim_fg);
    rx = help_r.min.x - 4.0;

    // Setup / Audio Gear
    let gear_r = Rect::from_center_size(pos2(rx - 19.0, cy), vec2(36.0, btn_h));
    if top_mode_button(ui, gear_r, app.ui.show_playback_engine, sky, "Setup & Playback Engine").clicked() {
        let _ = app.run("setup.playback_engine", json!({}));
    }
    draw_gear_icon(ui.painter(), gear_r.center(), if app.ui.show_playback_engine { sky } else { dim_fg });
    rx = gear_r.min.x - 4.0;

    // AudioSuite / FX bag icon
    let fx_open = app.ui.audiosuite.is_some();
    let fx_r = Rect::from_center_size(pos2(rx - 19.0, cy), vec2(36.0, btn_h));
    let fx_resp = top_mode_button(ui, fx_r, fx_open, sky, "AudioSuite Offline FX & Bounce");
    draw_fx_bag_icon(ui.painter(), fx_r.center(), if fx_open { sky } else { dim_fg });
    egui::Popup::from_toggle_button_response(&fx_resp)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(180.0);
            ui.label(egui::RichText::new("AUDIOSUITE & EXPORT").strong());
            if ui.button("Bounce Mix…").clicked() {
                let _ = app.dialogs.open_for_command(&app.engine, "file.bounce");
                ui.close();
            }
            ui.separator();
            for p in soundcraft_dsp::plugins().iter().filter(|p| !p.is_instrument) {
                if ui.button(p.name).clicked() {
                    let _ = app.run("audiosuite.open", json!({"plugin": p.id}));
                    ui.close();
                }
            }
        });
}

/// Row 2: Cubasis 3–style Action & Tool Strip (`SEL`, `SPLIT`, `GLUE`, `ERASE`, `DRAW`, `MUTE`,
/// `TRANSP`, `QUANT`, `1/16`, `STRETCH`, `UNDO`, `REDO`, `COPY`, `PASTE`, `# 1/8`).
fn action_sub_bar(app: &mut SoundApp, ui: &mut Ui) {
    let r = ui.max_rect();
    let sky = Color32::from_rgb(62, 166, 235);
    let border = Color32::from_rgb(38, 45, 53);
    ui.painter().hline(r.x_range(), r.max.y - 0.5, Stroke::new(1.0, border));

    let cur_tool = app.engine.session().edit.tool;
    let cur_mode = app.engine.session().edit.edit_mode;
    let grid_val = app.engine.session().edit.grid;
    let grid_str = grid_label(grid_val);

    let ncols = 15usize;
    let col_w = (r.width() / ncols as f32).max(38.0);

    for idx in 0..ncols {
        let x0 = r.min.x + idx as f32 * col_w;
        let cell = Rect::from_min_max(pos2(x0, r.min.y), pos2(x0 + col_w, r.max.y));
        if idx > 0 {
            ui.painter().vline(cell.min.x, (cell.min.y + 4.0)..=(cell.max.y - 4.0), Stroke::new(1.0, border));
        }

        let (caption, active, kind) = match idx {
            0 => ("SEL", matches!(cur_tool, Tool::Selector | Tool::Grabber | Tool::Smart), SubIcon::Select),
            1 => ("SPLIT", false, SubIcon::Split),
            2 => ("GLUE", false, SubIcon::Glue),
            3 => ("ERASE", false, SubIcon::Erase),
            4 => ("DRAW", cur_tool == Tool::Pencil, SubIcon::Draw),
            5 => ("MUTE", false, SubIcon::Mute),
            6 => ("TRANSP", false, SubIcon::Transpose),
            7 => ("QUANT", false, SubIcon::Quantize),
            8 => (grid_str.as_str(), false, SubIcon::GridNote),
            9 => ("STRETCH", cur_tool == Tool::Trim, SubIcon::Stretch),
            10 => ("UNDO", app.engine.undo_label().is_some(), SubIcon::Undo),
            11 => ("REDO", app.engine.redo_label().is_some(), SubIcon::Redo),
            12 => ("COPY", false, SubIcon::Copy),
            13 => ("PASTE", false, SubIcon::Paste),
            _ => (grid_str.as_str(), cur_mode.is_grid() || app.engine.session().edit.grid_lines, SubIcon::SnapGrid),
        };

        let resp = ui.interact(cell, ui.id().with(("mob_sub", idx)), Sense::click());
        let hovered = resp.hovered();

        if idx == 14 && active {
            // Rightmost `# 1/8` Grid Snap pill gets the signature Cubasis sky-blue highlight box
            let pill = cell.shrink2(vec2(3.0, 3.0));
            ui.painter().rect_filled(pill, 4.0, sky);
            draw_sub_icon(ui.painter(), pos2(cell.center().x, cell.min.y + 13.0), kind, Color32::from_rgb(18, 24, 30));
            ui.painter().text(pos2(cell.center().x, cell.max.y - 8.5), Align2::CENTER_CENTER, caption, bold(8.5), Color32::from_rgb(18, 24, 30));
        } else {
            if active && idx != 0 {
                ui.painter().rect_filled(cell.shrink(1.0), 3.0, Color32::from_rgb(34, 52, 68));
            } else if hovered {
                ui.painter().rect_filled(cell.shrink(1.0), 3.0, Color32::from_rgb(32, 38, 46));
            }
            let fg = if active && idx != 0 {
                sky
            } else if matches!(idx, 10 | 11) && !active {
                Color32::from_rgb(96, 106, 118)
            } else {
                Color32::from_rgb(206, 214, 224)
            };
            draw_sub_icon(ui.painter(), pos2(cell.center().x, cell.min.y + 13.0), kind, fg);
            ui.painter().text(pos2(cell.center().x, cell.max.y - 8.5), Align2::CENTER_CENTER, caption, bold(8.5), fg);
        }

        if idx == 8 {
            // Grid resolution popup on `1/16`
            egui::Popup::from_toggle_button_response(&resp)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_min_width(120.0);
                    ui.label(egui::RichText::new("QUANTIZE / GRID").strong());
                    for v in ["1 bar", "1/2", "1/4", "1/8", "1/16", "1/32", "1/8t", "1/16t"] {
                        if ui.button(v).clicked() {
                            let _ = app.run("view.grid", json!({"value": v}));
                            ui.close();
                        }
                    }
                });
        } else if resp.clicked() {
            match idx {
                0 => {
                    let next = if cur_tool == Tool::Grabber { "selector" } else { "grabber" };
                    let _ = app.run("edit.tool", json!({"tool": next}));
                }
                1 => {
                    let _ = app.run("edit.separate", json!({}));
                }
                2 => {
                    let _ = app.run("edit.consolidate", json!({}));
                }
                3 => {
                    let _ = app.run("edit.clear", json!({}));
                }
                4 => {
                    let next = if cur_tool == Tool::Pencil { "grabber" } else { "pencil" };
                    let _ = app.run("edit.tool", json!({"tool": next}));
                }
                5 => {
                    if !app.engine.session().edit.selected_clips.is_empty() {
                        let _ = app.run("edit.mute_clips", json!({}));
                    } else if let Some(tid) = app.engine.session().edit.selected_tracks.first().copied() {
                        let _ = app.run("mix.mute", json!({"track": tid.0}));
                    }
                }
                6 => {
                    let _ = app.run("window.transpose", json!({}));
                }
                7 => {
                    let _ = app.run("clip.quantize_to_grid", json!({}));
                }
                9 => {
                    let next = if cur_tool == Tool::Trim { "grabber" } else { "trim" };
                    let _ = app.run("edit.tool", json!({"tool": next}));
                }
                10 => {
                    let _ = app.run("app.undo", json!({}));
                }
                11 => {
                    let _ = app.run("app.redo", json!({}));
                }
                12 => {
                    let _ = app.run("edit.copy", json!({}));
                }
                13 => {
                    let _ = app.run("edit.paste", json!({}));
                }
                14 => {
                    let next = if cur_mode.is_grid() { "slip" } else { "grid" };
                    let _ = app.run("edit.mode", json!({"mode": next}));
                    let _ = app.run("view.grid_lines", json!({}));
                }
                _ => {}
            }
        }
    }
}

/// Left Vertical Inspector Rail (`42px` wide) with rounded-square icon buttons matching Cubasis 3.
fn inspector_rail(app: &mut SoundApp, ui: &mut Ui, rect: Rect) {
    let rail_bg = Color32::from_rgb(30, 36, 43);
    let inactive_box = Color32::from_rgb(122, 134, 146);
    let active_box = Color32::from_rgb(62, 166, 235);
    let dark_icon = Color32::from_rgb(24, 29, 35);
    let border = Color32::from_rgb(20, 24, 29);

    ui.painter().rect_filled(rect, 0.0, rail_bg);
    ui.painter().vline(rect.max.x - 0.5, rect.y_range(), Stroke::new(1.0, border));

    let tabs: [(u8, &str, RailIcon); 8] = [
        (0, "Track Overview (Vol / Pan / M / S / R)", RailIcon::PlayArrow),
        (1, "Routing (Input / Output / Format)", RailIcon::Routing),
        (2, "Insert Effects (Slots A–E)", RailIcon::Inserts),
        (3, "Send Effects (Buses A–E)", RailIcon::Sends),
        (4, "Automation Mode & Lane", RailIcon::Automation),
        (5, "Instrument & Track Notes", RailIcon::Notes),
        (6, "Channel Strip Fader & Meter", RailIcon::Fader),
        (7, "Track & Clip Color Palette", RailIcon::ColorDrop),
    ];

    let btn_w = 32.0;
    let btn_h = 26.0;
    let gap = 4.0;
    let mut y = rect.min.y + 6.0;

    for (tab_id, tip, icon) in tabs {
        if y + btn_h > rect.max.y - 34.0 {
            break;
        }
        let br = Rect::from_center_size(pos2(rect.center().x, y + btn_h * 0.5), vec2(btn_w, btn_h));
        let is_active = app.ui.mobile_inspector_tab == tab_id;
        let resp = ui.interact(br, ui.id().with(("rail_tab", tab_id)), Sense::click());
        let fill = if is_active {
            active_box
        } else if resp.hovered() {
            inactive_box.lerp_to_gamma(Color32::WHITE, 0.18)
        } else {
            inactive_box
        };
        ui.painter().rect_filled(br, CornerRadius::same(4), fill);
        draw_rail_icon(ui.painter(), br.center(), icon, dark_icon);

        if resp.clicked() {
            if app.ui.mobile_inspector_open && app.ui.mobile_inspector_tab == tab_id {
                app.ui.mobile_inspector_open = false;
            } else {
                app.ui.mobile_inspector_tab = tab_id;
                app.ui.mobile_inspector_open = true;
            }
        }
        resp.on_hover_text(tip);
        y += btn_h + gap;
    }

    // Bottom button: Full HOREGDAW Menu (`≡`) so every command in the DAW is accessible
    let menu_r = Rect::from_center_size(pos2(rect.center().x, rect.max.y - 18.0), vec2(btn_w, btn_h));
    let menu_resp = ui.interact(menu_r, ui.id().with("rail_full_menu"), Sense::click());
    let menu_fill = if menu_resp.hovered() { active_box } else { inactive_box };
    ui.painter().rect_filled(menu_r, CornerRadius::same(4), menu_fill);
    draw_rail_icon(ui.painter(), menu_r.center(), RailIcon::Hamburger, dark_icon);
    egui::Popup::from_toggle_button_response(&menu_resp)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            crate::menus::full_menu_popup(app, ui);
        });
    menu_resp.on_hover_text("All HOREGDAW Menus (File, Edit, Track, Clip, View, Options, Setup)");
}

/// Collapsible Inspector Drawer next to the left rail (`188px` wide) for the selected track.
fn inspector_drawer(app: &mut SoundApp, ui: &mut Ui, rect: Rect) {
    let bg = Color32::from_rgb(29, 35, 42);
    let header_bg = Color32::from_rgb(38, 46, 55);
    let border = Color32::from_rgb(18, 22, 27);
    let sky = Color32::from_rgb(62, 166, 235);

    ui.painter().rect_filled(rect, 0.0, bg);
    ui.painter().vline(rect.max.x - 0.5, rect.y_range(), Stroke::new(1.0, border));

    let sel_id = app
        .engine
        .session()
        .edit
        .selected_tracks
        .first()
        .copied()
        .or_else(|| app.engine.session().tracks.first().map(|t| t.id));

    let hdr = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_H));
    ui.painter().rect_filled(hdr, 0.0, header_bg);
    ui.painter().hline(hdr.x_range(), hdr.max.y - 0.5, Stroke::new(1.0, border));

    let title = match app.ui.mobile_inspector_tab {
        0 => "TRACK INSPECTOR",
        1 => "ROUTING & I/O",
        2 => "INSERT EFFECTS",
        3 => "BUS SENDS",
        4 => "AUTOMATION",
        5 => "INSTRUMENT / NOTES",
        6 => "CHANNEL STRIP",
        _ => "TRACK COLOR",
    };
    ui.painter().text(pos2(hdr.min.x + 8.0, hdr.center().y), Align2::LEFT_CENTER, title, bold(10.0), sky);

    let close_r = Rect::from_center_size(pos2(hdr.max.x - 14.0, hdr.center().y), vec2(20.0, 20.0));
    if ui.interact(close_r, ui.id().with("close_insp"), Sense::click()).clicked() {
        app.ui.mobile_inspector_open = false;
    }
    ui.painter().text(close_r.center(), Align2::CENTER_CENTER, "×", bold(14.0), Color32::from_rgb(180, 190, 200));

    let body = Rect::from_min_max(pos2(rect.min.x + 8.0, hdr.max.y + 6.0), pos2(rect.max.x - 8.0, rect.max.y - 6.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));

    let Some(tid) = sel_id else {
        child.label("No track in session.");
        if child.button("+ Add Audio Track").clicked() {
            let _ = app.run("track.new", json!({"count": 1, "kind": "audio", "format": "stereo"}));
        }
        return;
    };
    let Some(track) = app.engine.session().track(tid).cloned() else { return };

    child.horizontal(|ui| {
        let idx = app.engine.session().track_index(tid).unwrap_or(0);
        let col = cubasis_track_color(idx);
        let (badge, _) = ui.allocate_exact_size(vec2(20.0, 18.0), Sense::hover());
        ui.painter().rect_filled(badge, 3.0, col);
        ui.painter().text(badge.center(), Align2::CENTER_CENTER, format!("{}", idx + 1), bold(10.0), Color32::from_rgb(20, 24, 28));
        ui.label(egui::RichText::new(&track.name).strong().color(Color32::WHITE));
    });
    child.separator();

    egui::ScrollArea::vertical().id_salt("mob_insp_scroll").auto_shrink([false, false]).show(&mut child, |ui| {
        match app.ui.mobile_inspector_tab {
            0 => inspector_tab_overview(app, ui, &track),
            1 => inspector_tab_routing(app, ui, &track),
            2 => inspector_tab_inserts(app, ui, &track),
            3 => inspector_tab_sends(app, ui, &track),
            4 => inspector_tab_automation(app, ui, &track),
            5 => inspector_tab_notes(app, ui, &track),
            6 => inspector_tab_fader(app, ui, &track),
            _ => inspector_tab_color(app, ui, &track),
        }
    });
}

fn inspector_tab_overview(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    ui.horizontal(|ui| {
        if ui.selectable_label(track.mixer.mute, "MUTE").clicked() {
            let _ = app.run("mix.mute", json!({"track": tid}));
        }
        if ui.selectable_label(track.mixer.solo, "SOLO").clicked() {
            let _ = app.run("mix.solo", json!({"track": tid}));
        }
        if ui.selectable_label(track.mixer.record_arm, "REC").clicked() {
            let _ = app.run("mix.rec_arm", json!({"track": tid}));
        }
        if ui.selectable_label(track.mixer.input_monitor, "MON").clicked() {
            let _ = app.run("mix.input_monitor", json!({"track": tid}));
        }
    });
    ui.add_space(6.0);

    // Volume slider
    let mut db = track.mixer.volume_db;
    ui.label(egui::RichText::new(format!("Volume: {:.1} dB", db)).size(11.5));
    if ui.add(egui::Slider::new(&mut db, -60.0..=12.0).show_value(false)).changed() {
        let _ = app.run("mix.volume", json!({"track": tid, "db": db}));
    }
    if ui.small_button("Reset 0.0 dB").clicked() {
        let _ = app.run("mix.volume", json!({"track": tid, "db": 0.0}));
    }

    ui.add_space(6.0);
    // Pan slider
    let mut pan = track.mixer.pan.first().copied().unwrap_or(0.0);
    ui.label(egui::RichText::new(format!("Pan: {:.0}%", pan * 100.0)).size(11.5));
    if ui.add(egui::Slider::new(&mut pan, -1.0..=1.0).show_value(false)).changed() {
        let _ = app.run("mix.pan", json!({"track": tid, "pan": pan}));
    }
    if ui.small_button("Center Pan").clicked() {
        let _ = app.run("mix.pan", json!({"track": tid, "pan": 0.0}));
    }

    ui.add_space(8.0);
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Rename…").clicked() {
            app.dialogs.open = Some(Dialog::RenameTrack { id: track.id, name: track.name.clone() });
        }
        if ui.button("Duplicate").clicked() {
            let _ = app.run("track.duplicate", json!({"track": tid}));
        }
    });
    if ui.button("Delete Track").clicked() {
        let _ = app.run("track.delete", json!({"track": tid}));
    }
}

fn inspector_tab_routing(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    ui.label(egui::RichText::new(format!("Kind: {:?} ({:?})", track.kind, track.format)).size(11.5));
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Output Route:").strong());
    let cur_out = match &track.mixer.output {
        Route::Main => "Main Out".to_string(),
        Route::Bus(b) => format!("Bus {}", b.0),
        Route::Hardware(ch) => format!("Out {ch}"),
        Route::None => "No Output".to_string(),
    };
    ui.label(format!("Current: {cur_out}"));
    if ui.selectable_label(matches!(track.mixer.output, Route::Main), "Main Stereo Out").clicked() {
        let _ = app.run("mix.output", json!({"track": tid, "route": "main"}));
    }
    let buses: Vec<(u64, String)> = app.engine.session().busses.iter().map(|b| (b.id.0, b.name.clone())).collect();
    for (bid, bname) in buses {
        let active = matches!(track.mixer.output, Route::Bus(id) if id.0 == bid);
        if ui.selectable_label(active, format!("Bus: {bname}")).clicked() {
            let _ = app.run("mix.output", json!({"track": tid, "route": {"bus": bid}}));
        }
    }
}

fn inspector_tab_inserts(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    for slot in 0..5 {
        let ins = track.mixer.inserts.get(slot).and_then(|x| x.as_ref());
        let label = ins
            .and_then(|i| crate::mix_window::plugin_info(&i.plugin).map(|p| p.name))
            .or_else(|| ins.map(|i| i.plugin.as_str()))
            .unwrap_or("— Empty Slot —");
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}:", (b'A' + slot as u8) as char)).strong());
            let btn = ui.button(label);
            if btn.clicked() && ins.is_some() && !app.ui.plugin_windows.contains(&(track.id, slot)) {
                app.ui.plugin_windows.push((track.id, slot));
            }
            egui::Popup::from_toggle_button_response(&btn)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    crate::mix_window::plugin_menu(app, ui, track.id, slot, ins.is_some());
                });
        });
    }
}

fn inspector_tab_sends(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    let buses: Vec<(u64, String)> = app.engine.session().busses.iter().map(|b| (b.id.0, b.name.clone())).collect();
    for slot in 0..5 {
        let send = track.mixer.sends.get(slot).and_then(|x| x.as_ref());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}:", (b'A' + slot as u8) as char)).strong());
            if let Some(s) = send {
                let bname = match &s.target {
                    Route::Bus(b) => app.engine.session().bus(*b).map_or("Bus", |b| b.name.as_str()).to_string(),
                    _ => "Out".to_string(),
                };
                let mut lvl = s.level_db;
                ui.label(&bname);
                if ui.add(egui::Slider::new(&mut lvl, -60.0..=12.0).show_value(false)).changed() {
                    let _ = app.run("mix.send_level", json!({"track": tid, "slot": slot, "db": lvl}));
                }
                if ui.small_button("×").clicked() {
                    let _ = app.run("mix.send_remove", json!({"track": tid, "slot": slot}));
                }
            } else if let Some((first_bus, first_name)) = buses.first() {
                if ui.button(format!("+ Send to {first_name}")).clicked() {
                    let _ = app.run("mix.send", json!({"track": tid, "slot": slot, "bus": first_bus}));
                }
            } else {
                ui.label("No buses");
            }
        });
    }
}

fn inspector_tab_automation(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    ui.label(egui::RichText::new("Track View:").strong());
    for (view_id, label) in [
        ("waveform", "Waveform / Clips"),
        ("volume", "Volume Automation"),
        ("pan", "Pan Automation"),
        ("mute", "Mute Automation"),
    ] {
        if ui.selectable_label(track.view == view_id, label).clicked() {
            let _ = app.run("track.view", json!({"track": tid, "view": view_id}));
        }
    }
    ui.separator();
    ui.label(egui::RichText::new("Automation Mode:").strong());
    let cur_mode = format!("{:?}", track.mixer.automation_mode).to_lowercase();
    for mode in ["off", "read", "touch", "latch", "write"] {
        if ui.selectable_label(cur_mode == mode, mode.to_uppercase()).clicked() {
            let _ = app.run("track.automation_mode", json!({"track": tid, "mode": mode}));
        }
    }
}

fn inspector_tab_notes(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    if track.kind.is_midi() {
        ui.label(egui::RichText::new("Virtual Instrument:").strong());
        let cur = track.instrument.as_ref().map_or("None", |i| i.plugin.as_str());
        ui.label(format!("Active: {cur}"));
        for p in soundcraft_dsp::plugins().iter().filter(|p| p.is_instrument) {
            if ui.selectable_label(cur == p.id, p.name).clicked() {
                let _ = app.run("mix.instrument", json!({"track": tid, "plugin": p.id}));
            }
        }
        ui.separator();
    }
    ui.label(egui::RichText::new("Track Comments:").strong());
    let mut notes = track.comments.clone();
    if ui.text_edit_multiline(&mut notes).changed() {
        let _ = app.run("track.comments", json!({"track": tid, "comments": notes}));
    }
}

fn inspector_tab_fader(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    let meter = app.meters.get(&track.id).copied().unwrap_or_default();
    ui.label(egui::RichText::new(format!("{:.1} dB", track.mixer.volume_db)).strong().size(14.0));
    ui.horizontal(|ui| {
        let mut db = track.mixer.volume_db;
        let slider = egui::Slider::new(&mut db, -60.0..=12.0).vertical().show_value(false);
        if ui.add_sized([34.0, 140.0], slider).changed() {
            let _ = app.run("mix.volume", json!({"track": tid, "db": db}));
        }
        let (mr, _) = ui.allocate_exact_size(vec2(24.0, 140.0), Sense::hover());
        ui.painter().rect_filled(mr, 3.0, Color32::from_rgb(16, 20, 24));
        for ch in 0..2 {
            let lvl = meter.level.get(ch).copied().unwrap_or(0.0).clamp(0.0, 1.2);
            let h = lvl.clamp(0.0, 1.0) * (mr.height() - 4.0);
            let bar = Rect::from_min_max(
                pos2(mr.min.x + 2.0 + ch as f32 * 10.0, mr.max.y - 2.0 - h),
                pos2(mr.min.x + 10.0 + ch as f32 * 10.0, mr.max.y - 2.0),
            );
            let col = if lvl > 0.95 {
                Color32::from_rgb(239, 68, 68)
            } else if lvl > 0.7 {
                Color32::from_rgb(234, 179, 8)
            } else {
                Color32::from_rgb(52, 211, 153)
            };
            ui.painter().rect_filled(bar, 1.5, col);
        }
    });
}

fn inspector_tab_color(app: &mut SoundApp, ui: &mut Ui, track: &Track) {
    let tid = track.id.0;
    ui.label(egui::RichText::new("Tap a color for this track:").size(11.5));
    ui.add_space(4.0);
    egui::Grid::new("mob_color_grid").spacing(vec2(6.0, 6.0)).show(ui, |ui| {
        for (i, c) in soundcraft_model::TRACK_COLORS.iter().enumerate() {
            let (r, resp) = ui.allocate_exact_size(vec2(30.0, 22.0), Sense::click());
            ui.painter().rect_filled(r, 4.0, rgb(*c));
            if track.color == *c {
                ui.painter().rect_stroke(r, 4.0, Stroke::new(2.0, Color32::WHITE), StrokeKind::Inside);
            }
            if resp.clicked() {
                let _ = app.run("track.color", json!({"tracks": [tid], "color": i}));
            }
            if (i + 1) % 4 == 0 {
                ui.end_row();
            }
        }
    });
}

/// MediaBay & Project Drawer (opened via the top-left Media button).
fn media_drawer(app: &mut SoundApp, ui: &mut Ui, rect: Rect) {
    let bg = Color32::from_rgb(27, 33, 40);
    let header_bg = Color32::from_rgb(38, 46, 55);
    let border = Color32::from_rgb(18, 22, 27);
    let sky = Color32::from_rgb(62, 166, 235);

    ui.painter().rect_filled(rect, 0.0, bg);
    ui.painter().vline(rect.max.x - 0.5, rect.y_range(), Stroke::new(1.0, border));

    let hdr = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_H));
    ui.painter().rect_filled(hdr, 0.0, header_bg);
    ui.painter().text(pos2(hdr.min.x + 8.0, hdr.center().y), Align2::LEFT_CENTER, "MEDIABAY & PROJECT", bold(10.0), sky);

    let close_r = Rect::from_center_size(pos2(hdr.max.x - 14.0, hdr.center().y), vec2(20.0, 20.0));
    if ui.interact(close_r, ui.id().with("close_media"), Sense::click()).clicked() {
        app.ui.mobile_media_open = false;
    }
    ui.painter().text(close_r.center(), Align2::CENTER_CENTER, "×", bold(14.0), Color32::from_rgb(180, 190, 200));

    let body = Rect::from_min_max(pos2(rect.min.x + 6.0, hdr.max.y + 6.0), pos2(rect.max.x - 6.0, rect.max.y - 6.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(body));

    child.horizontal_wrapped(|ui| {
        if ui.button("Demo Project").clicked() {
            let _ = app.run("session.new", json!({"template": "demo"}));
        }
        if ui.button("New Empty").clicked() {
            let _ = app.dialogs.open_for_command(&app.engine, "session.new");
        }
        if ui.button("Save").clicked() {
            let _ = app.run("ui.menu.invoke", json!({"command": "session.save"}));
        }
        if ui.button("Bounce WAV").clicked() {
            let _ = app.dialogs.open_for_command(&app.engine, "file.bounce");
        }
    });
    child.separator();
    crate::panels::clip_list(app, &mut child);
}

/// Main Cubasis 3–style Arranger View: Compact Track Headers (`128px`) + Bar Ruler + Colored Clips.
fn arranger_view(app: &mut SoundApp, ui: &mut Ui, full: Rect) {
    let hscroll_h = 12.0;
    let hw = TRACK_HEADER_W;
    let tl = Rect::from_min_max(
        pos2(full.min.x + hw, full.min.y + RULER_H),
        pos2(full.max.x, full.max.y - hscroll_h),
    );
    let hdr_top = Rect::from_min_max(full.min, pos2(full.min.x + hw, full.min.y + RULER_H));
    let ruler_rect = Rect::from_min_max(pos2(full.min.x + hw, full.min.y), pos2(full.max.x, full.min.y + RULER_H));

    app.edit_layout.timeline = [tl.min.x, tl.min.y, tl.max.x, tl.max.y];

    ui.painter().rect_filled(full, 0.0, Color32::from_rgb(25, 30, 36));

    // Scroll & pinch/wheel zoom on the timeline
    crate::edit_window::scroll_and_zoom(app, ui, full, tl);

    // 1. Track List Sub-Header (`<->` Inspector toggle, `↕` Row height toggle, `+` Add Track)
    draw_track_subheader(app, ui, hdr_top);

    // 2. Clean Cubasis Bar Ruler (`1`, `3`, `5`, `7`, ... + Loop L/R markers)
    draw_cubasis_ruler(app, ui, ruler_rect, tl);

    // 3. Track Rows (Compact numbered headers + full-bleed colored clip lanes)
    let row_h = app.ui.mobile_row_height.clamp(28.0, 80.0);
    let mut y = tl.min.y - app.edit_layout.scroll_y;
    app.edit_layout.rows.clear();
    let mut content_h = 0.0;

    let visible_tracks: Vec<(usize, TrackId)> = app
        .engine
        .session()
        .tracks
        .iter()
        .filter(|x| crate::edit_window::in_edit_window(app.engine.session(), x))
        .enumerate()
        .map(|(i, t)| (i, t.id))
        .collect();

    for (_, tid) in &visible_tracks {
        app.edit_layout.rows.push((tid.0, [full.min.x, y, tl.max.x, y + row_h]));
        y += row_h + 1.0;
        content_h += row_h + 1.0;
    }
    app.edit_layout.content_h = content_h;

    let clip_rect = Rect::from_min_max(pos2(full.min.x, tl.min.y), pos2(full.max.x, tl.max.y));
    for (idx, tid) in visible_tracks {
        let Some((_, r)) = app.edit_layout.rows.iter().find(|(id, _)| *id == tid.0).copied() else { continue };
        let row = crate::edit_window::rect(r);
        if row.max.y >= clip_rect.min.y && row.min.y <= clip_rect.max.y {
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(row));
            child.set_clip_rect(clip_rect);
            draw_mobile_track_row(app, &mut child, idx, tid, row, tl);
        }
    }

    // Empty space below tracks: tap `+ Add Track`
    if y < tl.max.y {
        let empty_hdr = Rect::from_min_max(pos2(full.min.x, y.max(tl.min.y)), pos2(full.min.x + hw, tl.max.y));
        let add_btn = Rect::from_min_size(pos2(empty_hdr.min.x + 8.0, empty_hdr.min.y + 6.0), vec2(hw - 16.0, 26.0));
        if add_btn.max.y <= empty_hdr.max.y {
            let resp = ui.interact(add_btn, ui.id().with("mob_add_track_below"), Sense::click());
            ui.painter().rect_filled(
                add_btn,
                4.0,
                if resp.hovered() { Color32::from_rgb(46, 56, 68) } else { Color32::from_rgb(34, 41, 49) },
            );
            ui.painter().text(add_btn.center(), Align2::CENTER_CENTER, "+ ADD TRACK", bold(10.0), Color32::from_rgb(190, 202, 214));
            add_track_popup(app, &resp);
        }
    }

    // Playhead, selection highlight, and gesture overlay
    let mut tl_child = ui.new_child(egui::UiBuilder::new().max_rect(tl));
    tl_child.set_clip_rect(tl);
    crate::edit_window::overlay(app, &mut tl_child, tl, full);

    // Bottom horizontal scrollbar
    let sb = Rect::from_min_max(pos2(tl.min.x, tl.max.y), pos2(tl.max.x, full.max.y));
    crate::edit_window::hscrollbar(app, ui, sb, tl);
}

/// Top-left sub-header above track headers (`<->` Inspector toggle, `↕` Row height, `->|` Add Track).
fn draw_track_subheader(app: &mut SoundApp, ui: &mut Ui, rect: Rect) {
    let bg = Color32::from_rgb(35, 42, 49);
    let border = Color32::from_rgb(20, 24, 29);
    let fg = Color32::from_rgb(196, 206, 216);
    let sky = Color32::from_rgb(62, 166, 235);

    ui.painter().rect_filled(rect, 0.0, bg);
    ui.painter().hline(rect.x_range(), rect.max.y - 0.5, Stroke::new(1.0, border));
    ui.painter().vline(rect.max.x - 0.5, rect.y_range(), Stroke::new(1.0, border));

    let w3 = rect.width() / 3.0;
    // 1. `<->` Toggle Inspector Drawer
    let r0 = Rect::from_min_size(rect.min, vec2(w3, rect.height()));
    let resp0 = ui.interact(r0, ui.id().with("subhdr_insp"), Sense::click());
    if resp0.clicked() {
        app.ui.mobile_inspector_open = !app.ui.mobile_inspector_open;
    }
    ui.painter().text(
        r0.center(),
        Align2::CENTER_CENTER,
        "↔",
        bold(13.0),
        if app.ui.mobile_inspector_open { sky } else { fg },
    );
    resp0.on_hover_text("Expand / Collapse Track Inspector");

    // 2. `↕` Cycle Row Height (Compact 30px / Medium 44px / Tall 64px)
    let r1 = Rect::from_min_size(pos2(rect.min.x + w3, rect.min.y), vec2(w3, rect.height()));
    let resp1 = ui.interact(r1, ui.id().with("subhdr_height"), Sense::click());
    if resp1.clicked() {
        app.ui.mobile_row_height = if app.ui.mobile_row_height < 36.0 {
            46.0
        } else if app.ui.mobile_row_height < 54.0 {
            66.0
        } else {
            30.0
        };
    }
    ui.painter().text(r1.center(), Align2::CENTER_CENTER, "↕", bold(13.0), fg);
    resp1.on_hover_text("Cycle Track Row Height (Compact / Medium / Large)");

    // 3. `->|` / `+` Add Track popup
    let r2 = Rect::from_min_size(pos2(rect.min.x + w3 * 2.0, rect.min.y), vec2(w3, rect.height()));
    let resp2 = ui.interact(r2, ui.id().with("subhdr_add"), Sense::click());
    ui.painter().text(r2.center(), Align2::CENTER_CENTER, "+ ⇥", bold(11.5), fg);
    add_track_popup(app, &resp2);
    resp2.on_hover_text("Add or Manage Tracks");
}

fn add_track_popup(app: &mut SoundApp, resp: &egui::Response) {
    egui::Popup::from_toggle_button_response(resp)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(175.0);
            ui.label(egui::RichText::new("ADD TRACK").strong());
            if ui.button("+ Audio Track (Stereo)").clicked() {
                let _ = app.run("track.new", json!({"count": 1, "kind": "audio", "format": "stereo"}));
                ui.close();
            }
            if ui.button("+ Audio Track (Mono)").clicked() {
                let _ = app.run("track.new", json!({"count": 1, "kind": "audio", "format": "mono"}));
                ui.close();
            }
            if ui.button("+ Instrument / MIDI Track").clicked() {
                let _ = app.run("track.new", json!({"count": 1, "kind": "instrument", "format": "stereo"}));
                ui.close();
            }
            if ui.button("+ Aux / Bus Track").clicked() {
                let _ = app.run("track.new", json!({"count": 1, "kind": "aux", "format": "stereo"}));
                ui.close();
            }
            if ui.button("+ Master Fader").clicked() {
                let _ = app.run("track.new", json!({"count": 1, "kind": "master", "format": "stereo"}));
                ui.close();
            }
            ui.separator();
            if ui.button("Track Dialog…").clicked() {
                let _ = app.dialogs.open_for_command(&app.engine, "track.new");
                ui.close();
            }
        });
}

/// Clean Cubasis 3–style Bar & Beat Ruler with `L`/`R` loop markers and tap-to-seek.
fn draw_cubasis_ruler(app: &mut SoundApp, ui: &mut Ui, ruler: Rect, tl: Rect) {
    let bg = Color32::from_rgb(34, 41, 49);
    let border = Color32::from_rgb(20, 24, 29);
    let tick_col = Color32::from_rgb(90, 102, 114);
    let text_col = Color32::from_rgb(212, 220, 228);
    let sky = Color32::from_rgb(62, 166, 235);

    ui.painter().rect_filled(ruler, 0.0, bg);
    ui.painter().hline(ruler.x_range(), ruler.max.y - 0.5, Stroke::new(1.0, border));

    let s = app.engine.session();
    let sr = s.sample_rate;
    let spp = s.edit.zoom.samples_per_px.max(0.01);
    let view_start = crate::edit_window::sample_at(s, tl, tl.min.x).max(0);
    let view_end = crate::edit_window::sample_at(s, tl, tl.max.x).max(view_start + 1);

    let bar_grid = GridValue::Note { value: NoteValue::Bar, dotted: false, triplet: false };
    let bar_px = bar_grid.step_samples(view_start, sr, &s.tempo, s.frame_rate) as f64 / spp;
    let every = [1i64, 2, 4, 8, 16, 32, 64, 128].into_iter().find(|n| bar_px * *n as f64 >= 48.0).unwrap_or(256);
    let first = s.tempo.bar_beat_at(view_start, sr).bar;
    let mut bar = first - (first - 1).rem_euclid(every);
    let mut guard = 0;

    while guard < 400 {
        let at = s.tempo.samples_at_bar_beat(soundcraft_time::BarBeat { bar, beat: 1, tick: 0 }, sr);
        if at > view_end {
            break;
        }
        let x = crate::edit_window::x_of(s, tl, at);
        if x >= ruler.min.x && x <= ruler.max.x {
            ui.painter().vline(x, (ruler.min.y + 4.0)..=ruler.max.y, Stroke::new(1.0, tick_col));
            ui.painter().text(
                pos2(x + 4.0, ruler.center().y),
                Align2::LEFT_CENTER,
                format!("{bar}"),
                bold(11.5),
                text_col,
            );
        }
        if bar_px >= 28.0 {
            let m = s.tempo.meter_at_tick(s.tempo.samples_to_ticks(at, sr));
            for beat in 2..=i64::from(m.numerator) {
                let bat = s.tempo.samples_at_bar_beat(soundcraft_time::BarBeat { bar, beat, tick: 0 }, sr);
                let bx = crate::edit_window::x_of(s, tl, bat);
                if bx >= ruler.min.x && bx <= ruler.max.x {
                    ui.painter().vline(bx, (ruler.max.y - 6.0)..=ruler.max.y, Stroke::new(1.0, Color32::from_rgb(64, 74, 84)));
                }
            }
        }
        bar += every;
        guard += 1;
    }

    // Draw L / R selection or loop locators like Cubasis 3
    let sel = s.edit.selection;
    let (loc_start, loc_end) = if !sel.is_empty() {
        (sel.start, sel.end)
    } else {
        // Default visual L / R markers spanning the first 8 bars when no range is selected
        let b1 = s.tempo.samples_at_bar_beat(soundcraft_time::BarBeat { bar: 1, beat: 1, tick: 0 }, sr);
        let b9 = s.tempo.samples_at_bar_beat(soundcraft_time::BarBeat { bar: 9, beat: 1, tick: 0 }, sr);
        (b1, b9)
    };
    let lx = crate::edit_window::x_of(s, tl, loc_start).clamp(ruler.min.x, ruler.max.x);
    let rx = crate::edit_window::x_of(s, tl, loc_end).clamp(ruler.min.x, ruler.max.x);
    if !sel.is_empty() {
        let band = Rect::from_min_max(pos2(lx, ruler.min.y), pos2(rx, ruler.max.y));
        ui.painter().rect_filled(band, 0.0, Color32::from_rgba_unmultiplied(62, 166, 235, 38));
    }
    let flag_col = if s.edit.loop_playback { sky } else { Color32::from_rgb(122, 134, 146) };
    let l_flag = Rect::from_min_size(pos2(lx, ruler.min.y + 1.0), vec2(13.0, 12.0));
    ui.painter().rect_filled(l_flag, 2.0, flag_col);
    ui.painter().text(l_flag.center(), Align2::CENTER_CENTER, "L", bold(8.5), Color32::from_rgb(16, 20, 24));

    if rx > lx + 20.0 {
        let r_flag = Rect::from_min_size(pos2(rx - 13.0, ruler.min.y + 1.0), vec2(13.0, 12.0));
        ui.painter().rect_filled(r_flag, 2.0, flag_col);
        ui.painter().text(r_flag.center(), Align2::CENTER_CENTER, "R", bold(8.5), Color32::from_rgb(16, 20, 24));
    }

    // Playhead line in ruler
    let play_x = crate::edit_window::x_of(s, tl, app.position());
    if play_x >= ruler.min.x && play_x <= ruler.max.x {
        ui.painter().vline(play_x, ruler.y_range(), Stroke::new(1.5, Color32::WHITE));
    }

    // Tap / drag on ruler to locate playhead
    let resp = ui.interact(ruler, ui.id().with("mob_ruler_seek"), Sense::click_and_drag());
    if (resp.clicked() || resp.dragged())
        && let Some(pos) = resp.interact_pointer_pos()
    {
        let at = crate::edit_window::sample_at(app.engine.session(), tl, pos.x).max(0);
        let _ = app.run("transport.locate", json!({"at": at}));
    }
}

/// Draws a single Track Row: Cubasis numbered color badge + track name plate + full-bleed clips.
fn draw_mobile_track_row(app: &mut SoundApp, ui: &mut Ui, index: usize, id: TrackId, row: Rect, tl: Rect) {
    let s = app.engine.session_arc();
    let Some(track) = s.track(id) else { return };
    let selected = s.edit.selected_tracks.contains(&id);
    let badge_col = cubasis_track_color(index);

    let hdr = Rect::from_min_max(row.min, pos2(tl.min.x, row.max.y));
    let badge_r = Rect::from_min_max(hdr.min, pos2(hdr.min.x + NUM_BADGE_W, hdr.max.y));
    let plate_r = Rect::from_min_max(pos2(badge_r.max.x, hdr.min.y), hdr.max);

    // 1. Left colored number badge (`1`, `2`, `3`, ...)
    ui.painter().rect_filled(badge_r, 0.0, badge_col);
    ui.painter().text(
        badge_r.center(),
        Align2::CENTER_CENTER,
        format!("{}", index + 1),
        bold(11.5),
        Color32::from_rgb(22, 26, 32),
    );

    // 2. Right Track Name Plate (silver-white `#dce3ea` when selected, dark slate `#262d35` otherwise)
    let plate_bg = if selected {
        Color32::from_rgb(220, 227, 234)
    } else {
        Color32::from_rgb(38, 45, 53)
    };
    let plate_fg = if selected {
        Color32::from_rgb(20, 24, 29)
    } else {
        Color32::from_rgb(214, 222, 230)
    };
    ui.painter().rect_filled(plate_r, 0.0, plate_bg);

    // Subtle M / S / R mini pills on the right side of the name plate if active
    let mut name_max_x = plate_r.max.x - 4.0;
    if track.mixer.record_arm {
        let pill = Rect::from_min_size(pos2(name_max_x - 14.0, plate_r.center().y - 7.0), vec2(13.0, 14.0));
        ui.painter().rect_filled(pill, 2.0, Color32::from_rgb(239, 68, 68));
        ui.painter().text(pill.center(), Align2::CENTER_CENTER, "R", bold(8.5), Color32::WHITE);
        name_max_x -= 16.0;
    }
    if track.mixer.solo {
        let pill = Rect::from_min_size(pos2(name_max_x - 14.0, plate_r.center().y - 7.0), vec2(13.0, 14.0));
        ui.painter().rect_filled(pill, 2.0, Color32::from_rgb(234, 179, 8));
        ui.painter().text(pill.center(), Align2::CENTER_CENTER, "S", bold(8.5), Color32::from_rgb(20, 24, 28));
        name_max_x -= 16.0;
    }
    if track.mixer.mute {
        let pill = Rect::from_min_size(pos2(name_max_x - 14.0, plate_r.center().y - 7.0), vec2(13.0, 14.0));
        ui.painter().rect_filled(pill, 2.0, Color32::from_rgb(59, 130, 246));
        ui.painter().text(pill.center(), Align2::CENTER_CENTER, "M", bold(8.5), Color32::WHITE);
        name_max_x -= 16.0;
    }

    let mut display_name = track.name.clone();
    soundcraft_model::truncate_chars(&mut display_name, 13);
    let name_clip = Rect::from_min_max(pos2(plate_r.min.x + 6.0, plate_r.min.y), pos2(name_max_x, plate_r.max.y));
    ui.painter().with_clip_rect(name_clip).text(
        pos2(plate_r.min.x + 7.0, plate_r.center().y),
        Align2::LEFT_CENTER,
        &display_name,
        bold(11.5),
        plate_fg,
    );

    // Row dividers
    let border = Color32::from_rgb(20, 24, 29);
    ui.painter().hline(row.x_range(), row.max.y, Stroke::new(1.0, border));
    ui.painter().vline(badge_r.max.x - 0.5, hdr.y_range(), Stroke::new(1.0, border));
    ui.painter().vline(hdr.max.x - 0.5, hdr.y_range(), Stroke::new(1.0, border));

    // Track header interaction (tap to select, double-tap to open Inspector drawer, context menu)
    let hdr_resp = ui.interact(hdr, ui.id().with(("mob_thdr", id.0)), Sense::click());
    if hdr_resp.clicked() {
        crate::edit_window::click_track(app, id, ui.input(|i| i.modifiers), |_, _| true);
    }
    if hdr_resp.double_clicked() {
        app.ui.mobile_inspector_open = !app.ui.mobile_inspector_open;
    }
    hdr_resp.context_menu(|ui| {
        crate::edit_window::track_context_menu(app, ui, id);
    });

    // 3. Timeline Lane & Full-Bleed Cubasis Clips
    let lane = crate::edit_window::lane_rect(track, row, tl);
    let lane_bg = if selected {
        Color32::from_rgb(33, 39, 47)
    } else if index % 2 == 0 {
        Color32::from_rgb(26, 31, 37)
    } else {
        Color32::from_rgb(29, 34, 41)
    };
    let painter = ui.painter().with_clip_rect(lane.intersect(ui.clip_rect()));
    painter.rect_filled(lane, 0.0, lane_bg);
    crate::edit_window::grid_lines(&painter, &s, tl, lane);

    for (ci, clip) in track.clips().iter().enumerate() {
        draw_cubasis_clip(app, &painter, &s, index, ci, track, clip, tl, lane, badge_col);
    }
    crate::edit_window::draw_live_take(app, &painter, &s, track, tl, lane);
    if let Some(p) = AutoParam::parse(&track.view) {
        crate::edit_window::draw_automation(&painter, &s, track, &p, tl, lane);
    }
    crate::edit_window::lane_interaction(app, ui, track, lane, tl);
}

/// Draws a clip in the Cubasis 3 mobile style: vibrant full-bleed fill, dark left accent bar,
/// subtle waveform / MIDI preview, and bold white label (`01_Kick`, `02_Snare`, etc.).
#[allow(clippy::too_many_arguments)]
fn draw_cubasis_clip(
    app: &SoundApp,
    painter: &egui::Painter,
    s: &soundcraft_model::Session,
    track_idx: usize,
    clip_idx: usize,
    track: &Track,
    clip: &soundcraft_model::Clip,
    tl: Rect,
    lane: Rect,
    track_col: Color32,
) {
    let (mut start, mut end) = (clip.start, clip.end());
    if let Some(g) = &app.gesture {
        match g {
            crate::Gesture::MoveClips { clips, delta, .. } if clips.contains(&clip.id) => {
                start = (start + *delta).max(0);
                end = start + clip.length;
            }
            crate::Gesture::TrimStart { clip: id, to, .. } if *id == clip.id => start = *to,
            crate::Gesture::TrimEnd { clip: id, to, .. } if *id == clip.id => end = *to,
            _ => {}
        }
    }
    let x0 = crate::edit_window::x_of(s, tl, start);
    let x1 = crate::edit_window::x_of(s, tl, end);
    if x1 < lane.min.x || x0 > lane.max.x {
        return;
    }
    let r = Rect::from_min_max(pos2(x0.max(lane.min.x), lane.min.y + 1.0), pos2(x1.min(lane.max.x), lane.max.y - 1.0));
    if r.width() < 1.5 {
        return;
    }

    let sel = s.edit.selected_clips.contains(&clip.id);
    let fill = if clip.muted {
        Color32::from_rgb(70, 78, 88)
    } else if sel {
        track_col.lerp_to_gamma(Color32::WHITE, 0.22)
    } else {
        track_col
    };

    // Main vibrant clip block
    painter.rect_filled(r, CornerRadius::same(2), fill);

    // Subtle waveform or MIDI note bars inside the clip
    let inner = r.shrink2(vec2(2.0, 2.0)).intersect(lane);
    if inner.width() > 2.0 && inner.height() > 2.0 {
        match &clip.content {
            ClipContent::Audio { source, .. } => {
                if let Some(audio) = s.pool.get(*source) {
                    crate::waveform::draw(
                        &painter.with_clip_rect(inner),
                        s,
                        tl,
                        clip,
                        audio,
                        track.channels(),
                        r,
                        Color32::from_rgba_unmultiplied(18, 22, 28, 70),
                    );
                }
            }
            ClipContent::Midi { sequence } => {
                if !sequence.notes.is_empty() {
                    let (lo, hi) = sequence.notes.iter().fold((127u8, 0u8), |(lo, hi), n| (lo.min(n.pitch), hi.max(n.pitch)));
                    let (lo, hi) = if lo > hi { (48, 72) } else { (lo.saturating_sub(2), hi.saturating_add(2)) };
                    let span = f32::from(hi.saturating_sub(lo).max(1));
                    let base = s.tempo.samples_to_ticks(start, s.sample_rate);
                    let p = painter.with_clip_rect(inner);
                    for n in &sequence.notes {
                        let a = s.tempo.tick_to_samples(base + n.start, s.sample_rate);
                        let b = s.tempo.tick_to_samples(base + n.start + n.length, s.sample_rate).min(end);
                        let nx0 = crate::edit_window::x_of(s, tl, a).max(inner.min.x);
                        let nx1 = crate::edit_window::x_of(s, tl, b).min(inner.max.x);
                        if nx1 > nx0 {
                            let y = inner.max.y - 2.0 - (f32::from(n.pitch.saturating_sub(lo)) / span) * (inner.height() - 4.0);
                            p.rect_filled(
                                Rect::from_min_max(pos2(nx0, y - 1.2), pos2(nx1.max(nx0 + 2.0), y + 1.2)),
                                1.0,
                                Color32::from_rgba_unmultiplied(18, 22, 28, 120),
                            );
                        }
                    }
                }
            }
            ClipContent::Video { .. } => {}
        }
    }

    // Left dark separator bar & border (Cubasis clip boundary style)
    let border_col = if sel { Color32::WHITE } else { Color32::from_rgb(20, 24, 30) };
    painter.rect_stroke(r, CornerRadius::same(2), Stroke::new(if sel { 1.5 } else { 1.0 }, border_col), StrokeKind::Inside);

    // Bold clip title ("01_Kick", "02_Snare" style matching Cubasis 3)
    if r.width() > 24.0 {
        let clean_track = track.name.replace(' ', "");
        let label = if clip.name.is_empty() || clip.name == track.name || clip_idx == 0 {
            format!("{:02}_{}", track_idx + 1, clean_track)
        } else {
            format!("{:02}_{}", track_idx + 1, clip.name)
        };
        let text_pos = pos2((x0 + 6.0).max(lane.min.x + 6.0), r.center().y);
        let text_clip = r.shrink2(vec2(3.0, 1.0));
        painter.with_clip_rect(text_clip).text(
            pos2(text_pos.x + 0.8, text_pos.y + 0.8),
            Align2::LEFT_CENTER,
            &label,
            bold(11.0),
            Color32::from_rgba_unmultiplied(12, 16, 20, 185),
        );
        painter.with_clip_rect(text_clip).text(
            text_pos,
            Align2::LEFT_CENTER,
            &label,
            bold(11.0),
            Color32::WHITE,
        );
    }
}

/// Touch-friendly Mixer view when the user taps the Mixer icon in the top bar.
fn mobile_mixer_view(app: &mut SoundApp, ui: &mut Ui, rect: Rect) {
    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(24, 28, 33));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(4.0)));
    crate::mix_window::content(app, &mut child);
}

/// Formats `Samples` as a clean `Bar.Beat.Sub` string (`1.1.1`, `45.4.1`) matching Cubasis 3.
fn format_bars_beats_compact(s: &soundcraft_model::Session, samples: Samples) -> String {
    let bb = s.tempo.bar_beat_at(samples.max(0), s.sample_rate);
    let sub = ((bb.tick / 240).clamp(0, 3)) + 1;
    format!("{}.{}.{}", bb.bar, bb.beat, sub)
}

fn grid_label(g: GridValue) -> String {
    let l = g.label();
    if l == "1 bar" {
        "1/1".to_string()
    } else if let Some(prefix) = l.strip_suffix(" note") {
        prefix.to_uppercase()
    } else {
        l.to_uppercase()
    }
}

fn top_mode_button(ui: &mut Ui, rect: Rect, active: bool, accent: Color32, tooltip: &str) -> egui::Response {
    let resp = ui.interact(rect, ui.id().with(("top_btn", rect.min.x as i32, rect.min.y as i32)), Sense::click());
    if active {
        ui.painter().rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 42));
        ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, accent), StrokeKind::Inside);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(50, 58, 68));
    }
    resp.on_hover_text(tooltip)
}

// ============================================================================
// Procedural Vector Icons for Top Bar, Sub-Bar, and Inspector Rail
// ============================================================================

fn draw_piano_keys_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    let outer = Rect::from_center_size(c, vec2(20.0, 15.0));
    p.rect_stroke(outer, 2.0, Stroke::new(1.4, col), StrokeKind::Inside);
    p.vline(outer.min.x + outer.width() * 0.33, outer.y_range(), Stroke::new(1.0, col));
    p.vline(outer.min.x + outer.width() * 0.66, outer.y_range(), Stroke::new(1.0, col));
    let bk1 = Rect::from_min_size(pos2(outer.min.x + 4.5, outer.min.y), vec2(3.5, 8.5));
    let bk2 = Rect::from_min_size(pos2(outer.min.x + 11.5, outer.min.y), vec2(3.5, 8.5));
    p.rect_filled(bk1, 0.5, col);
    p.rect_filled(bk2, 0.5, col);
}

fn draw_faders_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    for (dx, knob_dy) in [(-6.0, 3.0), (0.0, -3.0), (6.0, 1.5)] {
        let x = c.x + dx;
        p.vline(x, (c.y - 7.5)..=(c.y + 7.5), Stroke::new(1.4, col));
        let knob = Rect::from_center_size(pos2(x, c.y + knob_dy), vec2(4.2, 3.2));
        p.rect_filled(knob, 1.0, col);
    }
}

fn draw_transport_prev(p: &egui::Painter, c: Pos2, col: Color32) {
    p.vline(c.x - 6.0, (c.y - 6.5)..=(c.y + 6.5), Stroke::new(2.0, col));
    p.add(egui::Shape::convex_polygon(
        vec![pos2(c.x - 4.0, c.y), pos2(c.x + 6.0, c.y - 6.5), pos2(c.x + 6.0, c.y + 6.5)],
        col,
        Stroke::NONE,
    ));
}

fn draw_transport_next(p: &egui::Painter, c: Pos2, col: Color32) {
    p.add(egui::Shape::convex_polygon(
        vec![pos2(c.x - 6.0, c.y - 6.5), pos2(c.x + 4.0, c.y), pos2(c.x - 6.0, c.y + 6.5)],
        col,
        Stroke::NONE,
    ));
    p.vline(c.x + 6.0, (c.y - 6.5)..=(c.y + 6.5), Stroke::new(2.0, col));
}

fn draw_transport_loop(p: &egui::Painter, c: Pos2, col: Color32) {
    let r = Rect::from_center_size(c, vec2(16.0, 11.0));
    p.rect_stroke(r, 4.0, Stroke::new(1.8, col), StrokeKind::Inside);
    p.add(egui::Shape::convex_polygon(
        vec![pos2(r.max.x - 2.0, r.min.y - 2.5), pos2(r.max.x + 2.5, r.min.y + 0.5), pos2(r.max.x - 2.0, r.min.y + 3.5)],
        col,
        Stroke::NONE,
    ));
}

fn draw_transport_play(p: &egui::Painter, c: Pos2, col: Color32) {
    p.add(egui::Shape::convex_polygon(
        vec![pos2(c.x - 5.5, c.y - 7.5), pos2(c.x + 7.5, c.y), pos2(c.x - 5.5, c.y + 7.5)],
        col,
        Stroke::NONE,
    ));
}

fn draw_metronome_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    let pts = vec![pos2(c.x - 6.5, c.y + 7.0), pos2(c.x - 2.5, c.y - 7.0), pos2(c.x + 2.5, c.y - 7.0), pos2(c.x + 6.5, c.y + 7.0)];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.5, col)));
    p.line_segment([pos2(c.x, c.y + 5.0), pos2(c.x + 5.5, c.y - 5.5)], Stroke::new(1.6, col));
}

fn draw_gear_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    p.circle_stroke(c, 5.2, Stroke::new(1.8, col));
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let p0 = pos2(c.x + a.cos() * 5.0, c.y + a.sin() * 5.0);
        let p1 = pos2(c.x + a.cos() * 7.8, c.y + a.sin() * 7.8);
        p.line_segment([p0, p1], Stroke::new(1.8, col));
    }
}

fn draw_fx_bag_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    let body = Rect::from_center_size(pos2(c.x, c.y + 2.0), vec2(14.0, 11.0));
    p.rect_stroke(body, 2.0, Stroke::new(1.5, col), StrokeKind::Inside);
    p.line_segment([pos2(c.x - 3.5, body.min.y), pos2(c.x - 2.0, body.min.y - 4.0)], Stroke::new(1.4, col));
    p.line_segment([pos2(c.x - 2.0, body.min.y - 4.0), pos2(c.x + 2.0, body.min.y - 4.0)], Stroke::new(1.4, col));
    p.line_segment([pos2(c.x + 2.0, body.min.y - 4.0), pos2(c.x + 3.5, body.min.y)], Stroke::new(1.4, col));
}

#[derive(Clone, Copy)]
enum SubIcon {
    Select,
    Split,
    Glue,
    Erase,
    Draw,
    Mute,
    Transpose,
    Quantize,
    GridNote,
    Stretch,
    Undo,
    Redo,
    Copy,
    Paste,
    SnapGrid,
}

fn draw_sub_icon(p: &egui::Painter, c: Pos2, icon: SubIcon, col: Color32) {
    let s = Stroke::new(1.4, col);
    match icon {
        SubIcon::Select => {
            let r = Rect::from_center_size(c, vec2(13.0, 9.0));
            p.rect_stroke(r, 1.5, s, StrokeKind::Inside);
        }
        SubIcon::Split => {
            p.circle_stroke(pos2(c.x - 3.5, c.y + 3.0), 2.2, s);
            p.circle_stroke(pos2(c.x + 3.5, c.y + 3.0), 2.2, s);
            p.line_segment([pos2(c.x - 2.0, c.y + 1.5), pos2(c.x + 3.5, c.y - 4.5)], s);
            p.line_segment([pos2(c.x + 2.0, c.y + 1.5), pos2(c.x - 3.5, c.y - 4.5)], s);
        }
        SubIcon::Glue => {
            let r1 = Rect::from_center_size(pos2(c.x - 3.5, c.y), vec2(5.5, 8.0));
            let r2 = Rect::from_center_size(pos2(c.x + 3.5, c.y), vec2(5.5, 8.0));
            p.rect_stroke(r1, 1.0, s, StrokeKind::Inside);
            p.rect_stroke(r2, 1.0, s, StrokeKind::Inside);
            p.line_segment([pos2(c.x - 1.0, c.y), pos2(c.x + 1.0, c.y)], s);
        }
        SubIcon::Erase => {
            p.line_segment([pos2(c.x - 4.5, c.y - 4.5), pos2(c.x + 4.5, c.y + 4.5)], Stroke::new(1.8, col));
            p.line_segment([pos2(c.x + 4.5, c.y - 4.5), pos2(c.x - 4.5, c.y + 4.5)], Stroke::new(1.8, col));
        }
        SubIcon::Draw => {
            p.line_segment([pos2(c.x - 4.5, c.y + 4.5), pos2(c.x + 4.5, c.y - 4.5)], Stroke::new(2.0, col));
            p.circle_filled(pos2(c.x - 4.8, c.y + 4.8), 1.4, col);
        }
        SubIcon::Mute => {
            p.circle_stroke(c, 5.0, s);
            p.line_segment([pos2(c.x - 3.5, c.y + 3.5), pos2(c.x + 3.5, c.y - 3.5)], s);
        }
        SubIcon::Transpose => {
            p.text(c, Align2::CENTER_CENTER, "♪↕", bold(10.5), col);
        }
        SubIcon::Quantize => {
            p.text(c, Align2::CENTER_CENTER, "Q", bold(11.5), col);
        }
        SubIcon::GridNote => {
            p.circle_filled(pos2(c.x - 1.5, c.y + 3.0), 2.4, col);
            p.vline(c.x + 0.5, (c.y - 4.5)..=(c.y + 3.0), s);
            p.line_segment([pos2(c.x + 0.5, c.y - 4.5), pos2(c.x + 4.5, c.y - 2.5)], s);
        }
        SubIcon::Stretch => {
            p.line_segment([pos2(c.x - 6.0, c.y), pos2(c.x + 6.0, c.y)], s);
            p.line_segment([pos2(c.x - 6.0, c.y), pos2(c.x - 3.0, c.y - 3.0)], s);
            p.line_segment([pos2(c.x - 6.0, c.y), pos2(c.x - 3.0, c.y + 3.0)], s);
            p.line_segment([pos2(c.x + 6.0, c.y), pos2(c.x + 3.0, c.y - 3.0)], s);
            p.line_segment([pos2(c.x + 6.0, c.y), pos2(c.x + 3.0, c.y + 3.0)], s);
        }
        SubIcon::Undo => {
            p.line_segment([pos2(c.x + 5.0, c.y + 1.0), pos2(c.x - 4.0, c.y + 1.0)], s);
            p.line_segment([pos2(c.x - 4.0, c.y + 1.0), pos2(c.x - 1.0, c.y - 2.5)], s);
            p.line_segment([pos2(c.x - 4.0, c.y + 1.0), pos2(c.x - 1.0, c.y + 4.5)], s);
        }
        SubIcon::Redo => {
            p.line_segment([pos2(c.x - 5.0, c.y + 1.0), pos2(c.x + 4.0, c.y + 1.0)], s);
            p.line_segment([pos2(c.x + 4.0, c.y + 1.0), pos2(c.x + 1.0, c.y - 2.5)], s);
            p.line_segment([pos2(c.x + 4.0, c.y + 1.0), pos2(c.x + 1.0, c.y + 4.5)], s);
        }
        SubIcon::Copy => {
            let r1 = Rect::from_center_size(pos2(c.x - 1.5, c.y - 1.0), vec2(7.0, 8.0));
            let r2 = Rect::from_center_size(pos2(c.x + 2.0, c.y + 1.5), vec2(7.0, 8.0));
            p.rect_stroke(r1, 1.0, s, StrokeKind::Inside);
            p.rect_stroke(r2, 1.0, s, StrokeKind::Inside);
        }
        SubIcon::Paste => {
            let r = Rect::from_center_size(pos2(c.x, c.y + 1.0), vec2(9.0, 9.0));
            p.rect_stroke(r, 1.0, s, StrokeKind::Inside);
            p.hline((c.x - 2.5)..=(c.x + 2.5), r.min.y - 1.0, Stroke::new(2.0, col));
        }
        SubIcon::SnapGrid => {
            p.text(c, Align2::CENTER_CENTER, "#", bold(12.0), col);
        }
    }
}

#[derive(Clone, Copy)]
enum RailIcon {
    PlayArrow,
    Routing,
    Inserts,
    Sends,
    Automation,
    Notes,
    Fader,
    ColorDrop,
    Hamburger,
}

fn draw_rail_icon(p: &egui::Painter, c: Pos2, icon: RailIcon, col: Color32) {
    let s = Stroke::new(1.5, col);
    match icon {
        RailIcon::PlayArrow => {
            p.add(egui::Shape::convex_polygon(
                vec![pos2(c.x - 4.0, c.y - 5.5), pos2(c.x + 5.5, c.y), pos2(c.x - 4.0, c.y + 5.5)],
                col,
                Stroke::NONE,
            ));
        }
        RailIcon::Routing => {
            p.line_segment([pos2(c.x - 5.5, c.y - 2.5), pos2(c.x + 5.5, c.y - 2.5)], s);
            p.line_segment([pos2(c.x + 2.5, c.y - 5.0), pos2(c.x + 5.5, c.y - 2.5)], s);
            p.line_segment([pos2(c.x - 5.5, c.y + 2.5), pos2(c.x + 5.5, c.y + 2.5)], s);
            p.line_segment([pos2(c.x - 2.5, c.y + 5.0), pos2(c.x - 5.5, c.y + 2.5)], s);
        }
        RailIcon::Inserts => {
            p.text(c, Align2::CENTER_CENTER, "FX", bold(10.0), col);
        }
        RailIcon::Sends => {
            p.circle_stroke(pos2(c.x - 3.0, c.y), 2.5, s);
            p.line_segment([pos2(c.x - 0.5, c.y), pos2(c.x + 6.0, c.y - 3.5)], s);
            p.line_segment([pos2(c.x - 0.5, c.y), pos2(c.x + 6.0, c.y + 3.5)], s);
        }
        RailIcon::Automation => {
            p.text(c, Align2::CENTER_CENTER, "R W", bold(9.0), col);
        }
        RailIcon::Notes => {
            let r = Rect::from_center_size(c, vec2(11.0, 11.0));
            p.rect_stroke(r, 1.5, s, StrokeKind::Inside);
            p.line_segment([pos2(c.x - 2.5, c.y + 2.5), pos2(c.x + 3.5, c.y - 3.5)], s);
        }
        RailIcon::Fader => {
            p.vline(c.x - 3.5, (c.y - 5.5)..=(c.y + 5.5), s);
            p.vline(c.x + 3.5, (c.y - 5.5)..=(c.y + 5.5), s);
            p.rect_filled(Rect::from_center_size(pos2(c.x - 3.5, c.y - 2.0), vec2(4.5, 3.0)), 1.0, col);
            p.rect_filled(Rect::from_center_size(pos2(c.x + 3.5, c.y + 2.0), vec2(4.5, 3.0)), 1.0, col);
        }
        RailIcon::ColorDrop => {
            p.circle_filled(pos2(c.x, c.y + 1.5), 4.2, col);
            p.add(egui::Shape::convex_polygon(
                vec![pos2(c.x, c.y - 5.5), pos2(c.x - 3.8, c.y + 0.5), pos2(c.x + 3.8, c.y + 0.5)],
                col,
                Stroke::NONE,
            ));
        }
        RailIcon::Hamburger => {
            for dy in [-4.0, 0.0, 4.0] {
                p.hline((c.x - 6.0)..=(c.x + 6.0), c.y + dy, Stroke::new(1.8, col));
            }
        }
    }
}
