# Changelog
Lista de alterações do projeto.

Esse formato é baseado no [Keep a Changelog](https://keepachangelog.com/pt/1.0.0/),
e esse projeto adere à [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]
### Adicionado
- Adiconado arquivo de configuração na compilação do arquivo

### Modificado
- Atualização do Serde

## [1.2.2] - 2021-01-01
### Modificado
- Mudança de cor de texo de fim
- Especicação de versão exata das libs usadas

### Correção
- Pegar o diretório do executável corretamente quando executado pela linha de comando

## [1.2.1] - 2020-12-31
### Modificado
- Atualização da lib rayon, chrono e blake3
- Aplicação de otimização opt-level3 para dependências
- Usa a lib console no lugar da colour para corrigir falhas do console

## [1.2.0] - 2020-09-27
### Adicionado
- Adição da configuração de extensão

### Modificado
- Compila com LTO ativado para diminuir o tamanho do binário e melhorar a performance
- Versão do linux compilado com musl para não depender do glibc
- Alterado nome da pasta e arquivo de configuração

## [1.1.1] - 2020-08-22
### Adicionado
- Uso de multithread
- Adição de cores nos textos de terminal

### Modificado
- Arquivo de configuração alterado para português
- Alterado o nome do arquivo para usar o hash blake3

## [1.1.0] - 2020-08-20
### Adicionado
- Adicionado manipulação dos canais RGB
- Adicionado o espelhamento vertical e horizontal
- Criação de novo arquivo de configuração em caso de arquivo inválido

### Modificado
- Adição de espaço entre os textos 
- Simplificado o import do serde
- Mais tratamento de erros com mensagens melhores
- Realização de cada ação somente em caso que for necessário

## [1.0.0] - 2020-08-18
### Adicionado
- Conversão de imagens para .JPG com configuração de altura e largura.