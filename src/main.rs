use eframe::egui;
use std::time::Instant;

#[derive(PartialEq)]
enum Direction {
    LeftToRight,
    RightToLeft,
}

struct Settings {
    distance: f32,
    wave_speed: f32,
    frequency: f32,
    wave_intensity: f32,
    wave_decay: f32,
    object_speed: f32,
    direction: Direction,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            distance: 2500.0,
            wave_speed: 331.5,
            frequency: 20.0,
            wave_intensity: 1.0,
            wave_decay: 3.0,
            object_speed: 50.0,
            direction: Direction::LeftToRight,
        }
    }
}

struct Waves {
    distance_from_a_meters: f32,
    emission_time: Instant,
}

struct Object {
    progress: f32,
    moving: bool,
    finished: bool,
    start_time: Instant,
    last_wave_time: Option<Instant>,
    waves: Vec<Waves>,
    total_emitted: u64,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            progress: 0.0,
            moving: false,
            finished: false,
            start_time: Instant::now(),
            last_wave_time: None,
            waves: Vec::new(),
            total_emitted: 0,
        }
    }
}

impl Object {
    /// resets the state by moving [`Object`] back to Point A
    #[inline(always)]
    fn reset(&mut self) {
        self.progress = 0.0;
        self.moving = false;
        self.finished = false;
        self.last_wave_time = None;
        self.waves.clear();
        self.total_emitted = 0;
    }

    /// sets the state for starting the animation
    #[inline(always)]
    fn start(&mut self) {
        self.start_time = Instant::now();
        self.last_wave_time = None;
        self.waves.clear();
        self.progress = 0.0;
        self.moving = true;
        self.finished = false;
        self.total_emitted = 0;
    }

    /// sets the state where Object has finished it's animation
    #[inline(always)]
    fn finished(&mut self) {
        self.progress = 1.0;
        self.moving = false;
        self.finished = true;
    }

    #[inline(always)]
    fn update(&mut self, settings: &Settings) {
        let total_time = settings.distance / settings.object_speed;
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.progress = (elapsed / total_time).clamp(0.0, 1.0);
        let wave_time_period = 1.0 / settings.frequency;
        let should_generate_wave = match self.last_wave_time {
            None => true,
            Some(last_wave_time) => last_wave_time.elapsed().as_secs_f32() >= wave_time_period,
        };
        if should_generate_wave {
            let now = Instant::now();
            let traveled = self.progress * settings.distance;
            let distance_from_a_meters = match settings.direction {
                Direction::LeftToRight => traveled,
                Direction::RightToLeft => settings.distance - traveled,
            };
            self.waves.push(Waves {
                distance_from_a_meters,
                emission_time: now,
            });
            self.last_wave_time = Some(now);
            self.total_emitted += 1;
        }
        if self.progress >= 1.0 {
            self.finished();
        }
    }

    #[inline(always)]
    fn prune_waves(&mut self, settings: &Settings) {
        let cutoff = settings.wave_decay * 5.0;
        self.waves
            .retain(|w| w.emission_time.elapsed().as_secs_f32() < cutoff);
    }
}

#[derive(Default)]
struct App {
    settings: Settings,
    object: Object,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let rect = ui.available_rect_before_wrap();
        let a = egui::Pos2 {
            x: rect.left() + 100.0,
            y: rect.center().y,
        };
        let b = egui::Pos2 {
            x: rect.right() - 100.0,
            y: rect.center().y,
        };

        let (start, end) = match self.settings.direction {
            Direction::LeftToRight => (a, b),
            Direction::RightToLeft => (b, a),
        };

        if self.object.moving {
            self.object.update(&self.settings);
        }
        self.object.prune_waves(&self.settings);
        if self.object.moving || !self.object.waves.is_empty() {
            ui.ctx().request_repaint();
        }

        let object_position = egui::Pos2 {
            x: start.x + (end.x - start.x) * self.object.progress,
            y: start.y + (end.y - start.y) * self.object.progress,
        };

        match self.settings.direction {
            Direction::LeftToRight => {
                ui.painter()
                    .circle_filled(a, 5.0, egui::Color32::LIGHT_GREEN);
                ui.painter().circle_filled(b, 5.0, egui::Color32::RED);
            }
            Direction::RightToLeft => {
                ui.painter().circle_filled(a, 5.0, egui::Color32::RED);
                ui.painter()
                    .circle_filled(b, 5.0, egui::Color32::LIGHT_GREEN);
            }
        };

        ui.painter().circle_filled(
            egui::Pos2 {
                x: object_position.x,
                y: object_position.y,
            },
            25.0,
            egui::Color32::WHITE,
        );

        let pixels_per_meter = (b.x - a.x).abs() / self.settings.distance;
        for wave in &self.object.waves {
            let elapsed_time = wave.emission_time.elapsed().as_secs_f32();
            let fade = (-elapsed_time / self.settings.wave_decay).exp();
            let alpha = (self.settings.wave_intensity * fade * 255.0) as u8;

            let wave_x = a.x + wave.distance_from_a_meters * pixels_per_meter;
            let radius_pixels = self.settings.wave_speed * elapsed_time * pixels_per_meter;

            ui.painter().circle_stroke(
                egui::Pos2 { x: wave_x, y: a.y },
                radius_pixels,
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(alpha)),
            );
        }

        ui.label(format!(
            "Object's position x: {:.2}, y: {:.2}",
            object_position.x, object_position.y
        ));
        ui.label(format!(
            "Time period: {:.4} s",
            1.0 / self.settings.frequency
        ));

        ui.label(format!("Waves generated: {}", self.object.total_emitted));
        ui.label(format!("Waves alive: {}", self.object.waves.len()));

        ui.horizontal(|ui| {
            ui.label("Direction:");
            let mut changed = false;
            changed |= ui
                .selectable_value(
                    &mut self.settings.direction,
                    Direction::LeftToRight,
                    "Left -> Right",
                )
                .changed();
            changed |= ui
                .selectable_value(
                    &mut self.settings.direction,
                    Direction::RightToLeft,
                    "Right -> Left",
                )
                .changed();
            if changed {
                self.object.reset();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("Distance: {:.2} m", self.settings.distance));
            let slider = ui.add(egui::Slider::new(
                &mut self.settings.distance,
                500.0..=5000.0,
            ));
            if slider.changed() {
                self.object.reset();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("Frequency: {:.2} Hz", self.settings.frequency));
            let slider = ui.add(egui::Slider::new(
                &mut self.settings.frequency,
                1.0..=1000.0,
            ));
            if slider.changed() {
                self.object.reset();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Object speed: {:.2} m/s",
                self.settings.object_speed
            ));
            let slider = ui.add(egui::Slider::new(
                &mut self.settings.object_speed,
                1.0..=1715.0,
            ));
            if slider.changed() {
                self.object.reset();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("Wave speed: {:.2} m/s", self.settings.wave_speed));
            let slider = ui.add(egui::Slider::new(
                &mut self.settings.wave_speed,
                1.0..=1715.0,
            ));
            if slider.changed() {
                self.object.reset();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Wave intensity: {:.2}",
                self.settings.wave_intensity
            ));
            ui.add(egui::Slider::new(
                &mut self.settings.wave_intensity,
                0.05..=1.0,
            ));
        });

        ui.horizontal(|ui| {
            ui.label(format!("Wave decay: {:.2} s", self.settings.wave_decay));
            ui.add(egui::Slider::new(&mut self.settings.wave_decay, 0.2..=10.0));
        });

        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.object.start();
                ui.ctx().request_repaint();
            }
            if ui.button("Reset").clicked() {
                self.object.reset();
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Doppler Effect",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
