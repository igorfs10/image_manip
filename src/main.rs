use std::env;
use std::fs::{create_dir_all, read_to_string, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;

use chrono::prelude::*;
use console::style;
use photon_rs::channels;
use photon_rs::native::{open_image, save_image};
use photon_rs::transform;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const ARQUIVO_CONFIGURACAO: &str = "/image_manip_config.toml"; // Nome do arquivo de configurações
const PASTA_CONVERSAO: &str = "/image_manip_convert"; // Nome da pasta que será criada para colocar as imagens alteradas

fn main() {
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
        match toml::from_str(&arquivo) {
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
                let mut img = open_image(caminho_imagem).expect("Arquivo inválido");

                // Retorna uma nova imagem com dimensões alteradas se as medidas definidas forem diferentes de 0
                if configuracoes.largura != 0 && configuracoes.altura != 0 {
                    img = transform::resize(
                        &img,
                        configuracoes.largura,
                        configuracoes.altura,
                        transform::SamplingFilter::Lanczos3,
                    );
                }

                // Altera os canais da imagem se definidas
                if configuracoes.canal_vermelho != 0 {
                    channels::alter_blue_channel(&mut img, configuracoes.canal_vermelho);
                }
                if configuracoes.canal_verde != 0 {
                    channels::alter_green_channel(&mut img, configuracoes.canal_verde);
                }
                if configuracoes.canal_azul != 0 {
                    channels::alter_red_channel(&mut img, configuracoes.canal_azul);
                }

                // Rotaciona a imagem se for setado
                if configuracoes.espelhamento_horizontal {
                    transform::fliph(&mut img);
                }
                if configuracoes.espelhamento_vertical {
                    transform::flipv(&mut img);
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
                save_image(img, &nome_imagem);
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

    let conteudo = toml::to_string(&configuracoes).unwrap();
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
    canal_vermelho: i16,
    canal_verde: i16,
    canal_azul: i16,
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
            canal_vermelho: 0,
            canal_verde: 0,
            canal_azul: 0,
            espelhamento_horizontal: false,
            espelhamento_vertical: false,
            extensao: "jpg".to_string(),
        }
    }
}
