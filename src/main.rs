#![windows_subsystem = "windows"]
use chrono::Local;
use eframe::egui;
use fs_extra::dir::{copy, CopyOptions};
use rfd::FileDialog;
use std::io;
use std::path::Path;
use chrono::Timelike;

// 定义时间选择的枚举类型
#[derive(PartialEq)]
enum TimeSelection {
    Second,
    Minute,
    Hour,
    Day,
}

// 为 TimeSelection 实现方法
impl TimeSelection {
    fn value(&self) -> i32 {
        match self {
            TimeSelection::Second => 60,
            TimeSelection::Minute => 60,
            TimeSelection::Hour => 24,
            TimeSelection::Day => 30,
        }
    }
}

impl Default for TimeSelection {
    fn default() -> Self {
        TimeSelection::Second
    }
}

// 定义应用程序构体
#[derive(Default)]
struct MyApp {
    time: i32,
    last_time: i32,
    current_selection: TimeSelection,
    daily_backup_hour: i32,
    backup_path: String,
    save_path: String,
    is_backup_up: bool,
    log_messages: Vec<String>,
    next_backup_time: Option<chrono::NaiveDateTime>,
    show_confirm_dialog: bool,
}

// 实现 eframe::App trait
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_backup_up {
            let now = Local::now();
            
            if self.current_selection == TimeSelection::Day {
                // 如果是按天备份，检查是否到达今天的备份时间
                if self.next_backup_time.is_none() {
                    // 计算下一次备份时间
                    let mut next_backup = now.date_naive().and_hms_opt(
                        self.daily_backup_hour as u32, 
                        0, 
                        0
                    ).unwrap();
                    
                    // 如果当前时间已经过了今天的备份时间，就设置为明天
                    if now.hour() >= self.daily_backup_hour as u32 {
                        next_backup = next_backup + chrono::TimeDelta::try_days(1).unwrap();
                    }
                    
                    self.next_backup_time = Some(next_backup);
                }

                // 检查是否到达备份时间
                if let Some(next_time) = self.next_backup_time {
                    if now.naive_local() >= next_time {
                        match self.start_backup() {
                            Ok(_) => self.add_log("Daily backup completed".to_string()),
                            Err(e) => self.add_log(format!("Daily backup failed: {}", e)),
                        }
                        // 设置下一天的备份时间
                        self.next_backup_time = Some(next_time + chrono::TimeDelta::try_days(1).unwrap());
                    }
                }
            } else {
                // 按秒、分、小时备份的逻辑
                if self.next_backup_time.is_none() {
                    // 设置下一次备份时间
                    self.next_backup_time = Some(now.naive_local() + chrono::TimeDelta::try_seconds(self.last_time as i64).unwrap());
                }

                if let Some(next_time) = self.next_backup_time {
                    if now.naive_local() >= next_time {
                        match self.start_backup() {
                            Ok(_) => self.add_log("Backup completed".to_string()),
                            Err(e) => self.add_log(format!("Backup failed: {}", e)),
                        }
                        // 设置下一次备份时间
                        self.next_backup_time = Some(next_time + chrono::TimeDelta::try_seconds(self.last_time as i64).unwrap());
                    }
                }
            }
            
            ctx.request_repaint();
        } else {
            self.next_backup_time = None;
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 标题
                ui.heading("Auto_Backup");
            });
            // 备份时间选择
            // 添加间距
            ui.add_space(10.0);
            // 水平布局
            ui.add_enabled_ui(!self.is_backup_up, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    self.last_time = add_time(self.time, &self.current_selection);
                    ui.label("Backup Time:");
                    // 获取最大值
                    let max_value = self.current_selection.value();
                    // 滑动条
                    ui.add(egui::Slider::new(&mut self.time, 0..=max_value));
                    // 下拉框
                    egui::ComboBox::from_label("Select Time")
                        .selected_text(match self.current_selection {
                            TimeSelection::Second => "Second",
                            TimeSelection::Minute => "Minute",
                            TimeSelection::Hour => "Hour",
                            TimeSelection::Day => "Day",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_selection,
                                TimeSelection::Second,
                                "Second",
                            );
                            ui.selectable_value(
                                &mut self.current_selection,
                                TimeSelection::Minute,
                                "Minute",
                            );
                            ui.selectable_value(
                                &mut self.current_selection,
                                TimeSelection::Hour,
                                "Hour",
                            );
                            ui.selectable_value(
                                &mut self.current_selection,
                                TimeSelection::Day,
                                "Day",
                            );
                        });
                });

            // 在时间选择的下方添加每日备份时间选择
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(self.current_selection == TimeSelection::Day, |ui| {
                        ui.label("Daily backup time:");
                    ui.add(egui::Slider::new(&mut self.daily_backup_hour, 0..=23).suffix("h"));
                });
            });

            // 如果选择了"天"作为备份周期，显示具体的备份时间说明
            if self.current_selection == TimeSelection::Day {
                ui.label(format!("Backup will occur at {:02}:00 every day", self.daily_backup_hour));
            }

                // 备份路径选择
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Backup Path:");
                    ui.text_edit_singleline(&mut self.backup_path);
                    if ui.button("Browse...").clicked() {
                        if let Some(selected_path) = FileDialog::new().pick_folder() {
                            self.backup_path = selected_path.display().to_string();
                        }
                    }
                });
                // 保存路径选择
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Save Path:     ");
                    ui.text_edit_singleline(&mut self.save_path);
                    if ui.button("Browse...").clicked() {
                        if let Some(selected_path) = FileDialog::new().pick_folder() {
                            self.save_path = selected_path.display().to_string();
                        }
                    }
                });
            });
            ui.add_space(10.0);
            // 除了天,别的全都是从当前时间开始计时
            ui.label("Seconds, minutes, and hours start counting from when the software begins the backup process.");



            // 添加日志显示区域
            ui.add_space(10.0);
            ui.label("Log:");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    for message in &self.log_messages {
                        ui.label(message);
                    }
                });
            
            // 添加清除日志按钮
            if ui.button("Clear Log").clicked() {
                self.log_messages.clear();
            }
            ui.add_space(20.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.add_space(10.0);
                // 停止按钮
                ui.add_enabled_ui(self.is_backup_up, |ui: &mut egui::Ui| {
                    if ui.button("stop").clicked() {
                        self.show_confirm_dialog = true;
                    };
                });
                // 备份按钮
                let paths_valid = !self.backup_path.is_empty() 
                    && !self.save_path.is_empty()
                    && self.validate_paths().is_ok();

                ui.add_enabled_ui(!self.is_backup_up && paths_valid, |ui| {
                    if ui.button("Backup").clicked() {
                        self.is_backup_up = true;
                        self.add_log("Backup started".to_string());
                    }
                });

                // 显示剩余时间
                if self.is_backup_up {
                    if self.current_selection == TimeSelection::Day {
                        // 按天备份的倒计时显示
                        if let Some(next_time) = self.next_backup_time {
                            let now = Local::now().naive_local();
                            if next_time > now {
                                let duration = next_time.signed_duration_since(now);
                                let hours = duration.num_hours();
                                let minutes = duration.num_minutes() % 60;
                                let seconds = duration.num_seconds() % 60;
                                
                                ui.label(format!(
                                    "Next backup in: {}h {}m {}s",
                                    hours,
                                    minutes,
                                    seconds
                                ));
                            }
                        }
                    } else {
                        // 其他时间单位的倒计时显示（保持原有逻辑）
                        if let Some(next_time) = self.next_backup_time {
                            let now = Local::now().naive_local();
                            if next_time > now {
                                let duration = next_time.signed_duration_since(now);
                                let seconds = duration.num_seconds();
                                let hours = seconds / 3600;
                                let minutes = (seconds % 3600) / 60;
                                let secs = seconds % 60;
                                
                                let time_text = if hours > 0 {
                                    format!("Next backup in: {}h {}m {}s", hours, minutes, secs)
                                } else if minutes > 0 {
                                    format!("Next backup in: {}m {}s", minutes, secs)
                                } else {
                                    format!("Next backup in: {}s", secs)
                                };
                                
                                ui.label(time_text);
                            }
                        }
                    }
                }

                if ui.button("test").clicked() {
                    self.add_log("Test button clicked".to_string());
                    self.add_log(format!("Backup path: {}", self.backup_path));
                    self.add_log(format!("Save path: {}", self.save_path));
                    self.add_log(format!("Time setting: {}", self.time));
                    self.add_log(format!("Calculated time: {}", self.last_time));
                };
            });


        });

        if self.show_confirm_dialog {
            egui::Window::new("Confirm")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to stop the backup?");
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.is_backup_up = false;
                            self.show_confirm_dialog = false;
                            self.add_log("Backup stopped".to_string());
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_confirm_dialog = false;
                        }
                    });
                });
        }
    }
}

// 这里根据下拉框和滑动条的值来计算时间
fn add_time(time: i32, current_selection: &TimeSelection) -> i32 {
    match current_selection {
        TimeSelection::Second => time,
        TimeSelection::Minute => time * 60,
        TimeSelection::Hour => time * 3600,
        TimeSelection::Day => time * 86400,
    }
}

fn backup_directory(from: &str, to: &str) -> io::Result<()> {
    // 获取当前时间并格式化
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("backup_{}", timestamp);
    let target_path = Path::new(to).join(backup_name);

    // 创建目标目录
    std::fs::create_dir_all(&target_path)?;

    // 配置复制选项
    let options = CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: true,
        content_only: false,
        ..Default::default()
    };

    // 执行复制
    copy(from, &target_path, &options).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("Backup Success: {}", target_path.display());
    Ok(())
}

impl MyApp {
    // 添加日志方法
    fn add_log(&mut self, message: String) {
        self.log_messages.push(format!("[{}] {}", 
            Local::now().format("%H:%M:%S"),
            message
        ));
    }

    fn validate_paths(&self) -> Result<(), String> {
        // 添加错误处理
        if self.backup_path.is_empty() || self.save_path.is_empty() {
            return Err("Paths cannot be empty".to_string());
        }

        // 使用更安全的路径检查
        let backup_path = Path::new(&self.backup_path);
        let save_path = Path::new(&self.save_path);

        if !backup_path.exists() {
            return Err("Backup path does not exist".to_string());
        }

        // 避免直接创建目录可能导致的问题
        if !save_path.exists() {
            return Err("Save path does not exist".to_string());
        }

        Ok(())
    }

    // 修改开始备份函数
    fn start_backup(&mut self) -> io::Result<()> {
        // 添加额外的安全检查
        if self.backup_path.is_empty() || self.save_path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Paths cannot be empty"
            ));
        }

        if let Err(e) = self.validate_paths() {
            self.add_log(format!("Error: {}", e));
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }

        match backup_directory(&self.backup_path, &self.save_path) {
            Ok(_) => {
                self.add_log("Backup completed successfully".to_string());
                Ok(())
            },
            Err(e) => {
                let error_msg = match e.kind() {
                    io::ErrorKind::PermissionDenied => "Permission denied",
                    io::ErrorKind::NotFound => "File or directory not found",
                    io::ErrorKind::AlreadyExists => "Target file already exists",
                    io::ErrorKind::InvalidInput => "Invalid path",
                    _ => "Unknown error",
                };
                let detailed_error = format!("Backup failed: {} - {}", error_msg, e);
                self.add_log(detailed_error.clone());
                Err(io::Error::new(io::ErrorKind::Other, detailed_error))
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    // 添加错误处理
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            eprintln!(
                "程序发生错误 at {}:{}: {}",
                location.file(),
                location.line(),
                panic_info
            );
        }
    }));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 450.0])
            .with_decorations(true)
            .with_transparent(false), // 禁用透明
        renderer: eframe::Renderer::Glow,  // 使用Glow渲染器
        ..Default::default()
    };

    eframe::run_native(
        "Auto_Backup",
        native_options,
        Box::new(move |cc| {
            // 配置字体
            let mut fonts = egui::FontDefinitions::default();

            // 添加得意黑字体
            fonts.font_data.insert(
                "SmileySans-Oblique".to_owned(),
                egui::FontData::from_static(include_bytes!("..\\assets\\SmileySans-Oblique.ttf")),
            );

            // 将得意黑设置为默认字体
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "SmileySans-Oblique".to_owned());

            // 应用字体配置
            cc.egui_ctx.set_fonts(fonts);

            let app = MyApp::default();
            Ok(Box::new(app))
        }),
    )
}
