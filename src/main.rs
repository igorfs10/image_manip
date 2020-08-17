use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::create_dir_all;

use chrono::prelude::*;
use photon_rs::native::{ open_image, save_image };
use photon_rs::transform::{ resize, SamplingFilter};

fn main() {
    let args: Vec<String> = env::args().collect();
    match create_dir_all("convert"){
        Ok(_) => {
            for i in 1..args.len(){
                println!("{}", &args[i]);
                let img = open_image(&args[i]);
        
                // Increment the red channel by 200
                // photon_rs::channels::alter_red_channel(&mut img, 200);
                let img_resized = resize(&img, 500, 500, SamplingFilter::Lanczos3);

                // Pega a hora para gerar um nome de arquivo
                let data: DateTime<Utc> = Utc::now();

                // Write file to filesystem.
                println!("Salvando imagem {}...", &args[i]);
                let nome_imagem = format!("convert/{}.jpg", data.timestamp_millis());

                // Salva arquivo
                save_image(img_resized, &nome_imagem);
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

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Aperte enter para encerrar...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}