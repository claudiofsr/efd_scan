use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum MetodoContagem {
    /// Contagem exata percorrendo o arquivo (mais lento)
    Exata,
    /// Estimativa baseada no tamanho do arquivo (instantâneo)
    Estimativa,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum TipoBusca {
    /// Filtra pelo registro completo (ex: 0500)
    Registro,
    /// Filtra pelo bloco (ex: C, 0, E)
    Bloco,
    /// Tenta detectar automaticamente (1 char = Bloco, 4 chars = Registro)
    Auto,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "EFD Scan: Contador de registros ou blocos em arquivos SPED EFD Contribuições e ICMS/IPI",
    long_about = "Uma ferramenta otimizada que utiliza mapeamento de memória e processamento paralelo para contar ocorrências de registros ou blocos inteiros em arquivos SPED de qualquer tamanho."
)]
pub struct Args {
    /// O identificador (registro ou bloco) a ser contado.
    ///
    /// Pode ser um registro completo (ex: 'C100')
    /// ou apenas um bloco (ex: '0', 'A', 'C', ...).
    #[arg(short = 't', long, value_name = "REGISTRO/BLOCO")]
    pub target: String,

    /// O caminho completo ou relativo para o arquivo .txt do SPED.
    ///
    /// Exemplo: efd_scan -p PISCOFINS_202109*.txt -t 0
    #[arg(short = 'p', long, value_name = "ARQUIVO SPED")]
    pub path: PathBuf,

    /// Como o sistema deve interpretar o seu 'target'.
    ///
    /// 'Auto' (padrão) identifica automaticamente:
    /// - 1 letra é Bloco,
    /// - 4 caracteres é Registro.
    #[arg(
        short = 's',
        long,
        value_enum,
        default_value_t = TipoBusca::Auto,
        help = "Modo de interpretação do alvo"
    )]
    pub tipo: TipoBusca,

    /// Controla quando o processamento paralelo deve ser ativado.
    ///
    /// Para arquivos pequenos, o modo sequencial é mais rápido.
    ///
    /// Em arquivos gigantes, o modo paralelo reduz drasticamente o tempo de espera.
    #[arg(short = 'm', long, default_value_t = 100_000, value_name = "Nº LINHAS")]
    pub min_linhas: usize,

    /// Define como o total de linhas do arquivo será calculado para o relatório.
    ///
    /// 'Estimativa' (padrão) é instantâneo. 'Exata' conta linha por linha (mais lento).
    #[arg(short = 'c', long, value_enum, default_value_t = MetodoContagem::Estimativa)]
    pub contagem: MetodoContagem,
}
