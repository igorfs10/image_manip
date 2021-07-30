//#![windows_subsystem = "windows"]

mod ui;

use std::env;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;

use chrono::prelude::*;
use console::style;
use fltk::enums::Shortcut;
use fltk::menu::MenuFlag;
use fltk::prelude::MenuExt;
use image::imageops;
use imageops::FilterType;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use fltk::app;

const ARQUIVO_CONFIGURACAO: &str = "/image_manip_config.json"; // Nome do arquivo de configurações
const PASTA_CONVERSAO: &str = "/image_manip_convert"; // Nome da pasta que será criada para colocar as imagens alteradas

const GUI: bool = true;

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
    ui.menu_bar
        .add("File/Exit", Shortcut::None, MenuFlag::Normal, |_| {
            std::process::exit(0);
        });
    app.run().unwrap();
}

fn console() {
    let start = Instant::now();
    // Carrega os argumentos enviados por linha de comando
    let args: Vec<String> = env::args().collect();
    let lista_imagens = args[1..args.len()].to_vec();

    // Pega o caminho do executável para criar a configuração e pasta de conversão
    let caminho_executavel_full = env::current_exe().unwrap();
    let caminho = caminho_executavel_full.parent().unwrap().to_str().unwrap();
    let caminho_configuracao = format!("{}{}", caminho, ARQUIVO_CONFIGURACAO);
    let caminho_conversao = format!("{}{}", caminho, PASTA_CONVERSAO);

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

// Struct que cria e carrega arquivo de configuração
#[derive(Serialize, Deserialize)]
struct Config {
    largura: u32,
    altura: u32,
    manter_proporcao: bool,
    espelhamento_horizontal: bool,
    espelhamento_vertical: bool,
    extensao: String,
}

// Implementa um valor padrão para inicializar a struct. Valor padrão de quando não houver configurações
impl Default for Config {
    fn default() -> Self {
        Config {
            largura: 0,
            altura: 0,
            manter_proporcao: false,
            espelhamento_horizontal: false,
            espelhamento_vertical: false,
            extensao: "jpg".to_string(),
        }
    }
}
