use egui::{vec2, Align, Layout, RichText};

use windows_sys::w;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW;

use crate::app::backend::TemplateApp;
use crate::app::networking::{ipv4_get, ipv6_get};
use crate::app::server;

impl TemplateApp {
    pub fn state_server(&mut self, _frame: &mut eframe::Frame, ctx: &egui::Context) {
        //settings
        egui::TopBottomPanel::top("srvr_settings").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.allocate_ui(vec2(300., 40.), |ui| {
                    if ui
                        .add(egui::widgets::ImageButton::new(egui::include_image!(
                            "../../../icons/settings.png"
                        )))
                        .clicked()
                    {
                        self.settings_window = !self.settings_window;
                    };
                });
                ui.allocate_ui(vec2(300., 40.), |ui| {
                    if ui
                        .add(egui::widgets::ImageButton::new(egui::include_image!(
                            "../../../icons/logout.png"
                        )))
                        .clicked()
                    {
                        self.server_mode = false;
                    };
                })
                .response
                .on_hover_text("Logout");
            });
            ui.allocate_space(vec2(ui.available_width(), 5.));
        });
        //main
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.label(RichText::from("Server mode").strong().size(30.));
                ui.label(RichText::from("Message stream").size(20.));
                if !self.server_has_started {
                    ui.label(RichText::from("Server setup").size(30.).strong());
                    ui.separator();
                    ui.label(RichText::from("Open on port").size(20.));
                    ui.text_edit_singleline(&mut self.open_on_port);
                    let temp_open_on_port = &self.open_on_port;
                    ui.checkbox(&mut self.ipv4_mode, "Internet protocol (IP) v4 mode");
                    if ui.button("Start").clicked() {
                        let temp_tx = self.stx.clone();
                        let server_pw = self.server_password.clone();
                        let ip_v4 = self.ipv4_mode;
                        self.server_has_started = match temp_open_on_port.parse::<i32>() {
                            Ok(port) => {
                                tokio::spawn(async move {
                                    match server::server_main(port.to_string(), server_pw, ip_v4)
                                        .await
                                    {
                                        Ok(ok) => {
                                            dbg!(&ok);

                                            let mut concatenated_string = String::new();

                                            for s in &ok {
                                                concatenated_string.push_str(s);
                                            }

                                            match temp_tx.send(ok.join(&concatenated_string)) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    println!("ln 214 {}", err)
                                                }
                                            };
                                        }
                                        Err(err) => {
                                            println!("ln 208 {:?}", err);
                                        }
                                    };
                                });
                                true
                            }
                            Err(_) => {
                                unsafe {
                                    MessageBoxW(0, w!("Error"), w!("Enter a valid port!"), 0);
                                }
                                false
                            }
                        };
                    }
                    ui.separator();
                    ui.checkbox(&mut self.server_req_password, "Server requires password");
                    if self.server_req_password {
                        ui.text_edit_singleline(&mut self.server_password);
                    }
                } else {
                    if self.public_ip.is_empty() {
                        let tx = self.dtx.clone();
                        std::thread::spawn(move || {
                            let combined_ips = ipv4_get()
                                .unwrap_or_else(|_| "Couldnt connect to the internet".to_string())
                                + ";"
                                + &ipv6_get().unwrap_or_else(|_| {
                                    "Couldnt connect to the internet".to_string()
                                });
                            tx.send(combined_ips)
                        });
                        match self.drx.recv() {
                            Ok(ok) => self.public_ip = ok,
                            Err(err) => {
                                eprintln!("{}", err)
                            }
                        }
                    }
                    let pub_ip: Vec<&str> = self.public_ip.rsplit(';').collect();
                    if self.ipv4_mode {
                        ui.label(RichText::from("Public ipV4 address : ").size(20.));
                        ui.text_edit_singleline(&mut pub_ip[1].trim().to_string());
                    } else {
                        ui.label(RichText::from("Public ipV6 address : ").size(20.));
                        ui.text_edit_singleline(&mut pub_ip[0].trim().to_string());
                    }
                    if self.server_req_password && !self.server_password.is_empty() {
                        ui.label(RichText::from(format!(
                            "Password : {}",
                            self.server_password
                        )));
                    }
                    ui.label(RichText::from("Port").size(15.).strong());
                    ui.label(RichText::from(self.open_on_port.clone()).size(15.));
                }
            });
        });
    }
}
