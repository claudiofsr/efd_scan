mod args;

use args::{Args, MetodoContagem, TipoBusca};
use clap::Parser;
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let start_time = Instant::now();

    // 1. Mapeamento do arquivo em memória
    let arquivo = File::open(&args.path)?;
    let mmap = unsafe { Mmap::map(&arquivo)? };

    // Otimização para sistemas Unix-like (Linux/macOS)
    // Se estiver lidando com arquivos realmente massivos (vários GBs),
    // "avisar" ao sistema operacional para ler o arquivo de forma sequencial após o Mmap.
    // Isso reduz o latency de page faults durante o processamento paralelo.
    #[cfg(unix)]
    mmap.advise(memmap2::Advice::Sequential)?;

    // 2. Determinação do padrão de busca (prefixo em bytes)
    // SPED sempre começa com '|'. Se for registro: '|0500|'. Se for bloco: '|C'.
    let target_limpo = args.target.trim_matches('|');

    let (prefixo, modo_desc) = match args.tipo {
        TipoBusca::Bloco => (format!("|{}", target_limpo), "Bloco"),
        TipoBusca::Registro => (format!("|{}|", target_limpo), "Registro"),
        TipoBusca::Auto => {
            if target_limpo.len() == 1 {
                (format!("|{}", target_limpo), "Bloco (Auto)")
            } else {
                (format!("|{}|", target_limpo), "Registro (Auto)")
            }
        }
    };
    let prefix_bytes = prefixo.as_bytes();

    // 3. Determinação do número de linhas (Metadados)
    let (numero_de_linhas, label_contagem) = match args.contagem {
        MetodoContagem::Exata => {
            // Uso de SIMD via bytecount em pedaços paralelos
            let count = mmap
                .par_chunks(8 * 1024 * 1024) // 8MB chunks
                .map(|chunk| bytecount::count(chunk, b'\n'))
                .sum();
            (count, "Exata")
        }
        MetodoContagem::Estimativa => {
            // Estimativa baseada na média de 100 bytes/linha do SPED
            (mmap.len() / 100, "Estimada")
        }
    };

    // 4. Processamento funcional e paralelo
    // split(|&b| b == b'\n') gera fatias (slices) sem copiar os dados
    let processar = |linha: &[u8]| -> bool {
        // Remove espaços, \t, \n, \r das extremidades sem alocar memória
        let trimmed = linha.trim_ascii();

        // Verifica se a linha começa com o prefixo e ignora linhas vazias
        !trimmed.is_empty() && trimmed.starts_with(prefix_bytes)
    };

    // 5. Execução (Funcional e Eficiente)
    let total = if numero_de_linhas < args.min_linhas {
        // Sequencial: utiliza Iterators nativos do Rust
        mmap.split(|&b| b == b'\n').filter(|l| processar(l)).count()
    } else {
        // Paralelo: Rayon divide o arquivo em chunks e processa em todos os cores
        mmap.par_split(|&b| b == b'\n')
            .filter(|l| processar(l))
            .count()
    };

    let modo_proc = if numero_de_linhas < args.min_linhas {
        "Sequencial"
    } else {
        "Paralelo"
    };

    // 6. Output formatado
    println!(
        "--------------------------------------------------\n\
         Arquivo:          {}\n\
         Tamanho:          {:.2} MB\n\
         Nº de Linhas:     {} ({})\n\
         Busca por:        {} ({})\n\
         Processamento:    {}\n\
         Total encontrado: {}\n\
         Tempo decorrido:  {:.2?}\n\
         --------------------------------------------------",
        args.path.display(),
        mmap.len() as f64 / 1_048_576.0,
        numero_de_linhas,
        label_contagem,
        prefixo,
        modo_desc,
        modo_proc,
        total,
        start_time.elapsed()
    );

    Ok(())
}
