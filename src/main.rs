use std::{fs::File, io::Write, sync::mpsc::Receiver, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use eframe::egui::{self, Vec2};
use egui_extras::{Column, TableBuilder};
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

fn main() {
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::F9);
    manager.register(hotkey).unwrap();

    let receiver = GlobalHotKeyEvent::receiver();

    let (tx,rx) = std::sync::mpsc::channel::<bool>();

    std::thread::spawn(move || loop{
        if let Ok(event) = receiver.try_recv() {
            if event.state == HotKeyState::Released{
                tx.send(true).unwrap();
            }
        }
        thread::sleep(Duration::from_millis(100));
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Davinci Resolve Markermaker", native_options, Box::new(|cc| Box::new(MarkerApp::new(cc, rx)))).unwrap();
}


#[derive(Debug)]
struct Marker{
    time: Duration,
    name: String
}

impl Marker{
    fn to_edl(&self, index: usize) -> String{
        let timestamp = format!("{}:0", to_timecode(self.time));

        format!("{index:0>3} 001 V C {timestamp}0 {timestamp}1 {timestamp}0 {timestamp}1\n |C:ResolveColorBlue |M:{} |D:1\n", self.name)
    }
}




fn to_timecode(time: Duration) -> String{
    let t = time.as_millis();

    let hour = t/(60*60*1000) + 1;
    let minute = t%(60*60*1000) / (60*1000);
    let second = t%(60*1000)/1000;



    format!("{hour:0>2}:{minute:0>2}:{second:0>2}")
}


struct MarkerApp {
    markers: Vec<Marker>,
    running: bool,
    start_instant: Instant,
    start_time: SystemTime,
    rx: Receiver<bool>
}

impl MarkerApp {
    fn new(cc: &eframe::CreationContext<'_>, rx: Receiver<bool>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let ctx = cc.egui_ctx.clone();


        thread::spawn(move ||{
            loop{                
                ctx.request_repaint();
                thread::sleep(Duration::from_secs(1));
            }

        });



        Self{
            markers: vec![Marker{ time: Duration::from_secs(70), name: "Mark1".to_string() }],
            running: false,
            start_instant: Instant::now(),
            start_time: SystemTime::now(),
            rx
        }
    }

    fn write_markers(&self){
        let mut file = File::create(format!("{}.edl", self.start_time.duration_since(UNIX_EPOCH).unwrap().as_secs())).unwrap();

        for (i, marker) in self.markers.iter().enumerate(){
            file.write_all(marker.to_edl(i + 1).as_bytes()).unwrap();
        }
    }


    fn add_marker(&mut self){
        self.markers.push(
            Marker{
                time: Instant::now() - self.start_instant,
                name: "Marker".to_string(),
            }
            );
        self.write_markers();

    }
}

impl eframe::App for MarkerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.rx.try_recv(){
            self.add_marker();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(if self.running{
                to_timecode(Instant::now() - self.start_instant)
            }else{
                "01:00:00".to_string()
            });

            ui.label("Shift+F9 to add marker. Edit names here later.");
            ui.label("Files are created in same folder as the .exe");
            ui.label("To import .edl file, right-click on the timeline in clip browser, go ");
            ui.label("Timelines -> Import -> Timeline Markers from EDL...");



            if ui.button(if self.running {"Stop"} else {"Start"}).clicked(){
                if self.running{
                }else{
                    self.markers.clear();
                    self.start_instant = Instant::now();
                    self.start_time = SystemTime::now();
                }
                self.running = !self.running;

            }

            ui.add_space(30.);

            if ui.add(egui::Button::new("Add Marker").min_size(Vec2::new(150., 50.))).clicked() && self.running{
                self.add_marker();
            }

            ui.add_space(30.);


            TableBuilder::new(ui)
                .striped(true)

                .column(Column::auto())
                .column(Column::auto().at_least(60.))
                .column(Column::auto().at_least(300.))
                .header(20., |mut header|{
                    header.col(|ui|{
                        ui.heading("id");
                    });
                    header.col(|ui|{
                        ui.heading("Time");
                    });
                    header.col(|ui|{
                        ui.heading("Name");
                    });
                })
            .body(|mut body|{
                let mut changed = false;
                for (i, marker) in &mut self.markers.iter_mut().enumerate(){
                    body.row(20.0, |mut row|{
                        row.col(|ui|{
                            ui.label(i.to_string());
                        });
                        row.col(|ui|{
                            ui.label(to_timecode(marker.time));
                        });
                        row.col(|ui|{
                            let res = ui.text_edit_singleline(&mut marker.name);

                            changed |= res.changed();
                        });
                    });

                }

                if changed{
                    self.write_markers();
                }
            });

        });


    }
}
