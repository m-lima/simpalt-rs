mod git;

macro_rules! style {
    (reset) => {
        "[m"
    };

    (fg = $color: expr $(, $($param: expr)*)?) => {
        concat!("[3", $color, "m" $(, $($param),*)?)
    };

    (bg = $color: expr $(, $($param: expr)*)?) => {
        concat!("[4", $color, "m" $(, $($param),*)?)
    };

    (fg = $fg: expr, bg = $bg: expr $(, $($param: expr)*)?) => {
        concat!("[3", $fg, "m", "[4", $bg, "m" $(, $($param),*)?)
    };
}

macro_rules! color {
    (black) => {
        0
    };
    (red) => {
        1
    };
    (green) => {
        2
    };
    (yellow) => {
        3
    };
    (blue) => {
        4
    };
    (magenta) => {
        5
    };
    (cyan) => {
        6
    };
    (white) => {
        7
    };
    ([$param: literal]) => {
        concat!("8;5;", $param)
    };
    ([$r: literal, $g: literal, $b: literal]) => {
        concat!("8;2;", $r, ";", $g, ";", $b)
    };
    (reset) => {
        9
    };
}
macro_rules! symbol {
    (error) => {
        "✘"
    };
    (jobs) => {
        "" // "⚙"
    };
    (lock) => {
        "" // 
    };
    (venv) => {
        "☢" //     
    };
    (root) => {
        "☢" //  ⚡
    };
    (div thin) => {
        ""
    };
    (merge) => {
        ""
    };
    (bisect) => {
        ""
    };
    (rebase) => {
        ""
    };
    (cherry) => {
        ""
    };
    (revert) => {
        ""
    };
    (div) => {
        ""
    };
}

fn segment_pwd(path: &std::path::PathBuf) -> String {
    if let Ok(home) = std::env::var("HOME").map(std::path::PathBuf::from) {
        if home.eq(path) {
            String::from("~")
        } else {
            let (prefix, components) =
                path.components()
                    .fold((None, vec![]), |(prefix, mut list), curr| match curr {
                        std::path::Component::Prefix(prefix) => (Some(prefix), list),
                        std::path::Component::RootDir | std::path::Component::Normal(_) => {
                            list.push(curr);
                            (prefix, list)
                        }
                        std::path::Component::ParentDir => {
                            list.pop();
                            (prefix, list)
                        }
                        std::path::Component::CurDir => (prefix, list),
                    });

            if let Some(std::path::Component::Normal(path)) = components.last() {
                String::from(path.to_string_lossy())
            } else if let Some(prefix) = prefix {
                String::from(prefix.as_os_str().to_string_lossy())
            } else {
                String::from(std::path::MAIN_SEPARATOR)
            }
        }
    } else {
        println!(
            style!(fg = color!(red), "`HOME` environment variable not available" style!(reset))
        );
        path.to_str().map(String::from).unwrap()
    }
}

fn left(args: impl Iterator<Item = String>) {
    let (host, error, jobs) = args.fold((None, false, false), |acc, curr| {
        if curr == "-e" {
            (acc.0, true, acc.2)
        } else if curr == "-j" {
            (acc.0, acc.1, true)
        } else {
            (Some(curr), acc.1, acc.2)
        }
    });

    let (host, host_padding) = host.map_or_else(|| (String::new(), ""), |host| (host, " "));

    let error = if error {
        style!(fg = color!(red), symbol!(error) " ")
    } else {
        ""
    };

    let jobs = if jobs {
        style!(fg = color!(cyan), symbol!(jobs) " ")
    } else {
        ""
    };

    let venv = if std::env::var("VIRTUAL_ENV").is_ok() {
        style!(fg = color!(green), " ") // ")
    } else {
        ""
    };

    let pwd = std::env::current_dir()
        .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
        .ok();

    let prompt_pwd = if let Some(ref pwd) = pwd {
        segment_pwd(pwd)
    } else {
        String::new()
    };

    let prompt_git = if let Some(ref pwd) = pwd {
        println!("{:?}", git::prompt(pwd));
        style!(fg = color!(black), bg = color!(reset), symbol!(div))
    } else {
        style!(fg = color!(black), bg = color!(reset), symbol!(div))
    };

    print!(
        concat!(
            style!(bg = color!(black), " {error}{jobs}{venv}"),
            style!(fg = color!(reset), "{host}"),
            style!(reset),
            style!(bg = color!(black), "{host_padding}{prompt_pwd} "),
            "{prompt_git}",
            style!(reset),
        ),
        error = error,
        jobs = jobs,
        venv = venv,
        host_padding = host_padding,
        host = host,
        prompt_pwd = prompt_pwd,
        prompt_git = prompt_git,
    );
}

fn right() {
    use chrono::Timelike;

    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
    print!(
        style!(fg = color!([23]), "{h:02}:{m:02}:{s:02}" style!(reset)),
        h = time.hour(),
        m = time.minute(),
        s = time.second(),
    );
}

fn help(bin: Option<String>) {
    let bin = bin
        .map(std::path::PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
        })
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")));

    println!("Usage: {bin} <COMMAND> [HOST [e] [j]]",);
    println!();
    println!("Commands:");
    println!("  r  Generate right side prompt");
    println!("  l  Generate left side prompt");
    println!("  h  Show this help message");
    println!();
    println!("Arguments (only for left side prompt):");
    println!("  HOST  Symbol to be used as host (can contain ansii escape codes)");
    println!("  -e    Last command was an error");
    println!("  -j    There are background processes running");
}

fn main() {
    let mut args = std::env::args();
    let bin = args.next();
    let command = args.next();

    match command.as_deref() {
        Some("r") => right(),
        Some("l") => left(args),
        _ => help(bin),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn pwd_parsing() {
//         let tests = [
//             ("", "/"),
//             ("/", "/"),
//             ("a/", "a"),
//             ("a/b", "b"),
//             ("/a/b", "b"),
//             ("C:/a", "a"),
//             ("C:/", "C:"),
//             ("C:", "C:"),
//         ]
//         .map(|(a, b)| {
//             (
//                 a.replace('/', String::from(std::path::MAIN_SEPARATOR).as_str()),
//                 b.replace('/', String::from(std::path::MAIN_SEPARATOR).as_str()),
//             )
//         });
//
//         for (input, output) in tests {
//             assert_eq!(pwd(input), output);
//         }
//     }
// }
