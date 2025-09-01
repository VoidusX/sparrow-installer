use anyhow::Result;
use clap::Parser;
use cli_log::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, poll},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use serde::Deserialize;
use std::io;
use std::time::{Duration, Instant};
use tokio::process::Command as AsyncCommand;

// Embedded configuration files
const THEME_CONFIG: &str = include_str!("theme.toml");
const TEXT_CONFIG: &str = include_str!("text.toml");

#[derive(Debug, Deserialize, Clone)]
struct ThemeConfig {
    colors: ThemeColors,
    ui: UiConfig,
    layout: LayoutConfig,
    progress: ThemeProgressConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct TextConfig {
    messages: Messages,
    ui_text: UiText,
    errors: Errors,
    dry_run: DryRun,
    progress: ProgressConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct Messages {
    welcome: String,
    confirmation_prompt: String,
    processing: String,
    dry_run_testing: String,
    dry_run_progress: String,
    dry_run_default_install: String,
    dry_run_system_update: String,
    dry_run_complete: String,
    dry_run_script_output: String,
    dry_run_update_output: String,
    dry_run_misc_text: String,
    operation_success: String,
    operation_failed: String,
    custom_disabled: String,
    option_disabled: String,
    navigation_help: String,
    confirmation_help: String,
    processing_help: String,
    disabled_help: String,
    password_help: String,
    password_prompt: String,
    password_instructions: String,
    password_empty_error: String,
    password_auth_failed: String,
    confirm_default_install: String,
    confirm_system_update: String,
    progress_installing: String,
    progress_updating: String,
    progress_preparing: String,
    progress_finalizing: String,
    progress_rebooting: String,
    progress_poweroff: String,
    spinner_chars: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct UiText {
    app_title: String,
    dry_run_indicator: String,
    default_title: String,
    default_description: String,
    custom_title: String,
    custom_description: String,
    update_title: String,
    update_description: String,
    exit_title: String,
    exit_description: String,
    success_prefix: String,
    error_prefix: String,
    fail_prefix: String,
    info_prefix: String,
    warning_prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct Errors {
    script_not_found: String,
    permission_denied: String,
    command_failed: String,
    network_error: String,
    disk_space_error: String,
    unknown_error: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct DryRun {
    mode_active: String,
    simulation_header: String,
    simulation_footer: String,
    would_execute: String,
    would_install: String,
    would_update: String,
    would_create: String,
    would_modify: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ProgressConfig {
    bar_fill_char: String,
    bar_empty_char: String,
    bar_width: u16,
    countdown_seconds: u16,
    indeterminate_actions: Vec<String>,
    determinante_actions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ThemeColors {
    primary: String,
    accent: String,
    title_bg: String,
    main_bg: String,
    content_bg: String,
    description_bg: String,
    title_fg: String,
    main_fg: String,
    content_fg: String,
    description_fg: String,
    selected_bg: String,
    selected_fg: String,
    disabled_bg: String,
    disabled_fg: String,
    confirmation_bg: String,
    confirmation_fg: String,
    success_bg: String,
    success_fg: String,
    error_bg: String,
    error_fg: String,
    fail_bg: String,
    fail_fg: String,
    dry_run_fg: String,
    separator_fg: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct UiConfig {
    title_height: u16,
    description_height: u16,
    show_separator: bool,
    separator_char: String,
    dry_run_icon: String,
    selection_prefix: String,
    disabled_suffix: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct LayoutConfig {
    title_alignment: String,
    content_alignment: String,
    description_alignment: String,
    confirmation_alignment: String,
    content_padding: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct ThemeProgressConfig {
    bar_color: String,
    bar_background: String,
    border_color: String,
    border_active_color: String,
    spinner_color: String,
    countdown_color: String,
    spinner_speed: u64,
    progress_bar_speed: u64,
    countdown_speed: u64,
}

impl ThemeConfig {
    fn load() -> Result<Self> {
        let config: ThemeConfig = toml::from_str(THEME_CONFIG)?;
        Ok(config)
    }
}

impl TextConfig {
    fn load() -> Result<Self> {
        let config: TextConfig = toml::from_str(TEXT_CONFIG)?;
        Ok(config)
    }
}

fn parse_color(color_str: &str) -> Color {
    match color_str {
        "Black" => Color::Black,
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "Gray" => Color::Gray,
        "DarkGray" => Color::DarkGray,
        "LightRed" => Color::LightRed,
        "LightGreen" => Color::LightGreen,
        "LightYellow" => Color::LightYellow,
        "LightBlue" => Color::LightBlue,
        "LightMagenta" => Color::LightMagenta,
        "LightCyan" => Color::LightCyan,
        "White" => Color::White,
        "Gold" => Color::Rgb(255, 215, 0),
        _ => Color::White,
    }
}

fn parse_alignment(alignment_str: &str) -> Alignment {
    match alignment_str {
        "Left" => Alignment::Left,
        "Center" => Alignment::Center,
        "Right" => Alignment::Right,
        _ => Alignment::Center,
    }
}

#[derive(Parser)]
#[command(name = "sparrow-installer")]
#[command(about = "Sparrow atomic desktop installer")]
struct Cli {
    /// Enable dry-run mode (don't execute actual commands)
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone)]
enum InstallerOption {
    Default,
    Custom,
    UpdateSystem,
    Exit,
}

impl InstallerOption {
    fn title<'a>(&self, text_config: &'a TextConfig) -> &'a str {
        match self {
            InstallerOption::Default => &text_config.ui_text.default_title,
            InstallerOption::Custom => &text_config.ui_text.custom_title,
            InstallerOption::UpdateSystem => &text_config.ui_text.update_title,
            InstallerOption::Exit => &text_config.ui_text.exit_title,
        }
    }

    fn description<'a>(&self, text_config: &'a TextConfig) -> &'a str {
        match self {
            InstallerOption::Default => &text_config.ui_text.default_description,
            InstallerOption::Custom => &text_config.ui_text.custom_description,
            InstallerOption::UpdateSystem => &text_config.ui_text.update_description,
            InstallerOption::Exit => &text_config.ui_text.exit_description,
        }
    }

    fn is_enabled(&self) -> bool {
        match self {
            InstallerOption::Custom => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    MainMenu,
    Confirmation,
    PasswordInput,
    Processing(String), // Processing with action description
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ProgressType {
    Indeterminate,
    Determinant(u16), // countdown seconds
}

#[derive(Debug, Clone)]
enum StatusType {
    Success,
    Error,
    Fail,
}

struct App {
    options: Vec<InstallerOption>,
    selected: usize,
    should_quit: bool,
    dry_run: bool,
    status_message: Option<(String, StatusType)>,
    show_confirmation: bool,
    confirmation_message: String,
    app_state: AppState,
    theme: ThemeConfig,
    text: TextConfig,
    progress_type: Option<ProgressType>,
    progress_step: usize,
    progress_bar_position: usize,
    countdown_remaining: u16,
    action_output: Vec<String>,
    last_spinner_update: Instant,
    last_progress_update: Instant,
    last_countdown_update: Instant,
    dry_run_start_time: Option<Instant>,
    password_input: String,
    pending_operation: Option<InstallerOption>,
    show_password: bool,
    pending_system_action: Option<SystemAction>,
}

#[derive(Clone, Debug)]
enum SystemAction {
    Reboot,
    Poweroff,
}

impl App {
    fn new(dry_run: bool) -> Result<Self> {
        let theme = ThemeConfig::load()?;
        let text = TextConfig::load()?;

        Ok(Self {
            options: vec![
                InstallerOption::Default,
                InstallerOption::Custom,
                InstallerOption::UpdateSystem,
                InstallerOption::Exit,
            ],
            selected: 0,
            should_quit: false,
            dry_run,
            status_message: None,
            show_confirmation: false,
            confirmation_message: String::new(),
            app_state: AppState::MainMenu,
            theme,
            text,
            progress_type: None,
            progress_step: 0,
            progress_bar_position: 0,
            countdown_remaining: 0,
            action_output: Vec::new(),
            last_spinner_update: Instant::now(),
            last_progress_update: Instant::now(),
            last_countdown_update: Instant::now(),
            dry_run_start_time: None,
            password_input: String::new(),
            pending_operation: None,
            show_password: false,
            pending_system_action: None,
        })
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.options.len() - 1
        } else {
            self.selected - 1
        };
    }

    fn show_confirmation(&mut self, message: String) {
        self.confirmation_message = message;
        self.show_confirmation = true;
        self.app_state = AppState::Confirmation;
    }

    fn hide_confirmation(&mut self) {
        self.show_confirmation = false;
        self.confirmation_message.clear();
        self.app_state = AppState::MainMenu;
    }

    fn show_password_input(&mut self, operation: InstallerOption) {
        self.app_state = AppState::PasswordInput;
        self.pending_operation = Some(operation);
        self.password_input.clear();
    }

    fn hide_password_input(&mut self) {
        self.app_state = AppState::MainMenu;
        self.pending_operation = None;
        self.password_input.clear();
        self.show_password = false;
    }

    async fn execute_option(&mut self) -> Result<()> {
        let option = &self.options[self.selected];

        match option {
            InstallerOption::Default => {
                if option.is_enabled() {
                    self.show_confirmation(self.text.messages.confirm_default_install.clone());
                } else {
                    self.status_message =
                        Some((self.text.messages.option_disabled.clone(), StatusType::Fail));
                }
            }
            InstallerOption::UpdateSystem => {
                if option.is_enabled() {
                    self.show_password_input(option.clone());
                } else {
                    self.status_message =
                        Some((self.text.messages.option_disabled.clone(), StatusType::Fail));
                }
            }
            InstallerOption::Exit => {
                self.start_poweroff();
            }
            InstallerOption::Custom => {
                self.status_message =
                    Some((self.text.messages.custom_disabled.clone(), StatusType::Fail));
            }
        }

        Ok(())
    }

    async fn confirm_password(&mut self) -> Result<()> {
        if let Some(operation) = self.pending_operation.clone() {
            //self.hide_password_input(); # DO NOT DO THAT IMMEDIATELY, THE PASSWORD WOULD GET THROWN AWAY
            if self.dry_run {
                // In dry-run mode, show confirmation after password input
                let confirmation_message = match operation {
                    InstallerOption::Default => self.text.messages.confirm_default_install.clone(),
                    InstallerOption::UpdateSystem => {
                        self.text.messages.confirm_system_update.clone()
                    }
                    _ => "Confirm operation?".to_string(),
                };
                self.show_confirmation(confirmation_message);
            } else {
                // Normal mode - proceed with operation
                let action_description = match operation {
                    InstallerOption::Default => self.text.messages.progress_installing.clone(),
                    InstallerOption::UpdateSystem => self.text.messages.progress_updating.clone(),
                    _ => self.text.messages.processing.clone(),
                };

                self.app_state = AppState::Processing(action_description.clone());

                // Set up progress type based on action
                self.progress_type = Some(match operation {
                    InstallerOption::Default => ProgressType::Indeterminate,
                    InstallerOption::UpdateSystem => ProgressType::Indeterminate,
                    _ => ProgressType::Indeterminate,
                });

                self.progress_step = 0;
                self.progress_bar_position = 0;
                self.countdown_remaining = self.text.progress.countdown_seconds;
                self.action_output.clear();
                let now = Instant::now();
                self.last_spinner_update = now;
                self.last_progress_update = now;
                self.last_countdown_update = now;

                let result = match operation {
                    InstallerOption::Default => self.install_default_dotfiles().await,
                    InstallerOption::UpdateSystem => self.update_system().await,
                    _ => Ok(()),
                };

                // If authentication failed, return to password input
                if let Err(ref e) = result {
                    if e.to_string()
                        .contains(&self.text.messages.password_auth_failed)
                    {
                        self.progress_type = None;
                        self.app_state = AppState::PasswordInput;
                        self.pending_operation = Some(operation);
                        self.password_input.clear();
                        self.status_message = Some((e.to_string(), StatusType::Error));
                        return Ok(());
                    }
                }

                self.finish_operation(result);
            }
        }
        Ok(())
    }

    async fn confirm_action(&mut self) -> Result<()> {
        let option = &self.options[self.selected].clone();
        self.hide_confirmation();

        let action_description = match option {
            InstallerOption::Default => self.text.messages.progress_installing.clone(),
            InstallerOption::UpdateSystem => self.text.messages.progress_updating.clone(),
            _ => self.text.messages.processing.clone(),
        };

        self.app_state = AppState::Processing(action_description.clone());

        // Set up progress type based on action
        self.progress_type = Some(match option {
            InstallerOption::Default => ProgressType::Indeterminate,
            InstallerOption::UpdateSystem => ProgressType::Indeterminate,
            _ => ProgressType::Indeterminate,
        });

        self.progress_step = 0;
        self.progress_bar_position = 0;
        self.countdown_remaining = self.text.progress.countdown_seconds;
        self.action_output.clear();
        let now = Instant::now();
        self.last_spinner_update = now;
        self.last_progress_update = now;
        self.last_countdown_update = now;

        if self.dry_run {
            // Start simulation with timeout tracking
            self.dry_run_start_time = Some(Instant::now());
            self.start_simulation(&option);
        } else {
            let result = match option {
                InstallerOption::Default => self.install_default_dotfiles().await,
                InstallerOption::UpdateSystem => self.update_system().await,
                _ => Ok(()),
            };

            self.finish_operation(result);
        }

        Ok(())
    }

    fn start_simulation(&mut self, option: &InstallerOption) {
        match option {
            InstallerOption::Default => {
                let output_lines: Vec<&str> = self
                    .text
                    .messages
                    .dry_run_script_output
                    .split('\n')
                    .collect();
                self.action_output = output_lines.iter().map(|s| s.to_string()).collect();
            }
            InstallerOption::UpdateSystem => {
                let output_lines: Vec<&str> = self
                    .text
                    .messages
                    .dry_run_update_output
                    .split('\n')
                    .collect();
                self.action_output = output_lines.iter().map(|s| s.to_string()).collect();
            }
            _ => {}
        }
    }

    fn update_progress(&mut self) {
        if let Some(ref progress_type) = self.progress_type.clone() {
            let now = Instant::now();

            // Check for dry-run timeout (10 seconds)
            if self.dry_run {
                if let Some(start_time) = self.dry_run_start_time {
                    if now.duration_since(start_time) >= Duration::from_secs(10) {
                        self.finish_current_operation();
                        return;
                    }
                }
            }

            match progress_type {
                ProgressType::Indeterminate => {
                    // Update spinner based on configured speed
                    let spinner_interval = Duration::from_millis(self.theme.progress.spinner_speed);
                    if now.duration_since(self.last_spinner_update) >= spinner_interval {
                        self.progress_step =
                            (self.progress_step + 1) % self.text.messages.spinner_chars.len();
                        self.last_spinner_update = now;
                    }

                    // Update progress bar based on configured speed
                    let progress_interval =
                        Duration::from_millis(self.theme.progress.progress_bar_speed);
                    if now.duration_since(self.last_progress_update) >= progress_interval {
                        self.progress_bar_position += 1;
                        self.last_progress_update = now;
                    }
                }
                ProgressType::Determinant(_) => {
                    // Update countdown based on configured speed (1 second intervals for countdown)
                    let countdown_interval = Duration::from_secs(1);
                    if now.duration_since(self.last_countdown_update) >= countdown_interval {
                        if self.countdown_remaining > 0 {
                            self.countdown_remaining -= 1;
                            self.last_countdown_update = now;
                        } else {
                            self.finish_current_operation();
                        }
                    }
                }
            }
        }
    }

    fn finish_current_operation(&mut self) {
        // Check if we're finishing a determinant action (reboot/poweroff)
        if let Some(ProgressType::Determinant(_)) = self.progress_type {
            // Check if this is a poweroff action
            let is_poweroff = if let Some(SystemAction::Poweroff) = &self.pending_system_action {
                true
            } else {
                false
            };

            // For dry-run mode
            if self.dry_run {
                self.progress_type = None;
                self.action_output.clear();
                self.progress_bar_position = 0;
                self.dry_run_start_time = None;
                self.password_input.clear();

                if is_poweroff {
                    // For poweroff, quit the app even in dry-run
                    self.should_quit = true;
                } else {
                    // For reboot, return to menu in dry-run
                    self.app_state = AppState::MainMenu;
                    self.status_message = Some((
                        "DRY-RUN: System action simulation complete".to_string(),
                        StatusType::Success,
                    ));
                    let now = Instant::now();
                    self.last_spinner_update = now;
                    self.last_progress_update = now;
                    self.last_countdown_update = now;
                }
                return;
            }

            // Set should_quit to true for both reboot and poweroff
            // The actual system command execution will happen after the UI loop exits
            self.should_quit = true;
            return;
        }

        // Standard operation completion
        self.progress_type = None;
        self.app_state = AppState::MainMenu;
        self.status_message = Some((
            self.text.messages.operation_success.clone(),
            StatusType::Success,
        ));
        self.action_output.clear();
        self.progress_bar_position = 0;
        self.dry_run_start_time = None;
        self.password_input.clear(); // Clear password for security
        let now = Instant::now();
        self.last_spinner_update = now;
        self.last_progress_update = now;
        self.last_countdown_update = now;
    }

    fn finish_operation(&mut self, result: Result<()>) {
        match result {
            Ok(()) => {
                self.progress_type = None;
                self.app_state = AppState::MainMenu;
                self.status_message = Some((
                    self.text.messages.operation_success.clone(),
                    StatusType::Success,
                ));
            }
            Err(e) => {
                // Check if this was a dotfiles installation failure
                if let Some(InstallerOption::Default) = &self.pending_operation {
                    // Dotfiles installation failed - trigger reboot
                    self.pending_system_action = Some(SystemAction::Reboot);
                    self.start_reboot();
                    return;
                } else {
                    // Other operation failed - return to main menu with error
                    self.progress_type = None;
                    self.app_state = AppState::MainMenu;
                    self.status_message = Some((format!("Error: {}", e), StatusType::Error));
                }
            }
        }
        self.action_output.clear();
        self.progress_bar_position = 0;
        self.dry_run_start_time = None;
        self.password_input.clear(); // Clear password for security
        let now = Instant::now();
        self.last_spinner_update = now;
        self.last_progress_update = now;
        self.last_countdown_update = now;
    }

    async fn install_default_dotfiles(&self) -> Result<()> {
        let script_path = "/usr/share/hypr/end-4_installer/setup.sh";

        if self.dry_run {
            return Ok(());
        }

        let output = AsyncCommand::new("bash").arg(script_path).output().await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Setup script failed: {}", error_msg));
        }

        Ok(())
    }

    async fn update_system(&mut self) -> Result<()> {
        debug!("Inputted PWD: {}", self.password_input);
        debug!("Dry-run State: {}", self.dry_run);
        if self.dry_run {
            return Ok(());
        }

        let mut cmd = AsyncCommand::new("sudo");
        cmd.args(["-S", "bootc", "update", "--apply"]);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(format!("{}\n", self.password_input).as_bytes())
                .await?;
            self.password_input.clear();
        }

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            if error_msg.contains("Sorry, try again") || error_msg.contains("incorrect password") {
                return Err(anyhow::anyhow!(
                    "{}",
                    self.text.messages.password_auth_failed
                ));
            }
            return Err(anyhow::anyhow!("System update failed: {}", error_msg));
        }

        Ok(())
    }

    fn start_reboot(&mut self) {
        self.app_state = AppState::Processing(self.text.messages.progress_rebooting.clone());
        self.progress_type = Some(ProgressType::Determinant(
            self.text.progress.countdown_seconds,
        ));
        self.countdown_remaining = self.text.progress.countdown_seconds;
        self.progress_step = 0;
        self.progress_bar_position = 0;
        self.action_output.clear();
        let now = Instant::now();
        self.last_spinner_update = now;
        self.last_progress_update = now;
        self.last_countdown_update = now;
        self.dry_run_start_time = if self.dry_run { Some(now) } else { None };
        self.pending_system_action = Some(SystemAction::Reboot);
    }

    fn start_poweroff(&mut self) {
        self.app_state = AppState::Processing(self.text.messages.progress_poweroff.clone());
        self.progress_type = Some(ProgressType::Determinant(
            self.text.progress.countdown_seconds,
        ));
        self.countdown_remaining = self.text.progress.countdown_seconds;
        self.progress_step = 0;
        self.progress_bar_position = 0;
        self.action_output.clear();
        let now = Instant::now();
        self.last_spinner_update = now;
        self.last_progress_update = now;
        self.last_countdown_update = now;
        self.dry_run_start_time = if self.dry_run { Some(now) } else { None };
        self.pending_system_action = Some(SystemAction::Poweroff);
    }

    async fn execute_reboot(&self) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }

        let output = AsyncCommand::new("systemctl")
            .arg("reboot")
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Reboot failed: {}", error_msg));
        }

        Ok(())
    }

    async fn execute_poweroff(&self) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }

        let output = AsyncCommand::new("systemctl")
            .arg("poweroff")
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Poweroff failed: {}", error_msg));
        }

        Ok(())
    }

    fn clear_status(&mut self) {
        self.status_message = None;
    }

    fn get_title_text(&self) -> String {
        if self.progress_type.is_some() {
            return self.text.messages.dry_run_testing.clone();
        }

        match &self.app_state {
            AppState::MainMenu => self.text.messages.welcome.clone(),
            AppState::Confirmation => self.text.messages.confirmation_prompt.clone(),
            AppState::PasswordInput => self.text.messages.password_prompt.clone(),
            AppState::Processing(action) => action.clone(),
        }
    }

    fn get_spinner_char(&self) -> String {
        if let Some(ProgressType::Indeterminate) = &self.progress_type {
            self.text.messages.spinner_chars[self.progress_step].clone()
        } else {
            String::new()
        }
    }

    fn get_progress_bar(&self, width: u16) -> String {
        if let Some(ref progress_type) = self.progress_type {
            let bar_width = width.saturating_sub(4) as usize; // Account for margins
            match progress_type {
                ProgressType::Indeterminate => {
                    // Create animated indeterminate bar using strikethrough
                    let cycle_length = bar_width + 6; // Full width plus extra for smooth cycling
                    let pos = self.progress_bar_position % cycle_length;
                    let mut bar = vec!['─'; bar_width]; // Use line character
                    for i in 0..4 {
                        let bar_pos = pos as i32 - i as i32;
                        if bar_pos >= 0 && (bar_pos as usize) < bar_width {
                            bar[bar_pos as usize] = '━'; // Use thick line for filled
                        }
                    }
                    bar.into_iter().collect()
                }
                ProgressType::Determinant(_) => {
                    // Create countdown progress bar using strikethrough
                    let filled = ((self.countdown_remaining as f32
                        / self.text.progress.countdown_seconds as f32)
                        * bar_width as f32) as usize;
                    let mut bar = vec!['─'; bar_width]; // Use line character
                    for i in 0..filled {
                        bar[i] = '━'; // Use thick line for filled
                    }
                    bar.into_iter().collect()
                }
            }
        } else {
            String::new()
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(theme.ui.title_height), // Title with separator and subtitle
            Constraint::Min(0),                        // Main content
            Constraint::Length(theme.ui.description_height), // Description/Status
        ])
        .split(f.size());

    // Title area
    let title_area = main_layout[0];

    let separator = if theme.ui.show_separator {
        format!(
            "\n{}",
            theme.ui.separator_char.repeat(title_area.width as usize)
        )
    } else {
        String::new()
    };

    let title_content = format!(
        "{}{}{}{}",
        app.text.ui_text.app_title,
        separator,
        if theme.ui.show_separator { "\n" } else { "" },
        app.get_title_text()
    );

    let title = Paragraph::new(title_content)
        .style(
            Style::default()
                .bg(parse_color(&theme.colors.title_bg))
                .fg(parse_color(&theme.colors.title_fg))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(parse_alignment(&theme.layout.title_alignment));
    f.render_widget(title, title_area);

    // Main content area
    if app.app_state == AppState::PasswordInput {
        // Show password input dialog with bordered input box
        let content_area = main_layout[1];

        // Split content area to have instruction text and input box
        let password_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Instructions area
                Constraint::Length(3), // Input box area (border + content + border)
                Constraint::Length(1), // Bottom instructions
            ])
            .split(content_area);

        // Show instructions at the top
        let instructions = Paragraph::new("Enter your password for sudo authentication:")
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.confirmation_bg))
                    .fg(parse_color(&theme.colors.confirmation_fg)),
            )
            .alignment(parse_alignment(&theme.layout.confirmation_alignment));

        f.render_widget(instructions, password_layout[0]);

        // Create bordered input box
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(parse_color(&theme.colors.primary)))
            .style(Style::default().bg(parse_color(&theme.colors.content_bg)));

        let input_area = input_block.inner(password_layout[1]);
        f.render_widget(input_block, password_layout[1]);

        // Show password with cursor
        let password_display = if app.show_password {
            format!("{}█", app.password_input)
        } else {
            format!("{}█", "*".repeat(app.password_input.len()))
        };

        let password_input = Paragraph::new(password_display)
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.content_bg))
                    .fg(parse_color(&theme.colors.content_fg)),
            )
            .alignment(Alignment::Left);

        f.render_widget(password_input, input_area);

        // Show bottom instructions
        let bottom_instructions = Paragraph::new(app.text.messages.password_instructions.clone())
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.confirmation_bg))
                    .fg(parse_color(&theme.colors.confirmation_fg)),
            )
            .alignment(Alignment::Center);

        f.render_widget(bottom_instructions, password_layout[2]);
    } else if app.show_confirmation {
        // Show confirmation dialog
        let confirmation_text = format!(
            "{}\n\n{}",
            app.confirmation_message, app.text.messages.confirmation_help
        );

        let confirmation = Paragraph::new(confirmation_text)
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.confirmation_bg))
                    .fg(parse_color(&theme.colors.confirmation_fg)),
            )
            .alignment(parse_alignment(&theme.layout.confirmation_alignment))
            .wrap(Wrap { trim: true });

        f.render_widget(confirmation, main_layout[1]);
    } else if app.progress_type.is_some() {
        // Show action content with yellow border
        let content_area = main_layout[1];

        // Create bordered area
        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(parse_color(&theme.progress.border_active_color)))
            .style(Style::default().bg(parse_color(&theme.colors.content_bg)));

        let inner_area = border_block.inner(content_area);
        f.render_widget(border_block, content_area);

        // Show action output
        let mut content_lines = Vec::new();

        // Add current action description without spinner (spinner is in status area)
        let action_desc = match &app.app_state {
            AppState::Processing(desc) => desc.clone(),
            _ => "Processing...".to_string(),
        };
        content_lines.push(action_desc);
        content_lines.push(String::new()); // Empty line

        // Add action output
        for line in &app.action_output {
            content_lines.push(line.clone());
        }

        // Add dry-run misc text if in dry-run mode
        if app.dry_run {
            content_lines.push(String::new()); // Empty line
            for line in app.text.messages.dry_run_misc_text.split('\n') {
                content_lines.push(line.to_string());
            }
        }

        let action_content = Paragraph::new(content_lines.join("\n"))
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.content_bg))
                    .fg(parse_color(&theme.colors.content_fg)),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_widget(action_content, inner_area);
    } else {
        // Show options list
        let options: Vec<ListItem> =
            app.options
                .iter()
                .enumerate()
                .map(|(i, option)| {
                    let (bg_color, fg_color, prefix) = if i == app.selected {
                        (
                            parse_color(&theme.colors.selected_bg),
                            parse_color(&theme.colors.selected_fg),
                            theme.ui.selection_prefix.as_str(),
                        )
                    } else if !option.is_enabled() {
                        (
                            parse_color(&theme.colors.disabled_bg),
                            parse_color(&theme.colors.disabled_fg),
                            "",
                        )
                    } else {
                        (
                            parse_color(&theme.colors.content_bg),
                            parse_color(&theme.colors.content_fg),
                            "",
                        )
                    };

                    let content = if !option.is_enabled() {
                        format!("{}{}", option.title(&app.text), theme.ui.disabled_suffix)
                    } else {
                        option.title(&app.text).to_string()
                    };

                    let display_text = format!(
                        "{}{}{}",
                        " ".repeat(theme.layout.content_padding as usize),
                        prefix,
                        content
                    );

                    ListItem::new(display_text).style(
                        Style::default().bg(bg_color).fg(fg_color).add_modifier(
                            if i == app.selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            },
                        ),
                    )
                })
                .collect();

        let options_list =
            List::new(options).style(Style::default().bg(parse_color(&theme.colors.main_bg)));

        f.render_widget(options_list, main_layout[1]);
    }

    // Description/Status area
    let description_area = main_layout[2];

    if app.progress_type.is_some() {
        // Show spinner text above progress bar in status area
        let spinner_text = match &app.app_state {
            AppState::Processing(desc) => {
                if let Some(ProgressType::Determinant(_)) = &app.progress_type {
                    format!("{} {}", app.countdown_remaining, desc)
                } else {
                    format!("{} {}", app.get_spinner_char(), desc)
                }
            }
            _ => "Processing...".to_string(),
        };

        let progress_bar = app.get_progress_bar(description_area.width);

        let status_content = format!("{}\n{}", spinner_text, progress_bar);

        let progress_widget = Paragraph::new(status_content)
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.description_bg))
                    .fg(parse_color(&theme.progress.bar_color)),
            )
            .alignment(Alignment::Center);

        f.render_widget(progress_widget, description_area);
    } else if let Some((message, status_type)) = &app.status_message {
        // Show status message with navigation help
        let (bg_color, fg_color, prefix) = match status_type {
            StatusType::Success => (
                parse_color(&theme.colors.success_bg),
                parse_color(&theme.colors.success_fg),
                &app.text.ui_text.success_prefix,
            ),
            StatusType::Error => (
                parse_color(&theme.colors.error_bg),
                parse_color(&theme.colors.error_fg),
                &app.text.ui_text.error_prefix,
            ),
            StatusType::Fail => (
                parse_color(&theme.colors.fail_bg),
                parse_color(&theme.colors.fail_fg),
                &app.text.ui_text.fail_prefix,
            ),
        };

        // Create status message with navigation help
        let status_line = vec![
            Span::styled(
                prefix,
                Style::default()
                    .bg(bg_color)
                    .fg(fg_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(": {}", message),
                Style::default()
                    .bg(parse_color(&theme.colors.description_bg))
                    .fg(Color::White),
            ),
        ];

        // Render status line first
        let status = Paragraph::new(Line::from(status_line))
            .style(Style::default().bg(parse_color(&theme.colors.description_bg)))
            .alignment(parse_alignment(&theme.layout.description_alignment));

        // Calculate area for status line and navigation help
        let status_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status message line
                Constraint::Min(0),    // Navigation help
            ])
            .split(description_area);

        f.render_widget(status, status_layout[0]);

        // Render navigation help below status
        let help = Paragraph::new(format!("\n{}", app.text.messages.navigation_help))
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.description_bg))
                    .fg(parse_color(&theme.colors.description_fg)),
            )
            .alignment(parse_alignment(&theme.layout.description_alignment))
            .wrap(Wrap { trim: true });

        f.render_widget(help, status_layout[1]);
    } else {
        // Show description or help text
        let description_text = if app.show_confirmation {
            // Don't show confirmation help in description area since it's already in the main confirmation dialog
            "Review your selection carefully before confirming.".to_string()
        } else if app.app_state == AppState::PasswordInput {
            app.text.messages.password_help.clone()
        } else {
            match &app.app_state {
                AppState::Processing(_) => app.text.messages.processing_help.clone(),
                _ => {
                    let selected_option = &app.options[app.selected];
                    if selected_option.is_enabled() {
                        format!(
                            "{}\n\n{}",
                            selected_option.description(&app.text),
                            app.text.messages.navigation_help
                        )
                    } else {
                        format!(
                            "{}\n\n{}",
                            app.text.messages.disabled_help, app.text.messages.navigation_help
                        )
                    }
                }
            }
        };

        let description = Paragraph::new(description_text)
            .style(
                Style::default()
                    .bg(parse_color(&theme.colors.description_bg))
                    .fg(parse_color(&theme.colors.description_fg)),
            )
            .alignment(parse_alignment(&theme.layout.description_alignment))
            .wrap(Wrap { trim: true });

        f.render_widget(description, description_area);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_cli_log!();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(cli.dry_run)?;
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Check if there's a pending system action to execute
    if let Some(system_action) = &app.pending_system_action {
        if !app.dry_run {
            match system_action {
                SystemAction::Reboot => {
                    if let Err(err) = app.execute_reboot().await {
                        eprintln!("Reboot failed: {}", err);
                    }
                }
                SystemAction::Poweroff => {
                    if let Err(err) = app.execute_poweroff().await {
                        eprintln!("Poweroff failed: {}", err);
                    }
                }
            }
        }
    }

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Update progress if in progress mode (time-based, independent of events)
        if app.progress_type.is_some() {
            app.update_progress();
        }

        // Use shorter timeout for responsive UI but progress updates are time-based
        let timeout = std::time::Duration::from_millis(50);

        if poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.app_state == AppState::PasswordInput {
                    match key.code {
                        KeyCode::Enter => {
                            if !app.password_input.is_empty() {
                                if let Err(e) = app.confirm_password().await {
                                    app.status_message =
                                        Some((format!("Error: {}", e), StatusType::Error));
                                }
                            } else {
                                app.status_message = Some((
                                    app.text.messages.password_empty_error.clone(),
                                    StatusType::Error,
                                ));
                            }
                        }
                        KeyCode::Esc => {
                            app.hide_password_input();
                        }
                        KeyCode::Tab => {
                            app.show_password = !app.show_password;
                        }
                        KeyCode::Backspace => {
                            app.password_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.password_input.push(c);
                        }
                        _ => {}
                    }
                } else if app.show_confirmation {
                    match key.code {
                        KeyCode::Enter => {
                            if let Err(e) = app.confirm_action().await {
                                app.status_message =
                                    Some((format!("Error: {}", e), StatusType::Error));
                            }
                        }
                        KeyCode::Char('y') => {
                            if let Err(e) = app.confirm_action().await {
                                app.status_message =
                                    Some((format!("Error: {}", e), StatusType::Error));
                            }
                        }
                        KeyCode::Esc => {
                            app.hide_confirmation();
                        }
                        KeyCode::Char('n') => {
                            app.hide_confirmation();
                        }
                        _ => {}
                    }
                } else if app.progress_type.is_some() {
                    // Prevent ESC during processing operations (installations/updates)
                    match key.code {
                        KeyCode::Esc => {
                            // Only allow ESC cancellation during dry-run simulations
                            if app.dry_run {
                                app.progress_type = None;
                                app.app_state = AppState::MainMenu;
                                app.status_message =
                                    Some(("Simulation cancelled.".to_string(), StatusType::Error));
                                app.action_output.clear();
                                app.dry_run_start_time = None;
                            }
                            // For actual installations/updates, ESC is ignored
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.start_poweroff();
                        }
                        KeyCode::Down => {
                            app.next();
                            app.clear_status();
                        }
                        KeyCode::Up => {
                            app.previous();
                            app.clear_status();
                        }
                        KeyCode::Enter => {
                            if let Err(e) = app.execute_option().await {
                                app.status_message =
                                    Some((format!("Error: {}", e), StatusType::Error));
                            }
                        }
                        KeyCode::Esc => {
                            app.clear_status();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
