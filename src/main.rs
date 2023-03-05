use std::process::{self, ExitCode};

use clap::{Parser, Subcommand};
use url::Url;
use similar::{TextDiff, ChangeTag};

use vim_fmi::{Controller, read_user};
use vim_fmi::vim::{Vim, Keylog};

#[derive(Debug, Parser)]
#[command(name = "vim-fmi")]
#[command(about = "Клиент за курса по Vim във ФМИ", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Инициализира потребителя, който ще праща решения
    #[command(arg_required_else_help = true)]
    Setup {
        /// Токен, генериран в сайта (https://vim-fmi.bg/user_tokens)
        user_token: String,
    },

    /// Стартира упражнение с подадения идентификатор
    #[command(arg_required_else_help = true)]
    Put {
        /// Идентификатора на дадено упражнение
        task_id: String,
    },

    /// Стартира Vim-а, който програмата може да намери. За тестване
    Vim,

    /// Показва текущата версия на клиента
    Version,
}

fn main() -> ExitCode {
    let args = Cli::parse();

    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(args: &Cli) -> anyhow::Result<()> {
    let host =
        if cfg!(debug_assertions) {
            Url::parse("http://localhost:3000")?
        } else {
            Url::parse("https://vim-fmi.bg")?
        };

    match &args.command {
        Commands::Vim => {
            let controller = Controller::new(host.clone())?;
            let input_path = controller.create_file("scratch", "")?;
            let log_path = controller.create_file("log", "")?;
            let vimrc_path = controller.vimrc_path();
            let vim = Vim::new(input_path, log_path, vimrc_path);

            let (_, log_bytes) = vim.run()?;

            let keylog = Keylog::new(&log_bytes);
            let script: String = keylog.into_iter().collect();
            println!("Клавишите ти бяха:\n{}", script);
        },
        Commands::Setup { user_token } => {
            let controller = Controller::new(host.clone())?;
            let _ = controller.setup_user(&user_token)?;

            println!("Токена ти е активиран, вече можеш да пускаш решения");
        },
        Commands::Put { task_id } => {
            if read_user()?.is_none() {
                eprintln!("Не си се активирал на този компютър.");
                eprintln!("Иди в сайта (https://vim-fmi.bg/user_tokens), създай си token и извикай:");
                eprintln!();
                eprintln!("  vim-fmi setup <token>");
                eprintln!();
                process::exit(1);
            }

            let controller = Controller::new(host.clone())?;
            let task = controller.download_task(&task_id)?;

            let input_path = controller.create_file("input", &task.input)?;
            let log_path = controller.create_file("log", "")?;
            let vimrc_path = controller.vimrc_path();
            let vim = Vim::new(input_path, log_path, vimrc_path);

            let (output, log_bytes) = vim.run()?;
            let keylog = Keylog::new(&log_bytes);
            let script: String = keylog.into_iter().collect();
            let trimmed_output = output.trim();

            if trimmed_output == task.output {
                if controller.upload(&task_id, log_bytes)? {
                    println!("Супер, решението е качено. Клавишите ти бяха:\n{}", script);
                } else {
                    println!("Имаше проблем при качване на решението, пробвай пак.");
                    println!("Ако не проработи 2-3 пъти, пиши в Discord или по мейл.");
                }
            } else {
                println!("Не се получи, клавишите ти бяха:\n{}", script);
                println!();
                println!("Ето ти разликата между твоя опит и очаквания:");
                println!();
                print_diff(&task.output, trimmed_output);
            }
        },
        Commands::Version => {
            println!(::clap::crate_version!());
        },
    }

    Ok(())
}

fn print_diff(input: &str, output: &str) {
    let diff = TextDiff::from_lines(input, output);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }
}
