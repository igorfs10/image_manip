use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::{ File, create_dir_all, read_to_string };
use std::path::Path;

use chrono::prelude::*;
use serde::{ Serialize, Deserialize };
use photon_rs::native::{ open_image, save_image };
use photon_rs::transform::{ resize, SamplingFilter};


const ARQUIVO_CONFIGURACAO: &str = "/config.toml";   // Nome do arquivo de configurações
const PASTA_CONVERSAO: &str = "/convert";            // Nome da pasta que será criada para colocar as imagens alteradas

// Configuração padrão. Quando arquivo de configuração não existe.
const LARGURA: u32 = 500;
const ALTURA: u32 = 500;

fn main() {
    // Carrega os argumentos enviados por linha de comando
    let args: Vec<String> = env::args().collect();

    // Pega o caminho do executável para criar a configuração e pasta de conversão
    let caminho = Path::new(&args[0]).parent().unwrap().to_str().unwrap();
    let caminho_configuracao = format!("{}{}", caminho, ARQUIVO_CONFIGURACAO);
    let caminho_conversao = format!("{}{}", caminho, PASTA_CONVERSAO);

    // Instancia do objeto de configurações
    let configuracoes: Config;

    // Verifica se o arquivo existe. Se existir carrega na variável configurações, caso contrário cria um arquivo com a configuração padrão.
    if Path::new(&caminho_configuracao).exists() {
        println!("Arquivo de configurações encontrado. Carregando configurações.");

        // Lê o arquivo de configuração em string e converte para o objeto
        let arquivo = read_to_string(caminho_configuracao).unwrap();
        configuracoes = toml::from_str(&arquivo).unwrap();

        println!("Configurações carregadas.");
    } else {
        println!("Arquivo de configurações não encontrado. Criando arquivo de configurações novo.");

        /* Cria o objeto de configuração padrão, converte o objeto em string,
            cria o arquivo e salva as configurações transformando a string em bytes */
        configuracoes = Config  { width: LARGURA, height: ALTURA };
        let conteudo = toml::to_string(&configuracoes).unwrap();
        let mut arquivo = File::create(caminho_configuracao).unwrap();
        arquivo.write_all(conteudo.as_bytes()).unwrap();

        println!("Arquivo criado.");
    }
    // Verifica se a pasta de conversão existe e automaticamente cria, caso não exista
    match create_dir_all(&caminho_conversao){
        Ok(_) => {
            // Percorre a array recebida por parâmetros começando pelo segundo elemento, pois o primeiro é o caminho da própria aplicação
            for i in 1..args.len(){

                // Abre a imagem recebida por parâmetro
                let img = open_image(&args[i]);
        
                // Increment the red channel by 200
                // photon_rs::channels::alter_red_channel(&mut img, 200);

                // Retorna uma nova imagem alterada
                let img_alterada = resize(&img, configuracoes.width, configuracoes.height, SamplingFilter::Lanczos3);

                // Remove a variável imagem que não será mais usada
                drop(img);

                // Pega a hora para gerar um nome de arquivo único
                let data: DateTime<Utc> = Utc::now();

                // Write file to filesystem.
                println!("Alterando imagem {}...", &args[i]);
                let nome_imagem = format!("{}/{}.jpg", &caminho_conversao, data.timestamp_millis());

                // Salva arquivo
                save_image(img_alterada, &nome_imagem);
                println!("Imagem salva em {}.", nome_imagem);
            }
        }
        Err(_) => {
            println!("Não foi possível criar a pasta de conversão.");
        }
    }
    pause();
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();


     println!("Finalizado.");

    // Queremos que o cursor fique no final da linha, então imprimimos sem uma linha nova
    write!(stdout, "Aperte enter para encerrar...").unwrap();
    stdout.flush().unwrap();

    // Lê um único byte e descarta
    let _ = stdin.read(&mut [0u8]).unwrap();
}

// Struct que cria e carrega arquivo de configuração
#[derive(Serialize, Deserialize)]
struct Config {
    width: u32,
    height: u32,
}