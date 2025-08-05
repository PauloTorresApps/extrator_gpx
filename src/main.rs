// Declara os novos módulos que farão parte do projeto.
mod drawing;
mod processing;
mod utils;

use iced::widget::{button, column, container, text, row, Scrollable, Space, pick_list};
use iced::{executor, Application, Command, Element, Length, Settings, Theme, Alignment};
use rfd::FileDialog;
use std::path::PathBuf;

// Ponto de entrada da aplicação GUI
pub fn main() -> iced::Result {
    GpxVideoApp::run(Settings::default())
}

// Enum para controlar qual vista está ativa
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Main,
    Logs,
}

// Enum para controlar o tema da aplicação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTheme {
    Light,
    Dark,
}

// Implementa a conversão para String para ser usado no PickList
impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppTheme::Light => "Claro",
                AppTheme::Dark => "Escuro",
            }
        )
    }
}

// Estrutura que guarda o estado da nossa aplicação
struct GpxVideoApp {
    gpx_path: Option<PathBuf>,
    video_path: Option<PathBuf>,
    status: String,
    is_processing: bool,
    current_view: View,
    logs: Vec<String>,
    theme: AppTheme,
}

// Mensagens que a nossa aplicação pode receber (eventos)
#[derive(Debug, Clone)]
enum Message {
    SwitchView(View),
    SelectGpx,
    SelectVideo,
    GpxSelected(Option<PathBuf>),
    VideoSelected(Option<PathBuf>),
    Generate,
    ProcessingFinished(Result<Vec<String>, (String, Vec<String>)>),
    ThemeChanged(AppTheme),
}

// Implementa o trait `Application` para uma app assíncrona
impl Application for GpxVideoApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    // Inicializa o estado da aplicação
    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                gpx_path: None,
                video_path: None,
                status: "Pronto para começar. Selecione os ficheiros na tela Principal.".to_string(),
                is_processing: false,
                current_view: View::Main,
                logs: Vec::new(),
                theme: AppTheme::Dark, // Define o tema escuro como padrão
            },
            Command::none(),
        )
    }

    // Define o título da janela
    fn title(&self) -> String {
        String::from("GPX Video Overlay Generator")
    }

    // Define o tema da aplicação
    fn theme(&self) -> Theme {
        match self.theme {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
        }
    }

    // Atualiza o estado da aplicação com base nas mensagens recebidas
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SwitchView(view) => {
                self.current_view = view;
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::SelectGpx => {
                return Command::perform(
                    async {
                        FileDialog::new()
                            .add_filter("GPX Files", &["gpx"])
                            .pick_file()
                    },
                    Message::GpxSelected,
                );
            }
            Message::SelectVideo => {
                return Command::perform(
                    async {
                        FileDialog::new()
                            .add_filter("Video Files", &["mp4", "MP4", "mov", "avi"])
                            .pick_file()
                    },
                    Message::VideoSelected,
                );
            }
            Message::GpxSelected(path) => {
                if let Some(p) = path {
                    self.status = format!("Ficheiro GPX carregado: {}", p.file_name().unwrap().to_str().unwrap());
                    self.gpx_path = Some(p);
                }
            }
            Message::VideoSelected(path) => {
                if let Some(p) = path {
                    self.status = format!("Ficheiro de vídeo carregado: {}", p.file_name().unwrap().to_str().unwrap());
                    self.video_path = Some(p);
                }
            }
            Message::Generate => {
                if let (Some(gpx_path), Some(video_path)) = (self.gpx_path.clone(), self.video_path.clone()) {
                    self.is_processing = true;
                    self.status = "A processar... Por favor, aguarde.".to_string();
                    self.logs = vec!["Iniciando processamento...".to_string()];
                    
                    return Command::perform(
                        async { processing::run_processing(gpx_path, video_path) },
                        Message::ProcessingFinished,
                    );
                }
            }
            Message::ProcessingFinished(result) => {
                self.is_processing = false;
                match result {
                    Ok(logs) => {
                        self.status = "Sucesso! O ficheiro 'output_video.mp4' foi gerado.".to_string();
                        self.logs = logs;
                    },
                    Err((err_msg, logs)) => {
                        self.status = format!("Erro: {}", err_msg);
                        self.logs = logs;
                        self.logs.push(format!("ERRO FINAL: {}", err_msg));
                    }
                }
            }
        }
        Command::none()
    }

    // Desenha a interface gráfica com base no estado atual
    fn view(&self) -> Element<Message> {
        // --- Menu Horizontal Superior ---
        let menu = container(
            row![
                button("Principal").on_press(Message::SwitchView(View::Main)).padding(10),
                button("Logs").on_press(Message::SwitchView(View::Logs)).padding(10),
                Space::with_width(Length::Fill), // Empurra o resto para a direita
                text("Tema:"),
                pick_list(
                    &[AppTheme::Light, AppTheme::Dark][..],
                    Some(self.theme),
                    Message::ThemeChanged
                ),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(5)
        .style(iced::theme::Container::Box);


        // --- Conteúdo Central Dinâmico ---
        let content = match self.current_view {
            View::Main => self.view_main(),
            View::Logs => self.view_logs(),
        };

        // --- Barra de Status Inferior ---
        let status_bar = container(
            text(&self.status).size(16)
        )
        .width(Length::Fill)
        .padding(10)
        .style(iced::theme::Container::Box);


        // --- Layout Principal ---
        column![
            menu,
            // O container do conteúdo preenche o espaço restante
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y(),
            status_bar,
        ]
        .spacing(5)
        .into()
    }
}

impl GpxVideoApp {
    // Constrói a vista principal
    fn view_main(&self) -> Element<Message> {
        let gpx_text = self.gpx_path.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| format!("GPX: {}", s))
            .unwrap_or_else(|| "Nenhum ficheiro GPX selecionado".to_string());

        let video_text = self.video_path.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| format!("Vídeo: {}", s))
            .unwrap_or_else(|| "Nenhum ficheiro de vídeo selecionado".to_string());

        let mut generate_button = button("Gerar Vídeo").padding(15);
        if !self.is_processing && self.gpx_path.is_some() && self.video_path.is_some() {
            generate_button = generate_button.on_press(Message::Generate);
        }

        column![
            text("Bem-vindo ao Gerador de Overlay de Vídeo!").size(24),
            Space::with_height(Length::Fixed(20.0)),
            button("Selecionar Ficheiro GPX").on_press(Message::SelectGpx).padding(10),
            text(gpx_text),
            Space::with_height(Length::Fixed(10.0)),
            button("Selecionar Ficheiro de Vídeo").on_press(Message::SelectVideo).padding(10),
            text(video_text),
            Space::with_height(Length::Fixed(30.0)),
            generate_button,
        ]
        .spacing(15)
        .align_items(Alignment::Center)
        .into()
    }

    // Constrói a vista de logs
    fn view_logs(&self) -> Element<Message> {
        // Junta todos os logs num único bloco de texto para permitir a seleção
        let all_logs = self.logs.join("\n");

        let scrollable_logs = Scrollable::new(
            // O widget `text` permite a seleção de texto por padrão
            container(text(all_logs)).padding(10)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        container(scrollable_logs).into()
    }
}
