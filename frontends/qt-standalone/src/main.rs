//! Zeebx Qt — o emulador de Zeebo com a interface em Qt Quick.
//!
//! A interface do emulador: `zeebx-qt` abre a biblioteca, e `zeebx-qt <jogo>` abre direto o jogo. A
//! linha de comando de depuração (`run`, `info`, `bench`...) é o `zeebx`, em `frontends/cli`.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod qt;

use std::process::ExitCode;

/// O nome com que o Wayland casa a janela com o `.desktop` instalado.
const APP_ID: &str = "zeebx-qt";

/// Ativa o modo portátil se existir uma pasta `portable` ao lado do executável.
/// Esta função roda antes do Qt e do sistema de logs para que eles herdem os caminhos.
#[cfg(windows)]
fn preparar_modo_portatil() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("não foi possível localizar o executável: {e}"))?;
    let pasta = exe
        .parent()
        .ok_or_else(|| "o executável não tem pasta pai".to_string())?;
    let portable = pasta.join("portable");

    // Sem a pasta-sinal, mantém o comportamento padrão do Windows.
    if !portable.is_dir() {
        return Ok(());
    }

    let config = portable.join("config");
    let dados = portable.join("data");
    let cache = portable.join("cache");
    let cache_qml = cache.join("qml");
    for dir in [&config, &dados, &cache, &cache_qml] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("não foi possível criar {}: {e}", dir.display()))?;
    }

    // Rust 2024 exige unsafe para set_var. Isso ocorre antes de criar threads ou inicializar Qt.
    // APPDATA/LOCALAPPDATA cobrem bibliotecas que consultam essas variáveis; HOME e XDG cobrem
    // bibliotecas multiplataforma. As variáveis específicas do Qt direcionam seus caches quando
    // a versão do Qt usada pelo pacote as respeita.
    let pipeline_cache = cache.join("zeebx-qt.qtpipelinecache");
    unsafe {
        std::env::set_var("APPDATA", &config);
        std::env::set_var("LOCALAPPDATA", &dados);
        std::env::set_var("XDG_CONFIG_HOME", &config);
        std::env::set_var("XDG_DATA_HOME", &dados);
        std::env::set_var("XDG_CACHE_HOME", &cache);
        std::env::set_var("HOME", &portable);
        std::env::set_var("QML_DISK_CACHE_PATH", &cache_qml);
        std::env::set_var("QSG_RHI_PIPELINE_CACHE_LOAD", &pipeline_cache);
        std::env::set_var("QSG_RHI_PIPELINE_CACHE_SAVE", &pipeline_cache);
    }

    eprintln!("Zeebx: modo portátil ativado em {}", portable.display());
    Ok(())
}

#[cfg(not(windows))]
fn preparar_modo_portatil() -> Result<(), String> {
    Ok(())
}

fn main() -> ExitCode {
    if let Err(erro) = preparar_modo_portatil() {
        eprintln!("Zeebx: {erro}");
        return ExitCode::FAILURE;
    }
    zeebx::registro::le_do_ambiente();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        // O CI verifica que o binário acha as bibliotecas do Qt, sem abrir uma janela.
        Some("--version") => {
            println!("zeebx-qt {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        jogo => qt::launch(jogo),
    }
}
