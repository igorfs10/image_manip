use serde::{Deserialize, Serialize};

// Struct que cria e carrega arquivo de configuração
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub largura: u32,
    pub altura: u32,
    pub manter_proporcao: bool,
    pub espelhamento_horizontal: bool,
    pub espelhamento_vertical: bool,
    pub extensao: String,
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
