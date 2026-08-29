use eframe::egui;
use std::time::Instant;

// Constants
const DISTANCE: f32 = 500.0;
const SOUND_SPEED: f32 = 331.5;
const FREQUENCY: f32 = 10.0;

struct Object {
    x: f32,
    y: f32,

    speed: f32,

    // 0.0 -> Point A
    // 1.0 -> Point B
    progress: f32,

    moving: bool,
    finished: bool,

    start_time: Instant,

    /// the time at which the last wave was generated
    last_wave_time: Option<Instant>,

    waves: Vec<Waves>,
}

struct Waves {
    distance_from_a: f32,
    emission_time: Instant, // time the wave was generated
}

impl eframe::App for Object {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let rect = ui.available_rect_before_wrap();

        // Position of Point A
        let point_a = egui::Pos2 {
            x: rect.left() + 100.0,
            y: rect.center().y,
        };

        // Position of Point B
        let point_b = egui::Pos2 {
            x: rect.right() - 100.0,
            y: rect.center().y,
        };

        let pixels_per_meter = (point_b.x - point_a.x) / DISTANCE;

        if !self.moving && !self.finished {
            self.x = point_a.x;
            self.y = point_a.y;
        }

        if self.moving {
            let total_time = DISTANCE / self.speed;
            let elapsed = self.start_time.elapsed().as_secs_f32();
            self.progress = (elapsed / total_time).clamp(0.0, 1.0);

            // rate at which waves should be generated each second.
            let wave_time_period = 1.0 / FREQUENCY;

            // Check if enough time has elapsed to emit the next wave
            let should_generate_wave = match self.last_wave_time {
                None => true,
                Some(last_wave_time) => last_wave_time.elapsed().as_secs_f32() >= wave_time_period,
            };
            if should_generate_wave {
                let now = Instant::now();
                // convert Object's progress into meters
                let distance_from_a = self.progress * DISTANCE;
                self.waves.push(Waves {
                    distance_from_a,
                    emission_time: now,
                });
                self.last_wave_time = Some(now);
            }

            // Upon reaching Point B
            if self.progress >= 1.0 {
                self.progress = 1.0;

                self.moving = false;
                self.finished = true;

                self.waves.clear();
            }
            ui.ctx().request_repaint();
        }

        self.x = point_a.x + (point_b.x - point_a.x) * self.progress;

        self.y = point_a.y + (point_b.y - point_a.y) * self.progress;

        // Drawing Point A
        ui.painter()
            .circle_filled(point_a, 5.0, egui::Color32::GREEN);

        // Drawing Point B
        ui.painter().circle_filled(point_b, 5.0, egui::Color32::RED);

        // Drawing Object
        ui.painter().circle_filled(
            egui::Pos2 {
                x: self.x,
                y: self.y,
            },
            25.0,
            egui::Color32::WHITE,
        );

        // Drawing waves
        for wave in &self.waves {
            // time since the wave was first emitted
            let elapsed_time = wave.emission_time.elapsed().as_secs_f32();
            let wave_x = point_a.x + wave.distance_from_a * pixels_per_meter;

            // radius = Sound_speed * time_since_emission * pixels_per_meter
            let radius_meters = SOUND_SPEED * elapsed_time;
            let radius_pixels = radius_meters * pixels_per_meter;
            ui.painter().circle_stroke(
                egui::Pos2 {
                    x: wave_x,
                    y: point_a.y,
                },
                radius_pixels,
                egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
            );
        }

        // Logs
        ui.label(format!(
            "Object position x: {:.2}, y: {:.2}",
            self.x, self.y
        ));
        ui.label(format!("Distance: {:.2} m", DISTANCE));
        ui.label(format!("Speed: {:.2} m/s", self.speed));
        ui.label(format!("Frequency: {:.2} Hz", FREQUENCY));
        ui.label(format!("Time period: {:.2} s", 1.0 / FREQUENCY));
        ui.label(format!("Number of waves generated: {}", self.waves.len()));
        ui.label(format!("Progress: {:.2}%", self.progress * 100.0));

        // Buttons
        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.start_time = Instant::now();
                self.last_wave_time = None;
                self.waves.clear();

                self.progress = 0.0;

                self.moving = true;
                self.finished = false;

                ui.ctx().request_repaint();
            }

            if ui.button("Reset").clicked() {
                self.progress = 0.0;

                self.moving = false;
                self.finished = false;

                self.last_wave_time = None;
                self.waves.clear();
            }
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Window",
        native_options,
        Box::new(|_| {
            Ok(Box::new(Object {
                x: 0.0,
                y: 0.0,

                speed: 100.0,

                progress: 0.0,

                moving: false,
                finished: false,

                start_time: Instant::now(),

                last_wave_time: None,

                waves: Vec::new(),
            }))
        }),
    )
}
