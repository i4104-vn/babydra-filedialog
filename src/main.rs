//! BabyDra FileDialog — Native File Chooser, XDG Desktop Portal & xdg-open replacement.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;

use gtk4::prelude::*;
use gtk4::Application;

use babydra_core::services::file_dialog::{
    start_portal_service, FileDialogMode, FileDialogOptions, FileDialogResult, FileFilter,
    PortalDialogRequest,
};
use babydra_ui_kit::components::modals::file_chooser::show_file_chooser_dialog;

fn print_help() {
    println!("babydra-filedialog - BabyDra File Dialog & Portal Service");
    println!();
    println!("USAGE:");
    println!("    babydra-filedialog [OPTIONS] [INITIAL_PATH]");
    println!("    babydra-filedialog --open <FILE_OR_URL>");
    println!("    xdg-open <FILE_OR_URL>");
    println!();
    println!("OPTIONS:");
    println!("    -f, --open-file          Open a single file (default)");
    println!("    -d, --open-folder        Open a directory / folder");
    println!("    -m, --open-multiple      Open multiple files");
    println!("    -s, --save-file          Save file dialog");
    println!("    -t, --title <TITLE>      Set dialog title");
    println!("    -p, --path <PATH>        Set initial folder path");
    println!("    -n, --name <NAME>        Set initial filename for saving");
    println!("    --filter <SPEC>          Filter specification (e.g. 'Images:*.png,*.jpg')");
    println!("    --portal                 Run as XDG Desktop Portal D-Bus service daemon");
    println!("    --open <TARGET>          Open file or URL with default app (xdg-open)");
    println!("    -h, --help               Print this help message");
    println!();
    println!("XDG-OPEN REPLACEMENT:");
    println!("    When invoked as 'xdg-open' or with '--open', opens targets in default apps.");
}

fn handle_xdg_open(target: &str) -> i32 {
    let _ = gtk4::init();

    let uri = if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("file://")
        || target.starts_with("mailto:")
    {
        target.to_string()
    } else {
        let path = Path::new(target);
        if path.is_dir() {
            babydra_core::services::explore::show_in_file_manager(path);
            return 0;
        }
        let abs = if path.is_relative() {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        };
        let canonical = std::fs::canonicalize(&abs).unwrap_or(abs);
        match url::Url::from_file_path(&canonical) {
            Ok(u) => u.to_string(),
            Err(_) => format!("file://{}", canonical.to_string_lossy()),
        }
    };

    if gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>).is_ok() {
        0
    } else {
        eprintln!("Failed to open URI: {uri}");
        1
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let bin_name = Path::new(&args[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Drop-in replacement when called as 'xdg-open'
    if bin_name == "xdg-open" {
        if args.len() < 2 {
            eprintln!("xdg-open: file or URL required");
            eprintln!("Try 'xdg-open --help' for more information.");
            std::process::exit(1);
        }
        let mut exit_code = 0;
        for arg in &args[1..] {
            if arg == "-h" || arg == "--help" {
                println!("Usage: xdg-open {{ file | URL }}");
                std::process::exit(0);
            }
            if arg == "--version" {
                println!("babydra-filedialog (xdg-open compatibility) 1.0.0");
                std::process::exit(0);
            }
            if handle_xdg_open(arg) != 0 {
                exit_code = 1;
            }
        }
        std::process::exit(exit_code);
    }

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    // Portal daemon mode
    let is_portal = args.iter().any(|a| a == "--portal");

    // Explicit --open flag
    if let Some(pos) = args.iter().position(|a| a == "--open") {
        if let Some(target) = args.get(pos + 1) {
            let code = handle_xdg_open(target);
            std::process::exit(code);
        } else {
            eprintln!("--open requires a target file, folder, or URL");
            std::process::exit(1);
        }
    }

    // Direct URL argument
    if !is_portal && args.len() == 2 && (args[1].starts_with("http://") || args[1].starts_with("https://") || args[1].starts_with("mailto:")) {
        let exit_code = handle_xdg_open(&args[1]);
        std::process::exit(exit_code);
    }

    // Parse options for CLI dialog invocation
    let mut mode = FileDialogMode::OpenFile;
    let mut custom_title: Option<String> = None;
    let mut initial_path: Option<PathBuf> = None;
    let mut initial_name: Option<String> = None;
    let mut filters: Vec<FileFilter> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-d" || arg == "--open-folder" {
            mode = FileDialogMode::OpenFolder;
        } else if arg == "-f" || arg == "--open-file" {
            mode = FileDialogMode::OpenFile;
        } else if arg == "-m" || arg == "--open-multiple" {
            mode = FileDialogMode::OpenMultipleFiles;
        } else if arg == "-s" || arg == "--save-file" {
            mode = FileDialogMode::SaveFile;
        } else if arg == "-t" || arg == "--title" {
            if i + 1 < args.len() {
                custom_title = Some(args[i + 1].clone());
                i += 1;
            }
        } else if arg == "-p" || arg == "--path" {
            if i + 1 < args.len() {
                initial_path = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
        } else if arg == "-n" || arg == "--name" {
            if i + 1 < args.len() {
                initial_name = Some(args[i + 1].clone());
                i += 1;
            }
        } else if arg == "--filter" {
            if i + 1 < args.len() {
                let spec = &args[i + 1];
                if let Some((name, pats)) = spec.split_once(':') {
                    let pat_list: Vec<String> = pats.split(',').map(|s| s.trim().to_string()).collect();
                    filters.push(FileFilter {
                        name: name.to_string(),
                        patterns: pat_list,
                        mime_types: Vec::new(),
                    });
                }
                i += 1;
            }
        } else if !arg.starts_with('-') && initial_path.is_none() {
            let p = PathBuf::from(arg);
            if p.is_dir() {
                initial_path = Some(p);
            } else if let Some(parent) = p.parent() {
                initial_path = Some(parent.to_path_buf());
                initial_name = p.file_name().and_then(|f| f.to_str()).map(|s| s.to_string());
            }
        }
        i += 1;
    }

    if filters.is_empty() {
        filters.push(FileFilter::all_files());
    }

    let default_title = match mode {
        FileDialogMode::OpenFolder => "Open Folder",
        FileDialogMode::OpenFile => "Open File",
        FileDialogMode::OpenMultipleFiles => "Open Files",
        FileDialogMode::SaveFile => "Save File",
    };

    let title = custom_title.unwrap_or_else(|| default_title.to_string());

    let opts = FileDialogOptions {
        title,
        mode,
        initial_folder: initial_path,
        initial_file: initial_name,
        accept_label: None,
        filters,
        default_filter_index: Some(0),
        show_hidden: false,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let app = Application::builder()
        .application_id(if is_portal {
            "org.babydra.filedialog.portal"
        } else {
            "org.babydra.filedialog.cli"
        })
        .flags(if is_portal {
            gio::ApplicationFlags::IS_SERVICE
        } else {
            gio::ApplicationFlags::NON_UNIQUE
        })
        .build();

    if is_portal {
        // Run as XDG Desktop Portal Service
        let (request_tx, request_rx) =
            tokio::sync::mpsc::unbounded_channel::<PortalDialogRequest>();
        let request_rx = Rc::new(RefCell::new(Some(request_rx)));

        app.connect_activate(move |portal_app| {
            // Keep portal service running continuously
            let _daemon_hold = portal_app.hold();
            std::mem::forget(_daemon_hold);

            babydra_ui_kit::ui::theme::init_theme();

            // Background task to serve D-Bus portal requests
            let req_tx = request_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = start_portal_service(req_tx).await {
                    eprintln!("Failed to register XDG Desktop Portal service: {e}");
                }
            });

            // Handle incoming portal dialog requests on GTK main context
            if let Some(mut rx) = request_rx.borrow_mut().take() {
                glib::spawn_future_local(async move {
                    while let Some(req) = rx.recv().await {
                        let response_tx = Rc::new(RefCell::new(Some(req.response_tx)));

                        show_file_chooser_dialog(None, req.options, move |result| {
                            if let Some(tx) = response_tx.borrow_mut().take() {
                                let dialog_res = match result {
                                    Some(paths) => FileDialogResult::success(paths),
                                    None => FileDialogResult::cancel(),
                                };
                                let _ = tx.send(dialog_res);
                            }
                        });
                    }
                });
            }
        });
    } else {
        // Run as single dialog CLI invocation
        let opts_clone = opts.clone();
        app.connect_activate(move |_app| {
            babydra_ui_kit::ui::theme::init_theme();

            show_file_chooser_dialog(None, opts_clone.clone(), |result| {
                match result {
                    Some(paths) => {
                        for p in paths {
                            println!("{}", p.to_string_lossy());
                        }
                        std::process::exit(0);
                    }
                    None => {
                        std::process::exit(1);
                    }
                }
            });
        });
    }

    let code = app.run_with_args(&["babydra-filedialog"]);
    std::process::exit(code.value());
}
