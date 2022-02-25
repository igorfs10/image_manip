//#![windows_subsystem = "windows"]

mod config;
mod ui;

use std::collections::VecDeque;
use std::env;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;

use chrono::prelude::*;
use console::style;
use fltk::prelude::{BrowserExt, MenuExt, WidgetExt};
use fltk::{app, dialog};
use fltk_theme::{ThemeType, WidgetTheme};
use image::imageops;
use imageops::FilterType;
use rayon::prelude::*;

use config::Config;

const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "bmp", "gif"];
const IMAGE_EXTENSION_FILTER: &str = "*.{jpg,jpeg,png,bmp,gif}";

const CONFIG_FILE: &str = "/image_manip_config.json"; // Nome do arquivo de configurações
const CONVERSION_FOLDER: &str = "/image_manip_convert"; // Nome da pasta que será criada para colocar as imagens alteradas

const GUI: bool = false;

fn main() {
    if GUI {
        gui();
    } else {
        console();
    }
}

fn gui() {
    let app = app::App::default();
    let mut ui = ui::ImageManip::make_window();
    let mut menu_item_exit = ui.menu_bar.find_item("File/Exit").unwrap();
    let mut args: VecDeque<String> = env::args().collect();
    let exe_folder = get_exe_location();
    let config_file_path = format!("{}{}", exe_folder, CONFIG_FILE);
    let conversion_folder_path = format!("{}{}", exe_folder, CONVERSION_FOLDER);
    let configuration = get_config_file(&config_file_path);
    println!("{} {}", configuration.largura, conversion_folder_path);

    WidgetTheme::new(ThemeType::Metro).apply();

    menu_item_exit.set_callback(|_| {
        std::process::exit(0);
    });

    args.pop_front();

    for arg in args {
        if is_image_extension(&arg) {
            ui.browser_files.add(&arg);
        }
    }

    let browser_files = ui.browser_files.clone();
    ui.button_convert.set_callback(move |_| {
        for i in 1..=browser_files.size() {
            println!("{}", browser_files.text(i).unwrap());
        }
    });

    let mut browser_files = ui.browser_files.clone();
    ui.button_clear.set_callback(move |_| {
        browser_files.clear();
    });

    let mut browser_files = ui.browser_files.clone();
    ui.button_add.set_callback(move |_| {
        let files = open_file_dialog();
        'outer: for file in files {
            for i in 1..=browser_files.size() {
                if browser_files.text(i).unwrap() == file {
                    continue 'outer;
                }
            }
            browser_files.add(&file);
        }
    });

    app.run().unwrap();
}

fn console() {
    let start = Instant::now();
    // Carrega os argumentos enviados por linha de comando
    let args: Vec<String> = env::args().collect();
    let lista_imagens = args[1..args.len()].to_vec();

    // Pega o caminho do executável para criar a configuração e pasta de conversão
    let caminho = get_exe_location();
    let caminho_configuracao = format!("{}{}", caminho, CONFIG_FILE);
    let caminho_conversao = format!("{}{}", caminho, CONVERSION_FOLDER);

    // Instancia do objeto de configurações
    let configuracoes: Config;

    /* Verifica se o arquivo existe. Se existir carrega na variável configurações,
    caso contrário cria um arquivo com a configuração padrão. */
    if Path::new(&caminho_configuracao).exists() {
        println!(
            "{}",
            style("Arquivo de configurações encontrado. Carregando configurações...").yellow()
        );

        let arquivo = read_to_string(&caminho_configuracao)
            .expect("Não foi possível ler o arquivo de configuração");

        // Tentar ler o arquivo de configuração em string e converte para o objeto em caso de erro interrompe a aplicação
        match serde_json::from_str(&arquivo) {
            Ok(arquivo_convertido) => {
                configuracoes = arquivo_convertido;
                println!("{}", style("Configurações carregadas").green());
            }
            Err(_) => {
                println!("{}", style("Não foi possível carregar o arquivo de configurações, criando um arquivo novo...").yellow());
                configuracoes = criar_configuracoes(&caminho_configuracao);
            }
        }
    } else {
        println!(
            "{}",
            style(
                "Arquivo de configurações não encontrado. Criando arquivo de configurações novo..."
            )
            .yellow()
        );

        /* Cria o objeto de configuração padrão, converte o objeto em string,
        cria o arquivo e salva as configurações transformando a string em bytes */
        configuracoes = criar_configuracoes(&caminho_configuracao);
    }

    println!("\n");

    // Verifica se a pasta de conversão existe e automaticamente cria, caso não exista
    match create_dir_all(&caminho_conversao) {
        Ok(_) => {
            // Percorre a array de caminho das imagens
            lista_imagens.par_iter().for_each(|caminho_imagem| {
                println!(
                    "{}",
                    style(format!("Alterando imagem {}...", caminho_imagem)).yellow()
                );

                // Abre a imagem recebida por parâmetro
                let mut img =
                    image::open(caminho_imagem).expect("Não foi possível abrir o arquivo");

                // Retorna uma nova imagem com dimensões alteradas se as medidas definidas forem diferentes de 0
                if configuracoes.largura != 0 && configuracoes.altura != 0 {
                    if configuracoes.manter_proporcao {
                        img = img.resize(
                            configuracoes.largura,
                            configuracoes.altura,
                            FilterType::Lanczos3,
                        );
                    } else {
                        img = img.resize_exact(
                            configuracoes.largura,
                            configuracoes.altura,
                            FilterType::Lanczos3,
                        );
                    }
                }

                // Rotaciona a imagem se for setado
                if configuracoes.espelhamento_horizontal {
                    img = img.fliph();
                }
                if configuracoes.espelhamento_vertical {
                    img = img.flipv();
                }

                // Pega a hora para gerar um nome de arquivo único
                let data: DateTime<Utc> = Utc::now();

                // Cria uma chave usando a hora atual e o caminho total do arquivo para criar a hash
                let mut chave = caminho_imagem.as_bytes().to_vec();
                chave.extend(data.timestamp_millis().to_be_bytes().iter());

                // Gera um hash para nomear o arquivo
                let hash = blake3::hash(&chave);

                // Gera o caminho do arquivo
                let nome_imagem = format!(
                    "{}/{}.{}",
                    &caminho_conversao,
                    hash.to_hex(),
                    configuracoes.extensao
                );

                // Salvar imagem
                img.save(&nome_imagem)
                    .expect("Não foi possível salvar a images");
                println!(
                    "{}",
                    style(format!(
                        "Imagem {} salva em {}",
                        caminho_imagem, nome_imagem
                    ))
                    .green()
                );
            });
        }
        Err(_) => {
            println!(
                "{}",
                style("Não foi possível criar a pasta de conversão").red()
            );
        }
    }
    let duration = start.elapsed();
    println!("Operação terminada em: {:?}", duration);
    pause();
}

fn pause() {
    let mut stdin = io::stdin();

    println!("{}", style("\n\nFim").blue());

    // Queremos que o cursor fique no final da linha, então imprimimos sem uma linha nova
    println!("{}", style("Aperte enter para encerrar...").blue());

    // Lê um único byte e descarta
    let _ = stdin.read(&mut [0u8]).unwrap();
}

// Função que cria as configurações e carrega as configurações
fn criar_configuracoes(caminho_configuracao: &str) -> Config {
    /* Cria o objeto de configuração padrão, converte o objeto em string,
    cria o arquivo e salva as configurações transformando a string em bytes */
    let configuracoes = Config::default();

    let conteudo = serde_json::to_string(&configuracoes).unwrap();
    let mut arquivo = File::create(caminho_configuracao)
        .expect("Não foi possível criar o arquivo de configuração");
    arquivo
        .write_all(conteudo.as_bytes())
        .expect("Não foi possível salvar o arquivo de configuração");

    println!("{}", style("Arquivo de configurações criado").green());

    configuracoes
}

fn get_exe_location() -> String {
    let exe_path = env::current_exe().unwrap();
    exe_path.parent().unwrap().to_str().unwrap().to_string()
}

fn is_image_extension(path_string: &str) -> bool {
    let path = Path::new(path_string);
    if let Some(extension) = path.extension() {
        let extension = extension.to_ascii_lowercase();
        IMAGE_EXTENSIONS.contains(&extension.to_str().unwrap())
    } else {
        false
    }
}

fn open_file_dialog() -> Vec<String> {
    let mut file = fltk::dialog::FileDialog::new(dialog::FileDialogType::BrowseMultiFile);
    file.set_filter(IMAGE_EXTENSION_FILTER);
    file.show();
    let mut paths = Vec::new();
    for path in file.filenames() {
        if is_image_extension(path.to_str().unwrap()) {
            paths.push(path.to_str().unwrap().to_string())
        }
    }
    paths
}

fn create_config_file(config_path: &str) -> Config {
    let configuration = Config::default();

    let content = serde_json::to_string(&configuration).unwrap();
    let mut file = File::create(config_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    configuration
}

fn get_config_file(config_path: &str) -> Config {
    let configuration: Config;
    if Path::new(config_path).exists() {
        let file = read_to_string(&config_path).unwrap();

        match serde_json::from_str(&file) {
            Ok(config) => {
                configuration = config;
            }
            Err(_) => {
                configuration = create_config_file(config_path);
            }
        }
    } else {
        configuration = create_config_file(config_path);
    }
    configuration
}
