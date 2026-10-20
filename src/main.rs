mod api;
mod barcode;
mod cloud;
mod config;
mod escpos;
mod models;
mod server;
mod storage;
mod sync;
mod terminal;
mod ui;

use anyhow::Result;
use config::AppConfig;
use server::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use wry::WebViewBuilder;

fn get_data_dir() -> PathBuf {
    if let Some(mut proj_dirs) = dirs_fallback() {
        proj_dirs.push("QordyPos");
        let _ = std::fs::create_dir_all(&proj_dirs);
        proj_dirs
    } else {
        PathBuf::from(".")
    }
}

fn dirs_fallback() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
    }
}

fn main() -> Result<()> {
    // 1. Loglama Başlat
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("========================================================");
    info!(" QORDY Esnaf POS v3.0 (Rust Edition - Offline-First)   ");
    info!("========================================================");

    // 2. Konfigürasyon Yükle (config.toml veya ortam değişkenleri)
    let app_config = AppConfig::load()?;
    info!("Konfigürasyon yüklendi: tenant={}, register={}", app_config.tenant_id, app_config.register_id);

    // 3. Tokio Async Runtime Başlat (Arka plan HTTP & Sync için)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // 4. Veritabanı Yolu ve Başlatma
    let data_dir = get_data_dir();
    let db_path = data_dir.join("qordy_pos.db");
    info!("SQLite Veritabanı: {}", db_path.display());

    let db = storage::DatabaseManager::new_with_config(&db_path, &app_config)?;

    // 5. Senkronizasyon Motoru
    let sync_engine = sync::SyncEngine::new(
        db.clone(),
        &app_config.cloud_url,
        &app_config.api_key,
        &app_config.tenant_id,
        &app_config.register_id,
    );

    let (ws_tx, _ws_rx) = tokio::sync::broadcast::channel(100);

    let terminal_config = terminal::TerminalConfig::default();
    let terminal_mgr = terminal::TerminalManager::new(terminal_config);

    // Arka plan işçilerini async runtime içinde başlat
    let app_state = Arc::new(AppState {
        db: db.clone(),
        sync: sync_engine.clone(),
        terminal_mgr: terminal_mgr.clone(),
        ws_tx,
        config: app_config.clone(),
    });

    let sync_clone = sync_engine.clone();
    let state_clone = app_state.clone();

    rt.spawn(async move {
        sync_clone.start_background_worker();
    });

    let port = app_config.port;
    rt.spawn(async move {
        if let Err(e) = server::start_server(state_clone, port).await {
            error!("HTTP Sunucu başlatılamadı: {}", e);
        }
    });

    // 6. Masaüstü Penceresi ve Wry Webview Başlat
    info!("Yerel Masaüstü Arayüzü başlatılıyor (Tao + Wry)...");
    let event_loop = EventLoop::new();

    let window_icon = {
        if let Ok(img) = image::load_from_memory(include_bytes!("../assets/images/favicon.png")) {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
        } else {
            None
        }
    };

    let mut win_builder = WindowBuilder::new()
        .with_title("QORDY Esnaf POS - Çevrimdışı Perakende & Hızlı Satış")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(1024.0, 680.0));

    if let Some(w_icon) = window_icon {
        win_builder = win_builder.with_window_icon(Some(w_icon));
    }

    let window = win_builder.build(&event_loop)?;

    #[cfg(target_os = "macos")]
    set_macos_dock_icon();

    let pos_url = format!("http://127.0.0.1:{}", port);

    // Webview oluştur
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    let webview = WebViewBuilder::new()
        .with_url(&pos_url)
        .build(&window)?;

    info!("Sistem hazır. Webview penceresi açıldı.");

    #[cfg(target_os = "macos")]
    let mut dock_icon_initialized = false;

    // Event Loop (Ana UI iş parçacığında çalışır)
    event_loop.run(move |event, _, control_flow| {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        let _ = &webview;
        *control_flow = ControlFlow::Wait;

        #[cfg(target_os = "macos")]
        if !dock_icon_initialized {
            set_macos_dock_icon();
            dock_icon_initialized = true;
        }

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                info!("Uygulama kapatılıyor...");
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}

#[cfg(target_os = "macos")]
#[allow(deprecated, unexpected_cfgs)]
fn set_macos_dock_icon() {
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let app: id = msg_send![objc::class!(NSApplication), sharedApplication];
        // Ensure regular activation policy so Dock icon is active
        let _: () = msg_send![app, setActivationPolicy: 0isize]; // NSApplicationActivationPolicyRegular = 0

        let mut set = false;

        // 1. Try embedded ICNS bytes directly
        let icns_bytes = include_bytes!("../assets/AppIcon.icns");
        let data: id = msg_send![objc::class!(NSData), dataWithBytes: icns_bytes.as_ptr() length: icns_bytes.len()];
        let img: id = msg_send![objc::class!(NSImage), alloc];
        let img: id = msg_send![img, initWithData: data];
        if img != nil {
            let _: () = msg_send![app, setApplicationIconImage: img];
            let dock_tile: id = msg_send![app, dockTile];
            let _: () = msg_send![dock_tile, display];
            info!("macOS Dock uygulama ikonu (embedded AppIcon.icns) başarıyla ayarlandı.");
            set = true;
        }

        // 2. Fallback to embedded PNG bytes
        if !set {
            let png_bytes = include_bytes!("../assets/favicon.png");
            let data: id = msg_send![objc::class!(NSData), dataWithBytes: png_bytes.as_ptr() length: png_bytes.len()];
            let img: id = msg_send![objc::class!(NSImage), alloc];
            let img: id = msg_send![img, initWithData: data];
            if img != nil {
                let _: () = msg_send![app, setApplicationIconImage: img];
                let dock_tile: id = msg_send![app, dockTile];
                let _: () = msg_send![dock_tile, display];
                info!("macOS Dock uygulama ikonu (embedded PNG) başarıyla ayarlandı.");
            }
        }
    }
}