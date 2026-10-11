//! Zeebx Qt — o emulador de Zeebo com a interface em Qt Quick.
//!
//! Um frontend à parte do `egui-standalone`: `zeebx-qt` abre a biblioteca, e `zeebx-qt <jogo>` abre
//! direto o jogo. A linha de comando de depuração (`run`, `info`, `bench`...) continua no `zeebx`.

// No Windows, a versão de distribuição abre sem a janela de console atrás da interface.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod qt;

use std::process::ExitCode;

/// O nome com que o Wayland casa a janela com o `.desktop` instalado, de onde tira o ícone. É o do
/// binário, que é o nome que o `cargo packager` dá ao `.desktop`.
const APP_ID: &str = "zeebx-qt";

/// Ativa caminhos portáteis antes de inicializar o Qt ou o sistema de logs.
/// A pasta `portable` precisa existir ao lado do executável; sem ela, nada muda.
#[cfg(windows)]
fn preparar_modo_portatil() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("não foi possível localizar o executável: {e}"))?;
    let pasta = exe.parent().ok_or_else(|| "o executável não tem pasta pai".to_string())?;
    let portable = pasta.join("portable");
    if !portable.is_dir() {
        return Ok(());
    }

    let config = portable.join("config");
    let dados = portable.join("data");
    let cache = portable.join("cache");
    for dir in [&config, &dados, &cache] {
        std::fs::create_dir_all(dir).map_err(|e| format!("não foi possível criar {}: {e}", dir.display()))?;
    }

    // Rust 2024 marca set_var como unsafe. Isto ocorre no início do main, antes de criar
    // QGuiApplication ou iniciar threads, para que Qt/QStandardPaths usem as pastas portáteis.
    unsafe {
        std::env::set_var("APPDATA", &config);
        std::env::set_var("LOCALAPPDATA", &dados);
        std::env::set_var("XDG_CONFIG_HOME", &config);
        std::env::set_var("XDG_DATA_HOME", &dados);
        std::env::set_var("XDG_CACHE_HOME", &cache);
        std::env::set_var("HOME", &portable);
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
        // Sai antes de criar o `QGuiApplication`, sem tela: o CI roda isto para provar que o
        // binário acha as bibliotecas do Qt, que o sistema carrega antes do `main`.
        Some("--version") => {
            println!("zeebx-qt {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        jogo => qt::launch(jogo),
    }
}
