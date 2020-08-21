use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::{ File, create_dir_all, read_to_string };
use std::path::Path;

use chrono::prelude::*;
use serde::{ Serialize, Deserialize };
use photon_rs::native::{ open_image, save_image };
use photon_rs::transform;
use photon_rs::channels;


const ARQUIVO_CONFIGURACAO: &str = "/config.toml";   // Nome do arquivo de configurações
const PASTA_CONVERSAO: &str = "/convert";            // Nome da pasta que será criada para colocar as imagens alteradas

fn main() {
    // Carrega os argumentos enviados por linha de comando
    let args: Vec<String> = env::args().collect();

    // Pega o caminho do executável para criar a configuração e pasta de conversão
    let caminho = Path::new(&args[0]).parent().unwrap().to_str().unwrap();
    let caminho_configuracao = format!("{}{}", caminho, ARQUIVO_CONFIGURACAO);
    let caminho_conversao = format!("{}{}", caminho, PASTA_CONVERSAO);

    // Instancia do objeto de configurações
    let configuracoes: Config;

    /* Verifica se o arquivo existe. Se existir carrega na variável configurações,
        caso contrário cria um arquivo com a configuração padrão. */
    if Path::new(&caminho_configuracao).exists() {
        println!("Arquivo de configurações encontrado. Carregando configurações...");

        let arquivo = read_to_string(&caminho_configuracao).expect("Não foi possível ler o arquivo de configuração.");
        
        // Tentar ler o arquivo de configuração em string e converte para o objeto em caso de erro interrompe a aplicação
        match toml::from_str(&arquivo) {
            Ok(arquivo_convertido) => {
                configuracoes = arquivo_convertido;
                println!("Configurações carregadas.");
            }
            Err(_) => {
                println!("Não foi possível carregar o arquivo de configurações, criando um arquivo novo...");
                configuracoes = criar_configuracoes(&caminho_configuracao);
            }
        }
    } else {
        println!("Arquivo de configurações não encontrado. Criando arquivo de configurações novo...");

        /* Cria o objeto de configuração padrão, converte o objeto em string,
            cria o arquivo e salva as configurações transformando a string em bytes */
        configuracoes = criar_configuracoes(&caminho_configuracao);
    }

    println!("\n");

    // Verifica se a pasta de conversão existe e automaticamente cria, caso não exista
    match create_dir_all(&caminho_conversao){
        Ok(_) => {
            // Percorre a array recebida por parâmetros começando pelo segundo elemento, pois o primeiro é o caminho da própria aplicação
            for i in 1..args.len(){
                println!("Alterando imagem {}...", &args[i]);

                // Abre a imagem recebida por parâmetro
                let mut img = open_image(&args[i]);
                
                // Retorna uma nova imagem com dimensões alteradas se as medidas definidas forem diferentes de 0
                if configuracoes.width != 0 && configuracoes.height != 0 {
                    img = transform::resize(&img, configuracoes.width, configuracoes.height, transform::SamplingFilter::Lanczos3);
                }

                // Altera os canais da imagem se definidas
                if configuracoes.red_channel != 0 { channels::alter_red_channel(&mut img, configuracoes.red_channel); }
                if configuracoes.green_channel != 0 { channels::alter_green_channel(&mut img, configuracoes.green_channel); }
                if configuracoes.blue_channel != 0 { channels::alter_blue_channel(&mut img, configuracoes.red_channel); }

                // Rotaciona a imagem se for setado
                if configuracoes.horizontal_flip { transform::fliph(&mut img); }
                if configuracoes.vertical_flip { transform::flipv(&mut img); }

                // Pega a hora para gerar um nome de arquivo único
                let data: DateTime<Utc> = Utc::now();

                // Salva arquivo
                let nome_imagem = format!("{}/{}.jpg", &caminho_conversao, data.timestamp_millis());
                save_image(img, &nome_imagem);
                println!("Imagem salva em {}.\n\n", nome_imagem);
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

    println!("Pronto.");

    // Queremos que o cursor fique no final da linha, então imprimimos sem uma linha nova
    write!(stdout, "Aperte enter para encerrar...").unwrap();
    stdout.flush().unwrap();

    // Lê um único byte e descarta
    let _ = stdin.read(&mut [0u8]).unwrap();
}

// Função que cria as configurações e carrega as configurações
fn criar_configuracoes(caminho_configuracao: &str) -> Config {
    
    /* Cria o objeto de configuração padrão, converte o objeto em string,
    cria o arquivo e salva as configurações transformando a string em bytes */
    let configuracoes = Config::default();
    
    let conteudo = toml::to_string(&configuracoes).unwrap();
    let mut arquivo = File::create(caminho_configuracao).expect("Não foi possível criar o arquivo de configuração.");
    arquivo.write_all(conteudo.as_bytes()).expect("Não foi possível salvar o arquivo de configuração.");

    println!("Arquivo de configurações criado.");

    return configuracoes;
}

// Struct que cria e carrega arquivo de configuração
#[derive(Serialize, Deserialize)]
struct Config {
    width: u32,
    height: u32,
    red_channel: i16,
    green_channel: i16,
    blue_channel: i16,
    horizontal_flip: bool,
    vertical_flip: bool
}

// Implementa um valor padrão para inicializar a struct. Valor padrão de quando não houver configurações
impl Default for Config {
    fn default() -> Self {
        Config {
            width: 0,
            height: 0,
            red_channel: 0,
            green_channel: 0,
            blue_channel: 0,
            horizontal_flip: false,
            vertical_flip: false
        }
    }
}
